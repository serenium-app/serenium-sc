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
            .expect("")
            .winner_reply = reward_logic_thread.find_winner_reply();
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
        if let Some(reply) = self
            .replies
            .values()
            .max_by_key(|thread_reply| thread_reply.likes)
        {
            Some(reply.post_data.post_id)
        } else {
            None
        }
    }
    pub fn find_top_liker_winner(&mut self) {}
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
