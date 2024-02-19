#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{prelude::*, ActorId};
use io::{Post, PostId};

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct ThreadLogic {
    pub address_ft: Option<ActorId>,
    pub address_storage: Option<ActorId>,
    pub address_reward_logic: Option<ActorId>,
}

impl ThreadLogic {
    pub fn new() -> Self {
        ThreadLogic {
            address_ft: None,
            address_storage: None,
            address_reward_logic: None,
        }
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadLogicAction {
    AddAddressFT(ActorId),
    AddAddressStorage(ActorId),
    AddAddressRewardLogic(ActorId),
    NewThread(Post),
    EndThread(PostId),
    AddReply(Post),
    LikeReply(PostId, u64),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadLogicEvent {
    FTAddressAdded {
        address: ActorId,
    },
    NewThreadCreated {
        by: ActorId,
        id: PostId,
    },
    ReplyAdded {
        by: ActorId,
        id: PostId,
        on_thread: PostId,
    },
    ReplyLiked {
        by: ActorId,
        like_count: u128,
        on_reply: PostId,
        on_thread: PostId,
    },
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = InOut<ThreadLogicAction, ThreadLogicEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<ThreadLogic>; // Logic is stateless, just to save addresses of related contracts
}
