#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{collections::HashMap as GHashMap, prelude::*, ActorId};
use io::{IoThread, PostId, Thread};

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
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageEvent {
    LogicContractAddressAdded,
    StorageError,
    ThreadPush(PostId),
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
