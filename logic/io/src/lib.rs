#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{msg, prelude::*, ActorId};
use io::{InitReply, InitThread, Post, PostId, Thread, ThreadReply};
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
            self.address_storage.expect(""),
            StorageAction::PushThread(thread.into()),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(StorageEvent::ThreadPush(_post_id)) => {
                msg::reply(ThreadLogicEvent::NewThreadCreated, 0).expect("")
            }
            Ok(StorageEvent::StorageError)
            | Ok(StorageEvent::LogicContractAddressAdded)
            | Ok(StorageEvent::ReplyPush(_))
            | Ok(StorageEvent::ReplyLiked)
            | Ok(StorageEvent::StatusStateChanged)
            | Err(_) => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
        };
    }

    pub async fn add_reply(&mut self, init_reply: InitReply) {
        let post = Post::new(init_reply.title, init_reply.content, init_reply.photo_url);

        let reply = ThreadReply {
            post_data: post,
            reports: 0,
            like_history: Default::default(),
            likes: 0,
        };

        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::PushReply(reply.post_data.post_id, reply.into()),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(StorageEvent::ReplyPush(post_id)) => msg::reply(
                ThreadLogicEvent::ReplyAdded {
                    by: msg::source(),
                    id: post_id,
                    on_thread: 0,
                },
                0,
            )
            .expect(""),
            Ok(StorageEvent::StorageError)
            | Err(_)
            | Ok(StorageEvent::ReplyLiked)
            | Ok(StorageEvent::LogicContractAddressAdded)
            | Ok(StorageEvent::ThreadPush(_))
            | Ok(StorageEvent::StatusStateChanged) => {
                msg::reply(ThreadLogicEvent::LogicError, 0).expect("")
            }
        };
    }

    pub async fn like_reply(&mut self, thread_id: PostId, reply_id: PostId, like_count: u64) {
        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::LikeReply(thread_id, reply_id, like_count),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(StorageEvent::ReplyLiked) => msg::reply(ThreadLogicEvent::ReplyLiked, 0).expect(""),
            Ok(StorageEvent::StorageError)
            | Ok(StorageEvent::LogicContractAddressAdded)
            | Ok(StorageEvent::ReplyPush(_))
            | Ok(StorageEvent::StatusStateChanged)
            | Ok(StorageEvent::ThreadPush(_))
            | Err(_) => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
        };
    }

    pub async fn expire_thread(&mut self, thread_id: PostId) {
        // TODO: Send msg to reward logic contract to trigger reward calculations and FT transfers

        // Only when reward logic has been successful, change state
        // TODO: Send msg to storage contract to change state
        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::ChangeStatusState(thread_id),
            0,
            0,
        )
        .expect("")
        .await;
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
    AddReply(InitReply),
    LikeReply(PostId, PostId, u64),
    ExpireThread(PostId),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadLogicEvent {
    FTAddressAdded,
    StorageAddressAdded,
    RewardLogicAddressAdded,
    NewThreadCreated,
    ReplyAdded {
        by: ActorId,
        id: PostId,
        on_thread: PostId,
    },
    ReplyLiked,
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
