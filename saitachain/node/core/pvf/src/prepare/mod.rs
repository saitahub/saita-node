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

//! Preparation part of pipeline
//!
//! The validation host spins up two processes: the queue (by running [`start_queue`]) and the pool
//! (by running [`start_pool`]).
//!
//! The pool will spawn workers in new processes and those should execute pass control to
//! `saitama_node_core_pvf_worker::prepare_worker_entrypoint`.

mod pool;
mod queue;
mod worker_intf;

pub use pool::start as start_pool;
pub use queue::{start as start_queue, FromQueue, ToQueue};
