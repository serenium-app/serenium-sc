#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{msg, prelude::*, ActorId};
use io::{InitReply, InitThread, Post, PostId, Thread, ThreadReply};
use reward_logic_io::{RewardLogicAction, RewardLogicEvent};
use sharded_fungible_token_io::{FTokenEvent, LogicAction};
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

    pub async fn mint_tokens(&mut self, amount: u128) -> Result<(), ()> {
        let res = msg::send_for_reply_as::<_, FTokenEvent>(
            self.address_ft.expect(""),
            LogicAction::Mint {
                recipient: self.address_storage.unwrap(),
                amount,
            },
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                FTokenEvent::Ok => Ok(()),
                _ => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub async fn transfer_tokens(
        &mut self,
        ft_address_id: ActorId,
        amount: u128,
        sender: ActorId,
        recipient: ActorId,
    ) -> Result<(), ()> {
        let res = msg::send_for_reply_as::<_, FTokenEvent>(
            ft_address_id,
            LogicAction::Transfer {
                sender,
                recipient,
                amount,
            },
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                FTokenEvent::Ok => Ok(()),
                _ => Err(()),
            },
            Err(_) => Err(()),
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

        self.mint_tokens(1).await.expect("");

        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::PushThread(thread),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageEvent::ThreadPush(_) => {
                    msg::reply(ThreadLogicEvent::NewThreadCreated, 0).expect("")
                }
                _ => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
            },
            Err(_) => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
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

        self.transfer_tokens(
            self.address_ft.unwrap(),
            1,
            msg::source(),
            self.address_storage.unwrap(),
        )
        .await
        .expect("");

        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::PushReply(reply.post_data.post_id, reply),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageEvent::ReplyPush(_) => {
                    msg::reply(ThreadLogicEvent::ReplyAdded, 0).expect("")
                }
                _ => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
            },
            Err(_) => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
        };
    }

    pub async fn like_reply(&mut self, thread_id: PostId, reply_id: PostId, like_count: u128) {
        self.transfer_tokens(
            self.address_ft.unwrap(),
            like_count,
            msg::source(),
            self.address_storage.unwrap(),
        )
        .await
        .expect("");

        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::LikeReply(thread_id, reply_id, like_count),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageEvent::ReplyLiked => msg::reply(ThreadLogicEvent::ReplyLiked, 0).expect(""),
                _ => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
            },
            Err(_) => msg::reply(ThreadLogicEvent::LogicError, 0).expect(""),
        };
    }

    pub async fn send_trigger_reward_msg(&mut self, thread_id: PostId) -> Result<(), ()> {
        let reward_res = msg::send_for_reply_as::<_, RewardLogicEvent>(
            self.address_reward_logic.expect(""),
            RewardLogicAction::TriggerRewardLogic(thread_id),
            0,
            0,
        )
        .expect("")
        .await;

        match reward_res {
            Ok(event) => match event {
                RewardLogicEvent::RewardLogicTriggered => Ok(()),
                _ => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub async fn send_thread_status_expired_msg(&mut self, thread_id: PostId) -> Result<(), ()> {
        let res = msg::send_for_reply_as::<_, StorageEvent>(
            self.address_storage.expect(""),
            StorageAction::ChangeStatusState(thread_id),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageEvent::StatusStateChanged => Ok(()),
                _ => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub async fn expire_thread(&mut self, thread_id: PostId) {
        // self.send_trigger_reward_msg(thread_id).await.unwrap();

        // Only when reward logic has been successful, change state
        // TODO: Send msg to storage contract to change state
        self.send_thread_status_expired_msg(thread_id)
            .await
            .unwrap();
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
    AddReply(InitReply),
    LikeReply(PostId, PostId, u128),
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
    ReplyAdded,
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
