#![no_std]

use gstd::{collections::HashMap as GHashMap, exec, msg, prelude::*, ActorId};

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

pub struct Thread {
    pub post_data: Post,
    pub thread_status: ThreadStatus,
    pub thread_type: ThreadType,
    pub distributed_tokens: u128,
    pub graph_rep: GHashMap<PostId, Vec<PostId>>,
    pub replies: GHashMap<PostId, ThreadReply>,
}

pub struct ThreadReply {
    pub post_data: Post,
    pub reports: u64,
    pub like_history: GHashMap<ActorId, u128>,
    pub likes: u128,
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
pub struct IoThreads {
    pub threads: Vec<(PostId, IoThread)>,
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoThread {
    pub post_data: Post,
    pub replies: Vec<(PostId, IoThreadReply)>,
    pub thread_status: ThreadStatus,
    pub thread_type: ThreadType,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(PostId, Vec<PostId>)>,
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoThreadReply {
    pub post_data: Post,
    pub likes: u128,
    pub reports: u64,
    pub like_history: Vec<(ActorId, u128)>,
}

impl From<IoThreadReply> for ThreadReply {
    fn from(io_reply: IoThreadReply) -> Self {
        let like_history: GHashMap<ActorId, u128> = io_reply
            .like_history
            .into_iter()
            .map(|(actor_id, likes)| (actor_id, likes))
            .collect();

        ThreadReply {
            post_data: io_reply.post_data,
            reports: io_reply.reports,
            like_history,
            likes: io_reply.likes,
        }
    }
}

impl From<ThreadReply> for IoThreadReply {
    fn from(thread_reply: ThreadReply) -> Self {
        let like_history: Vec<(ActorId, u128)> = thread_reply
            .like_history
            .into_iter()
            .map(|(actor_id, likes)| (actor_id, likes))
            .collect();

        IoThreadReply {
            post_data: thread_reply.post_data,
            likes: thread_reply.likes,
            reports: thread_reply.reports,
            like_history,
        }
    }
}

impl From<IoThread> for Thread {
    fn from(io_thread: IoThread) -> Self {
        let graph_rep: collections::HashMap<PostId, Vec<PostId>> =
            io_thread.graph_rep.into_iter().collect();
        let replies: collections::HashMap<PostId, ThreadReply> = io_thread
            .replies
            .into_iter()
            .map(|(id, reply)| (id, reply.into()))
            .collect();

        Thread {
            post_data: io_thread.post_data,
            thread_status: io_thread.thread_status,
            thread_type: io_thread.thread_type,
            distributed_tokens: io_thread.distributed_tokens,
            graph_rep,
            replies,
        }
    }
}

impl From<Thread> for IoThread {
    fn from(thread: Thread) -> Self {
        let graph_rep: Vec<(PostId, Vec<PostId>)> = thread
            .graph_rep
            .into_iter()
            .map(|(post_id, post_ids)| (post_id, post_ids))
            .collect();

        let replies: Vec<(PostId, IoThreadReply)> = thread
            .replies
            .into_iter()
            .map(|(post_id, reply)| (post_id, reply.into()))
            .collect();

        IoThread {
            post_data: thread.post_data,
            replies,
            thread_status: thread.thread_status,
            thread_type: thread.thread_type,
            distributed_tokens: thread.distributed_tokens,
            graph_rep,
        }
    }
}
