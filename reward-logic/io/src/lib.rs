#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{collections::HashMap as GHashMap, prelude::*, ActorId};
use io::{IoThread, PostId, Thread, ThreadReply};

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct RewardLogic {
    pub address_ft_contract: Option<ActorId>,
    pub address_logic_contract: Option<ActorId>,
}

impl RewardLogic {
    pub fn new() -> Self {
        RewardLogic {
            address_ft_contract: None,
            address_logic_contract: None,
        }
    }
    pub fn transfer_tokens_reward(_amount_tokens: u64, _dest: ActorId) {}
    pub fn trigger_reward_logic(thread: Thread) {
        let mut reward_logic_thread = RewardLogicThread::new(thread);
        reward_logic_thread.set_expired_thread_data();

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .winner_reply = reward_logic_thread.find_winner_reply();

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .top_liker_winner = reward_logic_thread.find_top_liker_winner();
    }
}

pub struct RewardLogicThread {
    pub distributed_tokens: u64,
    pub graph_rep: GHashMap<PostId, Vec<PostId>>,
    pub replies: GHashMap<PostId, ThreadReply>,
    pub expired_thread_data: Option<ExpiredThread>,
}

impl RewardLogicThread {
    pub fn new(thread: Thread) -> Self {
        RewardLogicThread {
            distributed_tokens: thread.distributed_tokens,
            graph_rep: thread.graph_rep,
            replies: thread.replies,
            expired_thread_data: None,
        }
    }

    pub fn set_expired_thread_data(&mut self) {
        let expired_thread_data = ExpiredThread::new();
        self.expired_thread_data = Some(expired_thread_data);
    }
    pub fn find_winner_reply(&mut self) -> Option<PostId> {
        self.replies
            .values()
            .max_by_key(|thread_reply| thread_reply.likes)
            .map(|reply| reply.post_data.post_id)
    }

    /// Finds the top liker winner based on the `like_history` of the winner's reply.
    ///
    /// This method retrieves the winner reply from `expired_thread_data`, then finds the entry in
    /// `self.replies` corresponding to the winner reply. It then iterates over the `like_history`
    /// associated with the winner reply to find the key (ActorId) corresponding to the entry with
    /// the highest value. If such an entry exists, it returns `Some(ActorId)` representing the key
    /// of the top liker winner. If `expired_thread_data` is not present, or if the winner reply is
    /// not available, or if the `like_history` is empty, it returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(ActorId)`: The key corresponding to the entry with the highest value in the
    ///   `like_history` of the winner's reply, if found.
    /// - `None`: If `expired_thread_data` is not present, or if the winner reply is not available,
    ///   or if the `like_history` is empty.
    pub fn find_top_liker_winner(&mut self) -> Option<ActorId> {
        self.expired_thread_data
            .as_ref()
            .and_then(|expired_thread_data| {
                expired_thread_data.winner_reply.and_then(|winner_reply| {
                    self.replies
                        .get_mut(&winner_reply)
                        .expect("")
                        .like_history
                        .iter()
                        .max_by_key(|&(_, v)| *v)
                        .map(|(k, _)| k)
                        .cloned()
                })
            })
    }
    pub fn find_path_winners(&mut self) {}
}

pub struct ExpiredThread {
    pub top_liker_winner: Option<ActorId>,
    pub path_winners: Option<Vec<ActorId>>,
    pub transaction_log: Option<Vec<(ActorId, u64)>>,
    pub winner_reply: Option<PostId>,
}

impl ExpiredThread {
    pub fn new() -> Self {
        ExpiredThread {
            top_liker_winner: None,
            path_winners: None,
            transaction_log: None,
            winner_reply: None,
        }
    }
}

impl Default for ExpiredThread {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum RewardLogicAction {
    TriggerRewardLogic(IoThread),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum RewardLogicEvent {
    RewardLogicTriggered,
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = InOut<RewardLogicAction, RewardLogicEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<RewardLogic>;
}
