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

    pub fn distributed_tokens(state: State, target_post_id: PostId) -> Option<u64> {
        if let Some((_, io_thread)) = state
            .threads
            .iter()
            .find(|(post_id, _)| *post_id == target_post_id)
        {
            Some(io_thread.distributed_tokens)
        } else {
            None
        }
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

    pub fn graph_rep(state: State, target_post_id: PostId) -> Option<Vec<(PostId, Vec<PostId>)>> {
        if let Some((_, io_thread)) = state
            .threads
            .iter()
            .find(|(post_id, _)| *post_id == target_post_id)
        {
            Some(io_thread.graph_rep.clone())
        } else {
            None
        }
    }

    /// Retrieves the like history associated with a target reply within a target thread in the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state containing threads and replies.
    /// * `target_post_id` - The ID of the target post (thread) where the reply is located.
    /// * `target_reply_id` - The ID of the target reply for which the like history is sought.
    ///
    /// # Returns
    ///
    /// * `Some(Vec<(ActorId, u64)>)` - If the target post and reply are found, returns the like history of the reply.
    /// * `None` - If either the target post or reply is not found.
    ///
    pub fn like_history(
        state: State,
        target_post_id: PostId,
        target_reply_id: PostId,
    ) -> Option<Vec<(ActorId, u64)>> {
        state
            .threads
            .iter()
            .find(|(thread_id, _)| *thread_id == target_post_id)
            .and_then(|(_, io_thread)| {
                io_thread
                    .replies
                    .iter()
                    .find(|(reply_id, _)| *reply_id == target_reply_id)
            })
            .map(|(_, io_reply)| io_reply.like_history.clone())
    }

    pub fn thread_by_post_id(state: State, target_post_id: PostId) -> Option<IoThread> {
        state
            .threads
            .iter()
            .find(|(thread_id, _)| *thread_id == target_post_id)
            .map(|(_, io_thread)| io_thread.clone())
    }
}
