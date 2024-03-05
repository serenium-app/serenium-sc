#![no_std]

use gmeta::metawasm;
use gstd::{prelude::*, ActorId};
use io::*;
use storage_io::*;

#[metawasm]
pub mod metafns {
    pub type State = IoThreadStorage;

    pub fn reward_data(_state: State) -> Option<()> {
        None
    }

    pub fn distributed_tokens(_state: State) -> Option<u64> {
        None
    }

    pub fn thread_status(state: State, target_post_id: PostId) -> Option<ThreadStatus> {
        if let Some((_, io_thread)) = state
            .threads
            .iter()
            .find(|(post_id, _)| *post_id == target_post_id)
        {
            Some(io_thread.thread_status.clone())
        } else {
            None
        }
    }

    pub fn all_threads(_state: State) -> Option<IoThreads> {
        None
    }

    pub fn graph_rep(_state: State) -> Option<Vec<(PostId, Vec<PostId>)>> {
        None
    }

    pub fn like_history(_state: State) -> Option<Vec<(ActorId, u64)>> {
        None
    }

    pub fn thread_by_post_id(_state: State, _target_post_id: PostId) -> Option<IoThread> {
        None
    }
}
