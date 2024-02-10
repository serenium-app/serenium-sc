#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{collections::HashMap as GHashMap, prelude::*, ActorId};
use io::{IoThread, IoThreadReply, PostId, Thread, ThreadReply};

#[derive(Default)]
pub struct ThreadStorage {
    pub threads: GHashMap<PostId, Thread>,
    pub address_logic_contract: Option<ActorId>,
}

impl ThreadStorage {
    pub fn new() -> Self {
        ThreadStorage {
            threads: GHashMap::new(),
            address_logic_contract: None,
        }
    }

    pub fn push_thread(&mut self, thread: Thread) {
        self.threads
            .insert(thread.post_data.post_id.clone(), thread);
    }

    pub fn push_reply(&mut self, thread_id: PostId, reply: ThreadReply) {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread
                .replies
                .insert(reply.post_data.post_id.clone(), reply);
        }
    }

    pub fn like_reply(&mut self, thread_id: PostId, reply_id: PostId, like_count: u64) {
        self.threads
            .get_mut(&thread_id)
            .and_then(|thread| thread.replies.get_mut(&reply_id))
            .map(|reply| reply.likes += like_count);
    }

    pub fn add_logic_contract_address(&mut self, address: ActorId) {
        self.address_logic_contract = Some(address);
    }
}

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoThreadStorage {
    pub threads: Vec<(PostId, IoThread)>,
    pub address_logic_contract: Option<ActorId>,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageAction {
    AddLogicContractAddress(ActorId),
    PushThread(IoThread),
    PushReply(PostId, IoThreadReply),
    LikeReply(PostId, PostId, u64),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageEvent {
    LogicContractAddressAdded,
    StorageError,
    ThreadPush(PostId),
    ReplyPush(PostId),
    ReplyLiked,
}

impl From<ThreadStorage> for IoThreadStorage {
    fn from(thread_storage: ThreadStorage) -> Self {
        let threads: Vec<(PostId, IoThread)> = thread_storage
            .threads
            .into_iter()
            .map(|(post_id, thread)| (post_id, thread.into()))
            .collect();

        IoThreadStorage {
            threads,
            address_logic_contract: thread_storage.address_logic_contract,
        }
    }
}
pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = In<ActorId>;
    type Handle = InOut<StorageAction, StorageEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<IoThreadStorage>;
}
