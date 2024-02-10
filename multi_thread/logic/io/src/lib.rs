#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{msg, prelude::*, ActorId};
use io::{InitThread, Post, PostId, Thread};
use storage_io::{StorageAction, StorageEvent};

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct ThreadLogic {
    pub admin: Option<ActorId>,
    pub address_ft: Option<ActorId>,
    pub address_storage: Option<ActorId>,
    pub address_reward_logic: Option<ActorId>,
}

impl ThreadLogic {
    pub fn new() -> Self {
        ThreadLogic {
            admin: None,
            address_ft: None,
            address_storage: None,
            address_reward_logic: None,
        }
    }

    pub async fn new_thread(&mut self, init_thread: InitThread) {
        let post = Post::new(
            init_thread.title,
            init_thread.content,
            init_thread.photo_url,
        );
        let thread = Thread {
            post_data: post,
            thread_status: Default::default(),
            thread_type: init_thread.thread_type,
            distributed_tokens: 0,
            graph_rep: Default::default(),
            replies: Default::default(),
        };
        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.unwrap(),
            StorageAction::PushThread(thread.into()),
            0,
            0,
        )
        .expect("")
        .await;
        match res {
            Ok(StorageEvent::ThreadPush(post_id)) => msg::reply(
                ThreadLogicEvent::NewThreadCreated {
                    by: msg::source(),
                    id: post_id,
                },
                0,
            )
            .expect(""),
            Ok(StorageEvent::StorageError) | Err(_) | _ => {
                msg::reply(ThreadLogicEvent::LogicError, 0).expect("")
            }
        };
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadLogicAction {
    AddAddressFT(ActorId),
    AddAddressStorage(ActorId),
    AddAddressRewardLogic(ActorId),
    NewThread(InitThread),
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
    LogicError,
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
