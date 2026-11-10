//! `suggest/core/dicnode`: the candidates of the search, their queues and cache. The profiler
//! (`dic_node_profiler`, debug builds only) is not ported.

pub mod cache;
pub mod node;
pub mod pool;
mod properties;
pub mod queue;
mod state_input;
mod state_output;
mod state_scoring;
pub mod utils;
pub mod vector;
