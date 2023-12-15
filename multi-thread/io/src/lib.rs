#![no_std]
use gstd::{ prelude::*, ActorId};
use gmeta::{In, InOut, Metadata};

extern crate alloc;

#[derive(Encode, Decode, TypeInfo)]
pub struct InitThread {
    pub id: String,
    pub thread_type: ThreadType,
    pub title: String,
    pub content: String,
    pub photo_url: String
}

#[derive(Encode, Decode, TypeInfo, PartialEq, Eq, Clone, Debug)]
pub struct ThreadReply {
    pub id: String,
    pub owner: ActorId,
    pub title: String,
    pub content: String,
    pub likes: u128,
    pub reports: u128,
}

#[derive(Default, Encode, Decode, Clone, TypeInfo)]
pub struct Thread {
    pub id: String,
    pub owner: ActorId,
    pub thread_type: ThreadType,
    pub title: String,
    pub content: String,
    pub photo_url: String,
    pub replies: Vec<(String, ThreadReply)>,
    pub participants:Vec<(ActorId, u128)>,
    pub thread_status: ThreadState,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(String, Vec<String>)>
}

#[derive( Encode, Decode, Clone, TypeInfo)]
#[derive(Default)]
pub enum ThreadType {
    #[default]
    Challenge,
    Question
}

#[derive( Encode, Decode, Clone, TypeInfo)]
#[derive(Default)]
pub enum ThreadState {
    #[default]
    Active,
    Expired
}

#[derive(Encode, Decode, TypeInfo)]
pub enum ThreadAction {
    NewThread(InitThread),
    EndThread,
    AddReply(String, String, String, String),
    LikeReply(u128, String)
}

#[derive(Encode, Decode, TypeInfo, PartialEq, Eq, Clone, Debug)]
pub enum ThreadEvent {
    NewThreadCreated,
    ThreadEnded,
    ReplyAdded,
    ReplyLiked
}

#[derive(Debug, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTAction {
    Mint(u128),
    Burn(u128),
    Transfer {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        to: ActorId,
        amount: u128,
    },
    TotalSupply,
    BalanceOf(ActorId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum FTEvent {
    Ok,
    Err,
    Balance(u128),
    PermitId(u128),
}

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFT {
    pub ft_program_id: ActorId,
}

#[derive(Default, Clone, Encode, Decode, TypeInfo)]
pub struct IoThread {
    pub id: String,
    pub owner: ActorId,
    pub thread_type: ThreadType,
    pub title: String,
    pub content: String,
    pub photo_url: String,
    pub replies: Vec<(String, ThreadReply)>,
    pub participants:Vec<(ActorId, u128)>,
    pub thread_status: ThreadState,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(String, Vec<String>)>
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = In<InitFT>;
    type Handle = InOut<ThreadAction,ThreadEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = IoThread;
}