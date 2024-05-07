// Copyright (C) Saitama (UK) Ltd.
// This file is part of SaitaChain.

// Saitama is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Saitama is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with SaitaChain.  If not, see <http://www.gnu.org/licenses/>.

//! The [`Candidates`] store tracks information about advertised candidates
//! as well as which peers have advertised them.
//!
//! Due to the request-oriented nature of this protocol, we often learn
//! about candidates just as a hash, alongside claimed properties that the
//! receipt would commit to. However, it is only later on that we can
//! confirm those claimed properties. This store lets us keep track of
//! all candidates which are currently 'relevant' after spam-protection, and
//! gives us the ability to detect mis-advertisements after the fact
//! and punish them accordingly.

use saitama_node_network_protocol::PeerId;
use saitama_node_subsystem::messages::HypotheticalCandidate;
use saitama_primitives::vstaging::{
	CandidateHash, CommittedCandidateReceipt, GroupIndex, Hash, Id as ParaId,
	PersistedValidationData,
};

use std::{
	collections::{
		hash_map::{Entry, HashMap},
		HashSet,
	},
	sync::Arc,
};

/// This encapsulates the correct and incorrect advertisers
/// post-confirmation of a candidate.
#[derive(Debug, Default, PartialEq)]
pub struct PostConfirmationReckoning {
	/// Peers which advertised correctly.
	pub correct: HashSet<PeerId>,
	/// Peers which advertised the candidate incorrectly.
	pub incorrect: HashSet<PeerId>,
}

/// Outputs generated by initial confirmation of a candidate.
#[derive(Debug, PartialEq)]
pub struct PostConfirmation {
	/// The hypothetical candidate used to determine importability and membership
	/// in the hypothetical frontier.
	pub hypothetical: HypotheticalCandidate,
	/// A "reckoning" of peers who have advertised the candidate previously,
	/// either accurately or inaccurately.
	pub reckoning: PostConfirmationReckoning,
}

/// A tracker for all known candidates in the view.
///
/// See module docs for more info.
#[derive(Default)]
pub struct Candidates {
	candidates: HashMap<CandidateHash, CandidateState>,
	by_parent: HashMap<(Hash, ParaId), HashSet<CandidateHash>>,
}

impl Candidates {
	/// Insert an advertisement.
	///
	/// This should be invoked only after performing
	/// spam protection and only for advertisements that
	/// are valid within the current view. [`Candidates`] never prunes
	/// candidate by peer ID, to avoid peers skirting misbehavior
	/// reports by disconnecting intermittently. Therefore, this presumes
	/// that spam protection limits the peers which can send advertisements
	/// about unconfirmed candidates.
	///
	/// It returns either `Ok(())` or an immediate error in the
	/// case that the candidate is already known and reality conflicts
	/// with the advertisement.
	pub fn insert_unconfirmed(
		&mut self,
		peer: PeerId,
		candidate_hash: CandidateHash,
		claimed_relay_parent: Hash,
		claimed_group_index: GroupIndex,
		claimed_parent_hash_and_id: Option<(Hash, ParaId)>,
	) -> Result<(), BadAdvertisement> {
		let entry = self.candidates.entry(candidate_hash).or_insert_with(|| {
			CandidateState::Unconfirmed(UnconfirmedCandidate {
				claims: Vec::new(),
				parent_claims: HashMap::new(),
				unconfirmed_importable_under: HashSet::new(),
			})
		});

		match entry {
			CandidateState::Confirmed(ref c) => {
				if c.relay_parent() != claimed_relay_parent {
					return Err(BadAdvertisement)
				}

				if c.group_index() != claimed_group_index {
					return Err(BadAdvertisement)
				}

				if let Some((claimed_parent_hash, claimed_id)) = claimed_parent_hash_and_id {
					if c.parent_head_data_hash() != claimed_parent_hash {
						return Err(BadAdvertisement)
					}

					if c.para_id() != claimed_id {
						return Err(BadAdvertisement)
					}
				}
			},
			CandidateState::Unconfirmed(ref mut c) => {
				c.add_claims(
					peer,
					CandidateClaims {
						relay_parent: claimed_relay_parent,
						group_index: claimed_group_index,
						parent_hash_and_id: claimed_parent_hash_and_id,
					},
				);

				if let Some(parent_claims) = claimed_parent_hash_and_id {
					self.by_parent.entry(parent_claims).or_default().insert(candidate_hash);
				}
			},
		}

		Ok(())
	}

	/// Note that a candidate has been confirmed. If the candidate has just been
	/// confirmed (previous state was `Unconfirmed`), then this returns `Some`. Otherwise, `None`.
	///
	/// If we are confirming for the first time, then remove any outdated claims, and generate a
	/// reckoning of which peers advertised correctly and incorrectly.
	///
	/// This does no sanity-checking of input data, and will overwrite already-confirmed candidates.
	pub fn confirm_candidate(
		&mut self,
		candidate_hash: CandidateHash,
		candidate_receipt: CommittedCandidateReceipt,
		persisted_validation_data: PersistedValidationData,
		assigned_group: GroupIndex,
	) -> Option<PostConfirmation> {
		let parent_hash = persisted_validation_data.parent_head.hash();
		let relay_parent = candidate_receipt.descriptor().relay_parent;
		let para_id = candidate_receipt.descriptor().para_id;

		let prev_state = self.candidates.insert(
			candidate_hash,
			CandidateState::Confirmed(ConfirmedCandidate {
				receipt: Arc::new(candidate_receipt),
				persisted_validation_data,
				assigned_group,
				parent_hash,
				importable_under: HashSet::new(),
			}),
		);
		let new_confirmed =
			match self.candidates.get_mut(&candidate_hash).expect("just inserted; qed") {
				CandidateState::Confirmed(x) => x,
				_ => panic!("just inserted as confirmed; qed"),
			};

		self.by_parent.entry((parent_hash, para_id)).or_default().insert(candidate_hash);

		match prev_state {
			None => Some(PostConfirmation {
				reckoning: Default::default(),
				hypothetical: new_confirmed.to_hypothetical(candidate_hash),
			}),
			Some(CandidateState::Confirmed(_)) => None,
			Some(CandidateState::Unconfirmed(u)) => Some({
				let mut reckoning = PostConfirmationReckoning::default();

				for (leaf_hash, x) in u.unconfirmed_importable_under {
					if x.relay_parent == relay_parent &&
						x.parent_hash == parent_hash &&
						x.para_id == para_id
					{
						new_confirmed.importable_under.insert(leaf_hash);
					}
				}

				for (peer, claims) in u.claims {
					// Update the by-parent-hash index not to store any outdated
					// claims.
					if let Some((claimed_parent_hash, claimed_id)) = claims.parent_hash_and_id {
						if claimed_parent_hash != parent_hash || claimed_id != para_id {
							if let Entry::Occupied(mut e) =
								self.by_parent.entry((claimed_parent_hash, claimed_id))
							{
								e.get_mut().remove(&candidate_hash);
								if e.get().is_empty() {
									e.remove();
								}
							}
						}
					}

					if claims.check(relay_parent, assigned_group, parent_hash, para_id) {
						reckoning.correct.insert(peer);
					} else {
						reckoning.incorrect.insert(peer);
					}
				}

				PostConfirmation {
					reckoning,
					hypothetical: new_confirmed.to_hypothetical(candidate_hash),
				}
			}),
		}
	}

	/// Whether a candidate is confirmed.
	pub fn is_confirmed(&self, candidate_hash: &CandidateHash) -> bool {
		match self.candidates.get(candidate_hash) {
			Some(CandidateState::Confirmed(_)) => true,
			_ => false,
		}
	}

	/// Get a reference to the candidate, if it's known and confirmed.
	pub fn get_confirmed(&self, candidate_hash: &CandidateHash) -> Option<&ConfirmedCandidate> {
		match self.candidates.get(candidate_hash) {
			Some(CandidateState::Confirmed(ref c)) => Some(c),
			_ => None,
		}
	}

	/// Whether statements from a candidate are importable.
	///
	/// This is only true when the candidate is known, confirmed,
	/// and is importable in a fragment tree.
	pub fn is_importable(&self, candidate_hash: &CandidateHash) -> bool {
		self.get_confirmed(candidate_hash).map_or(false, |c| c.is_importable(None))
	}

	/// Note that a candidate is importable in a fragment tree indicated by the given
	/// leaf hash.
	pub fn note_importable_under(&mut self, candidate: &HypotheticalCandidate, leaf_hash: Hash) {
		match candidate {
			HypotheticalCandidate::Incomplete {
				candidate_hash,
				candidate_para,
				parent_head_data_hash,
				candidate_relay_parent,
			} => {
				let u = UnconfirmedImportable {
					relay_parent: *candidate_relay_parent,
					parent_hash: *parent_head_data_hash,
					para_id: *candidate_para,
				};

				if let Some(&mut CandidateState::Unconfirmed(ref mut c)) =
					self.candidates.get_mut(&candidate_hash)
				{
					c.note_maybe_importable_under(leaf_hash, u);
				}
			},
			HypotheticalCandidate::Complete { candidate_hash, .. } => {
				if let Some(&mut CandidateState::Confirmed(ref mut c)) =
					self.candidates.get_mut(&candidate_hash)
				{
					c.importable_under.insert(leaf_hash);
				}
			},
		}
	}

	/// Get all hypothetical candidates which should be tested
	/// for inclusion in the frontier.
	///
	/// Provide optional parent parablock information to filter hypotheticals to only
	/// potential children of that parent.
	pub fn frontier_hypotheticals(
		&self,
		parent: Option<(Hash, ParaId)>,
	) -> Vec<HypotheticalCandidate> {
		fn extend_hypotheticals<'a>(
			v: &mut Vec<HypotheticalCandidate>,
			i: impl IntoIterator<Item = (&'a CandidateHash, &'a CandidateState)>,
			maybe_required_parent: Option<(Hash, ParaId)>,
		) {
			for (c_hash, candidate) in i {
				match candidate {
					CandidateState::Unconfirmed(u) =>
						u.extend_hypotheticals(*c_hash, v, maybe_required_parent),
					CandidateState::Confirmed(c) => v.push(c.to_hypothetical(*c_hash)),
				}
			}
		}

		let mut v = Vec::new();
		if let Some(parent) = parent {
			let maybe_children = self.by_parent.get(&parent);
			let i = maybe_children
				.into_iter()
				.flatten()
				.filter_map(|c_hash| self.candidates.get_key_value(c_hash));

			extend_hypotheticals(&mut v, i, Some(parent));
		} else {
			extend_hypotheticals(&mut v, self.candidates.iter(), None);
		}
		v
	}

	/// Prune all candidates according to the relay-parent predicate
	/// provided.
	pub fn on_deactivate_leaves(
		&mut self,
		leaves: &[Hash],
		relay_parent_live: impl Fn(&Hash) -> bool,
	) {
		let by_parent = &mut self.by_parent;
		let mut remove_parent_claims = |c_hash, parent_hash, id| {
			if let Entry::Occupied(mut e) = by_parent.entry((parent_hash, id)) {
				e.get_mut().remove(&c_hash);
				if e.get().is_empty() {
					e.remove();
				}
			}
		};
		self.candidates.retain(|c_hash, state| match state {
			CandidateState::Confirmed(ref mut c) =>
				if !relay_parent_live(&c.relay_parent()) {
					remove_parent_claims(*c_hash, c.parent_head_data_hash(), c.para_id());
					false
				} else {
					for leaf_hash in leaves {
						c.importable_under.remove(leaf_hash);
					}
					true
				},
			CandidateState::Unconfirmed(ref mut c) => {
				c.on_deactivate_leaves(
					leaves,
					|parent_hash, id| remove_parent_claims(*c_hash, parent_hash, id),
					&relay_parent_live,
				);
				c.has_claims()
			},
		})
	}
}

/// A bad advertisement was recognized.
#[derive(Debug, PartialEq)]
pub struct BadAdvertisement;

#[derive(Debug, PartialEq)]
enum CandidateState {
	Unconfirmed(UnconfirmedCandidate),
	Confirmed(ConfirmedCandidate),
}

/// Claims made alongside the advertisement of a candidate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CandidateClaims {
	/// The relay-parent committed to by the candidate.
	relay_parent: Hash,
	/// The group index assigned to this candidate.
	group_index: GroupIndex,
	/// The hash of the parent head-data and the ParaId. This is optional,
	/// as only some types of advertisements include this data.
	parent_hash_and_id: Option<(Hash, ParaId)>,
}

impl CandidateClaims {
	fn check(
		&self,
		relay_parent: Hash,
		group_index: GroupIndex,
		parent_hash: Hash,
		para_id: ParaId,
	) -> bool {
		self.relay_parent == relay_parent &&
			self.group_index == group_index &&
			self.parent_hash_and_id.map_or(true, |p| p == (parent_hash, para_id))
	}
}

// properties of an unconfirmed but hypothetically importable candidate.
#[derive(Debug, Hash, PartialEq, Eq)]
struct UnconfirmedImportable {
	relay_parent: Hash,
	parent_hash: Hash,
	para_id: ParaId,
}

// An unconfirmed candidate may have have been advertised under
// multiple identifiers. We track here, on the basis of unique identifier,
// the peers which advertised each candidate in a specific way.
#[derive(Debug, PartialEq)]
struct UnconfirmedCandidate {
	claims: Vec<(PeerId, CandidateClaims)>,
	// ref-counted
	parent_claims: HashMap<(Hash, ParaId), Vec<(Hash, usize)>>,
	unconfirmed_importable_under: HashSet<(Hash, UnconfirmedImportable)>,
}

impl UnconfirmedCandidate {
	fn add_claims(&mut self, peer: PeerId, claims: CandidateClaims) {
		// This does no deduplication, but this is only called after
		// spam prevention is already done. In practice we expect that
		// each peer will be able to announce the same candidate about 1 time per live relay-parent,
		// but in doing so it limits the amount of other candidates it can advertise. on balance,
		// memory consumption is bounded in the same way.
		if let Some(parent_claims) = claims.parent_hash_and_id {
			let sub_claims = self.parent_claims.entry(parent_claims).or_default();
			match sub_claims.iter().position(|x| x.0 == claims.relay_parent) {
				Some(p) => sub_claims[p].1 += 1,
				None => sub_claims.push((claims.relay_parent, 1)),
			}
		}
		self.claims.push((peer, claims));
	}

	fn note_maybe_importable_under(
		&mut self,
		active_leaf: Hash,
		unconfirmed_importable: UnconfirmedImportable,
	) {
		self.unconfirmed_importable_under.insert((active_leaf, unconfirmed_importable));
	}

	fn on_deactivate_leaves(
		&mut self,
		leaves: &[Hash],
		mut remove_parent_index: impl FnMut(Hash, ParaId),
		relay_parent_live: impl Fn(&Hash) -> bool,
	) {
		self.claims.retain(|c| {
			if relay_parent_live(&c.1.relay_parent) {
				true
			} else {
				if let Some(parent_claims) = c.1.parent_hash_and_id {
					if let Entry::Occupied(mut e) = self.parent_claims.entry(parent_claims) {
						if let Some(p) = e.get().iter().position(|x| x.0 == c.1.relay_parent) {
							let sub_claims = e.get_mut();
							sub_claims[p].1 -= 1;
							if sub_claims[p].1 == 0 {
								sub_claims.remove(p);
							}
						};

						if e.get().is_empty() {
							remove_parent_index(parent_claims.0, parent_claims.1);
							e.remove();
						}
					}
				}

				false
			}
		});

		self.unconfirmed_importable_under
			.retain(|(l, props)| leaves.contains(l) && relay_parent_live(&props.relay_parent));
	}

	fn extend_hypotheticals(
		&self,
		candidate_hash: CandidateHash,
		v: &mut Vec<HypotheticalCandidate>,
		required_parent: Option<(Hash, ParaId)>,
	) {
		fn extend_hypotheticals_inner<'a>(
			candidate_hash: CandidateHash,
			v: &mut Vec<HypotheticalCandidate>,
			i: impl IntoIterator<Item = (&'a (Hash, ParaId), &'a Vec<(Hash, usize)>)>,
		) {
			for ((parent_head_hash, para_id), possible_relay_parents) in i {
				for (relay_parent, _rc) in possible_relay_parents {
					v.push(HypotheticalCandidate::Incomplete {
						candidate_hash,
						candidate_para: *para_id,
						parent_head_data_hash: *parent_head_hash,
						candidate_relay_parent: *relay_parent,
					});
				}
			}
		}

		match required_parent {
			Some(parent) => extend_hypotheticals_inner(
				candidate_hash,
				v,
				self.parent_claims.get_key_value(&parent),
			),
			None => extend_hypotheticals_inner(candidate_hash, v, self.parent_claims.iter()),
		}
	}

	fn has_claims(&self) -> bool {
		!self.claims.is_empty()
	}
}

/// A confirmed candidate.
#[derive(Debug, PartialEq)]
pub struct ConfirmedCandidate {
	receipt: Arc<CommittedCandidateReceipt>,
	persisted_validation_data: PersistedValidationData,
	assigned_group: GroupIndex,
	parent_hash: Hash,
	// active leaves statements about this candidate are importable under.
	importable_under: HashSet<Hash>,
}

impl ConfirmedCandidate {
	/// Get the relay-parent of the candidate.
	pub fn relay_parent(&self) -> Hash {
		self.receipt.descriptor().relay_parent
	}

	/// Get the para-id of the candidate.
	pub fn para_id(&self) -> ParaId {
		self.receipt.descriptor().para_id
	}

	/// Get the underlying candidate receipt.
	pub fn candidate_receipt(&self) -> &Arc<CommittedCandidateReceipt> {
		&self.receipt
	}

	/// Get the persisted validation data.
	pub fn persisted_validation_data(&self) -> &PersistedValidationData {
		&self.persisted_validation_data
	}

	/// Whether the candidate is importable.
	pub fn is_importable<'a>(&self, under_active_leaf: impl Into<Option<&'a Hash>>) -> bool {
		match under_active_leaf.into() {
			Some(h) => self.importable_under.contains(h),
			None => !self.importable_under.is_empty(),
		}
	}

	/// Get the parent head data hash.
	pub fn parent_head_data_hash(&self) -> Hash {
		self.parent_hash
	}

	/// Get the group index of the assigned group. Note that this is in the context
	/// of the state of the chain at the candidate's relay parent and its para-id.
	pub fn group_index(&self) -> GroupIndex {
		self.assigned_group
	}

	fn to_hypothetical(&self, candidate_hash: CandidateHash) -> HypotheticalCandidate {
		HypotheticalCandidate::Complete {
			candidate_hash,
			receipt: self.receipt.clone(),
			persisted_validation_data: self.persisted_validation_data.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use saitama_primitives::HeadData;
	use saitama_primitives_test_helpers::make_candidate;

	#[test]
	fn inserting_unconfirmed_rejects_on_incompatible_claims() {
		let relay_head_data_a = HeadData(vec![1, 2, 3]);
		let relay_head_data_b = HeadData(vec![4, 5, 6]);
		let relay_hash_a = relay_head_data_a.hash();
		let relay_hash_b = relay_head_data_b.hash();

		let para_id_a = 1.into();
		let para_id_b = 2.into();

		let (candidate_a, pvd_a) = make_candidate(
			relay_hash_a,
			1,
			para_id_a,
			relay_head_data_a,
			HeadData(vec![1]),
			Hash::from_low_u64_be(1000).into(),
		);

		let candidate_hash_a = candidate_a.hash();

		let peer = PeerId::random();

		let group_index_a = 100.into();
		let group_index_b = 200.into();

		let mut candidates = Candidates::default();

		// Confirm a candidate first.
		candidates.confirm_candidate(candidate_hash_a, candidate_a, pvd_a, group_index_a);

		// Relay parent does not match.
		assert_eq!(
			candidates.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash_b,
				group_index_a,
				Some((relay_hash_a, para_id_a)),
			),
			Err(BadAdvertisement)
		);

		// Group index does not match.
		assert_eq!(
			candidates.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash_a,
				group_index_b,
				Some((relay_hash_a, para_id_a)),
			),
			Err(BadAdvertisement)
		);

		// Parent head data does not match.
		assert_eq!(
			candidates.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash_a,
				group_index_a,
				Some((relay_hash_b, para_id_a)),
			),
			Err(BadAdvertisement)
		);

		// Para ID does not match.
		assert_eq!(
			candidates.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash_a,
				group_index_a,
				Some((relay_hash_a, para_id_b)),
			),
			Err(BadAdvertisement)
		);

		// Everything matches.
		assert_eq!(
			candidates.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash_a,
				group_index_a,
				Some((relay_hash_a, para_id_a)),
			),
			Ok(())
		);
	}

	// Tests that:
	//
	// - When the advertisement matches, confirming does not change the parent hash index.
	// - When it doesn't match, confirming updates the index. Specifically, confirming should prune
	//   unconfirmed claims.
	#[test]
	fn confirming_maintains_parent_hash_index() {
		let relay_head_data = HeadData(vec![1, 2, 3]);
		let relay_hash = relay_head_data.hash();

		let candidate_head_data_a = HeadData(vec![1]);
		let candidate_head_data_b = HeadData(vec![2]);
		let candidate_head_data_c = HeadData(vec![3]);
		let candidate_head_data_d = HeadData(vec![4]);
		let candidate_head_data_hash_a = candidate_head_data_a.hash();
		let candidate_head_data_hash_b = candidate_head_data_b.hash();
		let candidate_head_data_hash_c = candidate_head_data_c.hash();

		let (candidate_a, pvd_a) = make_candidate(
			relay_hash,
			1,
			1.into(),
			relay_head_data,
			candidate_head_data_a.clone(),
			Hash::from_low_u64_be(1000).into(),
		);
		let (candidate_b, pvd_b) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_a,
			candidate_head_data_b.clone(),
			Hash::from_low_u64_be(2000).into(),
		);
		let (candidate_c, _) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_b.clone(),
			candidate_head_data_c.clone(),
			Hash::from_low_u64_be(3000).into(),
		);
		let (candidate_d, pvd_d) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_c.clone(),
			candidate_head_data_d,
			Hash::from_low_u64_be(4000).into(),
		);

		let candidate_hash_a = candidate_a.hash();
		let candidate_hash_b = candidate_b.hash();
		let candidate_hash_c = candidate_c.hash();
		let candidate_hash_d = candidate_d.hash();

		let peer = PeerId::random();
		let group_index = 100.into();

		let mut candidates = Candidates::default();

		// Insert some unconfirmed candidates.

		// Advertise A without parent hash.
		candidates
			.insert_unconfirmed(peer, candidate_hash_a, relay_hash, group_index, None)
			.ok()
			.unwrap();
		assert_eq!(candidates.by_parent, HashMap::default());

		// Advertise A with parent hash and ID.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_a,
				relay_hash,
				group_index,
				Some((relay_hash, 1.into())),
			)
			.ok()
			.unwrap();
		assert_eq!(
			candidates.by_parent,
			HashMap::from([((relay_hash, 1.into()), HashSet::from([candidate_hash_a]))])
		);

		// Advertise B with parent A.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_b,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				((candidate_head_data_hash_a, 1.into()), HashSet::from([candidate_hash_b]))
			])
		);

		// Advertise C with parent A.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_c,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c])
				)
			])
		);

		// Advertise D with parent A.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_d,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c, candidate_hash_d])
				)
			])
		);

		// Insert confirmed candidates and check parent hash index.

		// Confirmation matches advertisement. Index should be unchanged.
		candidates.confirm_candidate(candidate_hash_a, candidate_a, pvd_a, group_index);
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c, candidate_hash_d])
				)
			])
		);
		candidates.confirm_candidate(candidate_hash_b, candidate_b, pvd_b, group_index);
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c, candidate_hash_d])
				)
			])
		);

		// Confirmation does not match advertisement. Index should be updated.
		candidates.confirm_candidate(candidate_hash_d, candidate_d, pvd_d, group_index);
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c])
				),
				((candidate_head_data_hash_c, 1.into()), HashSet::from([candidate_hash_d]))
			])
		);

		// Make a new candidate for C with a different para ID.
		let (new_candidate_c, new_pvd_c) = make_candidate(
			relay_hash,
			1,
			2.into(),
			candidate_head_data_b,
			candidate_head_data_c.clone(),
			Hash::from_low_u64_be(3000).into(),
		);
		candidates.confirm_candidate(candidate_hash_c, new_candidate_c, new_pvd_c, group_index);
		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				((candidate_head_data_hash_a, 1.into()), HashSet::from([candidate_hash_b])),
				((candidate_head_data_hash_b, 2.into()), HashSet::from([candidate_hash_c])),
				((candidate_head_data_hash_c, 1.into()), HashSet::from([candidate_hash_d]))
			])
		);
	}

	#[test]
	fn test_returned_post_confirmation() {
		let relay_head_data = HeadData(vec![1, 2, 3]);
		let relay_hash = relay_head_data.hash();

		let candidate_head_data_a = HeadData(vec![1]);
		let candidate_head_data_b = HeadData(vec![2]);
		let candidate_head_data_c = HeadData(vec![3]);
		let candidate_head_data_d = HeadData(vec![4]);
		let candidate_head_data_hash_a = candidate_head_data_a.hash();
		let candidate_head_data_hash_b = candidate_head_data_b.hash();

		let (candidate_a, pvd_a) = make_candidate(
			relay_hash,
			1,
			1.into(),
			relay_head_data,
			candidate_head_data_a.clone(),
			Hash::from_low_u64_be(1000).into(),
		);
		let (candidate_b, pvd_b) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_a.clone(),
			candidate_head_data_b.clone(),
			Hash::from_low_u64_be(2000).into(),
		);
		let (candidate_c, _) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_a.clone(),
			candidate_head_data_c.clone(),
			Hash::from_low_u64_be(3000).into(),
		);
		let (candidate_d, pvd_d) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_b.clone(),
			candidate_head_data_d,
			Hash::from_low_u64_be(4000).into(),
		);

		let candidate_hash_a = candidate_a.hash();
		let candidate_hash_b = candidate_b.hash();
		let candidate_hash_c = candidate_c.hash();
		let candidate_hash_d = candidate_d.hash();

		let peer_a = PeerId::random();
		let peer_b = PeerId::random();
		let peer_c = PeerId::random();
		let peer_d = PeerId::random();

		let group_index = 100.into();

		let mut candidates = Candidates::default();

		// Insert some unconfirmed candidates.

		// Advertise A without parent hash.
		candidates
			.insert_unconfirmed(peer_a, candidate_hash_a, relay_hash, group_index, None)
			.ok()
			.unwrap();

		// Advertise A with parent hash and ID.
		candidates
			.insert_unconfirmed(
				peer_a,
				candidate_hash_a,
				relay_hash,
				group_index,
				Some((relay_hash, 1.into())),
			)
			.ok()
			.unwrap();

		// (Correctly) advertise B with parent A. Do it from a couple of peers.
		candidates
			.insert_unconfirmed(
				peer_a,
				candidate_hash_b,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();
		candidates
			.insert_unconfirmed(
				peer_b,
				candidate_hash_b,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();

		// (Wrongly) advertise C with parent A. Do it from a couple peers.
		candidates
			.insert_unconfirmed(
				peer_b,
				candidate_hash_c,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();
		candidates
			.insert_unconfirmed(
				peer_c,
				candidate_hash_c,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();

		// Advertise D. Do it correctly from one peer (parent B) and wrongly from another (parent
		// A).
		candidates
			.insert_unconfirmed(
				peer_c,
				candidate_hash_d,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_b, 1.into())),
			)
			.ok()
			.unwrap();
		candidates
			.insert_unconfirmed(
				peer_d,
				candidate_hash_d,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();

		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c, candidate_hash_d])
				),
				((candidate_head_data_hash_b, 1.into()), HashSet::from([candidate_hash_d]))
			])
		);

		// Insert confirmed candidates and check parent hash index.

		// Confirmation matches advertisement.
		let post_confirmation = candidates.confirm_candidate(
			candidate_hash_a,
			candidate_a.clone(),
			pvd_a.clone(),
			group_index,
		);
		assert_eq!(
			post_confirmation,
			Some(PostConfirmation {
				hypothetical: HypotheticalCandidate::Complete {
					candidate_hash: candidate_hash_a,
					receipt: Arc::new(candidate_a),
					persisted_validation_data: pvd_a,
				},
				reckoning: PostConfirmationReckoning {
					correct: HashSet::from([peer_a]),
					incorrect: HashSet::from([]),
				},
			})
		);

		let post_confirmation = candidates.confirm_candidate(
			candidate_hash_b,
			candidate_b.clone(),
			pvd_b.clone(),
			group_index,
		);
		assert_eq!(
			post_confirmation,
			Some(PostConfirmation {
				hypothetical: HypotheticalCandidate::Complete {
					candidate_hash: candidate_hash_b,
					receipt: Arc::new(candidate_b),
					persisted_validation_data: pvd_b,
				},
				reckoning: PostConfirmationReckoning {
					correct: HashSet::from([peer_a, peer_b]),
					incorrect: HashSet::from([]),
				},
			})
		);

		// Confirm candidate with two wrong peers (different group index).
		let (new_candidate_c, new_pvd_c) = make_candidate(
			relay_hash,
			1,
			2.into(),
			candidate_head_data_b,
			candidate_head_data_c.clone(),
			Hash::from_low_u64_be(3000).into(),
		);
		let post_confirmation = candidates.confirm_candidate(
			candidate_hash_c,
			new_candidate_c.clone(),
			new_pvd_c.clone(),
			group_index,
		);
		assert_eq!(
			post_confirmation,
			Some(PostConfirmation {
				hypothetical: HypotheticalCandidate::Complete {
					candidate_hash: candidate_hash_c,
					receipt: Arc::new(new_candidate_c),
					persisted_validation_data: new_pvd_c,
				},
				reckoning: PostConfirmationReckoning {
					correct: HashSet::from([]),
					incorrect: HashSet::from([peer_b, peer_c]),
				},
			})
		);

		// Confirm candidate with one wrong peer (different parent head data).
		let post_confirmation = candidates.confirm_candidate(
			candidate_hash_d,
			candidate_d.clone(),
			pvd_d.clone(),
			group_index,
		);
		assert_eq!(
			post_confirmation,
			Some(PostConfirmation {
				hypothetical: HypotheticalCandidate::Complete {
					candidate_hash: candidate_hash_d,
					receipt: Arc::new(candidate_d),
					persisted_validation_data: pvd_d,
				},
				reckoning: PostConfirmationReckoning {
					correct: HashSet::from([peer_c]),
					incorrect: HashSet::from([peer_d]),
				},
			})
		);
	}

	#[test]
	fn test_hypothetical_frontiers() {
		let relay_head_data = HeadData(vec![1, 2, 3]);
		let relay_hash = relay_head_data.hash();

		let candidate_head_data_a = HeadData(vec![1]);
		let candidate_head_data_b = HeadData(vec![2]);
		let candidate_head_data_c = HeadData(vec![3]);
		let candidate_head_data_d = HeadData(vec![4]);
		let candidate_head_data_hash_a = candidate_head_data_a.hash();
		let candidate_head_data_hash_b = candidate_head_data_b.hash();
		let candidate_head_data_hash_d = candidate_head_data_d.hash();

		let (candidate_a, pvd_a) = make_candidate(
			relay_hash,
			1,
			1.into(),
			relay_head_data,
			candidate_head_data_a.clone(),
			Hash::from_low_u64_be(1000).into(),
		);
		let (candidate_b, _) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_a.clone(),
			candidate_head_data_b.clone(),
			Hash::from_low_u64_be(2000).into(),
		);
		let (candidate_c, _) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_a.clone(),
			candidate_head_data_c.clone(),
			Hash::from_low_u64_be(3000).into(),
		);
		let (candidate_d, _) = make_candidate(
			relay_hash,
			1,
			1.into(),
			candidate_head_data_b.clone(),
			candidate_head_data_d,
			Hash::from_low_u64_be(4000).into(),
		);

		let candidate_hash_a = candidate_a.hash();
		let candidate_hash_b = candidate_b.hash();
		let candidate_hash_c = candidate_c.hash();
		let candidate_hash_d = candidate_d.hash();

		let peer = PeerId::random();
		let group_index = 100.into();

		let mut candidates = Candidates::default();

		// Confirm A.
		candidates.confirm_candidate(
			candidate_hash_a,
			candidate_a.clone(),
			pvd_a.clone(),
			group_index,
		);

		// Advertise B with parent A.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_b,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();

		// Advertise C with parent A.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_c,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_a, 1.into())),
			)
			.ok()
			.unwrap();

		// Advertise D with parent B.
		candidates
			.insert_unconfirmed(
				peer,
				candidate_hash_d,
				relay_hash,
				group_index,
				Some((candidate_head_data_hash_b, 1.into())),
			)
			.ok()
			.unwrap();

		assert_eq!(
			candidates.by_parent,
			HashMap::from([
				((relay_hash, 1.into()), HashSet::from([candidate_hash_a])),
				(
					(candidate_head_data_hash_a, 1.into()),
					HashSet::from([candidate_hash_b, candidate_hash_c])
				),
				((candidate_head_data_hash_b, 1.into()), HashSet::from([candidate_hash_d]))
			])
		);

		let hypothetical_a = HypotheticalCandidate::Complete {
			candidate_hash: candidate_hash_a,
			receipt: Arc::new(candidate_a),
			persisted_validation_data: pvd_a,
		};
		let hypothetical_b = HypotheticalCandidate::Incomplete {
			candidate_hash: candidate_hash_b,
			candidate_para: 1.into(),
			parent_head_data_hash: candidate_head_data_hash_a,
			candidate_relay_parent: relay_hash,
		};
		let hypothetical_c = HypotheticalCandidate::Incomplete {
			candidate_hash: candidate_hash_c,
			candidate_para: 1.into(),
			parent_head_data_hash: candidate_head_data_hash_a,
			candidate_relay_parent: relay_hash,
		};
		let hypothetical_d = HypotheticalCandidate::Incomplete {
			candidate_hash: candidate_hash_d,
			candidate_para: 1.into(),
			parent_head_data_hash: candidate_head_data_hash_b,
			candidate_relay_parent: relay_hash,
		};

		let hypotheticals = candidates.frontier_hypotheticals(Some((relay_hash, 1.into())));
		assert_eq!(hypotheticals.len(), 1);
		assert!(hypotheticals.contains(&hypothetical_a));

		let hypotheticals =
			candidates.frontier_hypotheticals(Some((candidate_head_data_hash_a, 2.into())));
		assert_eq!(hypotheticals.len(), 0);

		let hypotheticals =
			candidates.frontier_hypotheticals(Some((candidate_head_data_hash_a, 1.into())));
		assert_eq!(hypotheticals.len(), 2);
		assert!(hypotheticals.contains(&hypothetical_b));
		assert!(hypotheticals.contains(&hypothetical_c));

		let hypotheticals =
			candidates.frontier_hypotheticals(Some((candidate_head_data_hash_d, 1.into())));
		assert_eq!(hypotheticals.len(), 0);

		let hypotheticals = candidates.frontier_hypotheticals(None);
		assert_eq!(hypotheticals.len(), 4);
		assert!(hypotheticals.contains(&hypothetical_a));
		assert!(hypotheticals.contains(&hypothetical_b));
		assert!(hypotheticals.contains(&hypothetical_c));
		assert!(hypotheticals.contains(&hypothetical_d));
	}
}
