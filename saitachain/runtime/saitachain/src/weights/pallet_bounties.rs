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

//! Autogenerated weights for `pallet_bounties`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-18, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `runner-e8ezs4ez-project-163-concurrent-0`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("saitachain-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/saitachain
// benchmark
// pallet
// --chain=saitachain-dev
// --steps=50
// --repeat=20
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --pallet=pallet_bounties
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --header=./file_header.txt
// --output=./runtime/saitachain/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bounties`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bounties::WeightInfo for WeightInfo<T> {
	/// Storage: Bounties BountyCount (r:1 w:1)
	/// Proof: Bounties BountyCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Bounties BountyDescriptions (r:0 w:1)
	/// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(16400), added: 18875, mode: MaxEncodedLen)
	/// Storage: Bounties Bounties (r:0 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// The range of component `d` is `[0, 16384]`.
	fn propose_bounty(d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `177`
		//  Estimated: `3593`
		// Minimum execution time: 30_000_000 picoseconds.
		Weight::from_parts(31_021_890, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			// Standard Error: 6
			.saturating_add(Weight::from_parts(757, 0).saturating_mul(d.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: Bounties BountyApprovals (r:1 w:1)
	/// Proof: Bounties BountyApprovals (max_values: Some(1), max_size: Some(402), added: 897, mode: MaxEncodedLen)
	fn approve_bounty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `269`
		//  Estimated: `3642`
		// Minimum execution time: 11_055_000 picoseconds.
		Weight::from_parts(11_875_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	fn propose_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `3642`
		// Minimum execution time: 10_266_000 picoseconds.
		Weight::from_parts(10_581_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn unassign_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `465`
		//  Estimated: `3642`
		// Minimum execution time: 43_566_000 picoseconds.
		Weight::from_parts(44_671_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn accept_curator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `461`
		//  Estimated: `3642`
		// Minimum execution time: 28_400_000 picoseconds.
		Weight::from_parts(29_259_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: ChildBounties ParentChildBounties (r:1 w:0)
	/// Proof: ChildBounties ParentChildBounties (max_values: None, max_size: Some(16), added: 2491, mode: MaxEncodedLen)
	fn award_bounty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `439`
		//  Estimated: `3642`
		// Minimum execution time: 20_071_000 picoseconds.
		Weight::from_parts(20_662_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: System Account (r:3 w:3)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: ChildBounties ChildrenCuratorFees (r:1 w:1)
	/// Proof: ChildBounties ChildrenCuratorFees (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: Bounties BountyDescriptions (r:0 w:1)
	/// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(16400), added: 18875, mode: MaxEncodedLen)
	fn claim_bounty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `803`
		//  Estimated: `8799`
		// Minimum execution time: 119_806_000 picoseconds.
		Weight::from_parts(122_217_000, 0)
			.saturating_add(Weight::from_parts(0, 8799))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: ChildBounties ParentChildBounties (r:1 w:0)
	/// Proof: ChildBounties ParentChildBounties (max_values: None, max_size: Some(16), added: 2491, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Bounties BountyDescriptions (r:0 w:1)
	/// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(16400), added: 18875, mode: MaxEncodedLen)
	fn close_bounty_proposed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `483`
		//  Estimated: `3642`
		// Minimum execution time: 48_528_000 picoseconds.
		Weight::from_parts(49_592_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: ChildBounties ParentChildBounties (r:1 w:0)
	/// Proof: ChildBounties ParentChildBounties (max_values: None, max_size: Some(16), added: 2491, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Bounties BountyDescriptions (r:0 w:1)
	/// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(16400), added: 18875, mode: MaxEncodedLen)
	fn close_bounty_active() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `719`
		//  Estimated: `6196`
		// Minimum execution time: 79_963_000 picoseconds.
		Weight::from_parts(81_894_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Bounties Bounties (r:1 w:1)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	fn extend_bounty_expiry() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `325`
		//  Estimated: `3642`
		// Minimum execution time: 15_794_000 picoseconds.
		Weight::from_parts(16_237_000, 0)
			.saturating_add(Weight::from_parts(0, 3642))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Bounties BountyApprovals (r:1 w:1)
	/// Proof: Bounties BountyApprovals (max_values: Some(1), max_size: Some(402), added: 897, mode: MaxEncodedLen)
	/// Storage: Bounties Bounties (r:100 w:100)
	/// Proof: Bounties Bounties (max_values: None, max_size: Some(177), added: 2652, mode: MaxEncodedLen)
	/// Storage: System Account (r:200 w:200)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `b` is `[0, 100]`.
	fn spend_funds(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + b * (297 ±0)`
		//  Estimated: `1887 + b * (5206 ±0)`
		// Minimum execution time: 5_312_000 picoseconds.
		Weight::from_parts(5_480_000, 0)
			.saturating_add(Weight::from_parts(0, 1887))
			// Standard Error: 12_652
			.saturating_add(Weight::from_parts(45_246_882, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
			.saturating_add(Weight::from_parts(0, 5206).saturating_mul(b.into()))
	}
}
