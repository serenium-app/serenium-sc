#![no_std]

use gstd::{exec, msg, prelude::*, ActorId};

pub type PostId = u32;
pub type Timestamp = u64;
pub type URL = String;

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Post {
    pub post_id: PostId,
    pub posted_at: Timestamp,
    pub owner: ActorId,
    pub title: String,
    pub content: String,
    pub photo_url: Option<URL>,
}

impl Post {
    pub fn new(title: String, content: String, photo_url: String) -> Self {
        Post {
            post_id: exec::block_height(),
            posted_at: exec::block_timestamp(),
            owner: msg::source(),
            title,
            content,
            photo_url: if photo_url.is_empty() {
                None
            } else {
                Some(photo_url)
            },
        }
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitThread {
    pub title: String,
    pub content: String,
    pub photo_url: String,
    pub thread_type: ThreadType,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitReply {
    pub title: String,
    pub content: String,
    pub photo_url: String,
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadType {
    Challenge,
    Question,
}

#[derive(Default, Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadStatus {
    #[default]
    Active,
    Expired,
}

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFT {
    pub ft_program_id: ActorId,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Threads {
    pub threads: Vec<(PostId, Thread)>,
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Thread {
    pub post_data: Post,
    pub replies: Vec<(PostId, ThreadReply)>,
    pub thread_status: ThreadStatus,
    pub thread_type: ThreadType,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(PostId, Vec<PostId>)>,
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct ThreadReply {
    pub post_data: Post,
    pub likes: u128,
    pub reports: u64,
    pub like_history: Vec<(ActorId, u128)>,
}
