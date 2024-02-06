#![no_std]

use gstd::{collections::HashMap as GHashMap, prelude::*, ActorId};

pub type PostId = String;
pub type Timestamp = u64;

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Post {
    pub post_id: PostId,
    pub posted_at: Timestamp,
    pub owner: ActorId,
    pub title: String,
    pub content: String,
    pub photo_url: String,
}

pub struct Thread {
    pub post_data: Post,
    pub thread_status: ThreadStatus,
    pub distributed_tokens: u64,
    pub graph_rep: GHashMap<PostId, Vec<PostId>>,
    pub replies: GHashMap<PostId, ThreadReply>,
}

pub struct ThreadReply {
    pub post_data: Post,
    pub reports: u64,
    pub like_history: GHashMap<ActorId, u64>,
    pub likes: u64,
}

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ThreadType {
    #[default]
    Challenge,
    Question,
}

#[derive(Default, Encode, Decode, TypeInfo)]
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

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoThread {
    pub post_data: Post,
    pub replies: Vec<(PostId, IoThreadReply)>,
    pub thread_status: ThreadStatus,
    pub distributed_tokens: u64,
    pub graph_rep: Vec<(PostId, Vec<PostId>)>,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoThreadReply {
    pub post_data: Post,
    pub likes: u64,
    pub reports: u64,
    pub like_history: Vec<(ActorId, u64)>,
}

impl From<IoThreadReply> for ThreadReply {
    fn from(io_reply: IoThreadReply) -> Self {
        let like_history: GHashMap<ActorId, u64> = io_reply
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
        let like_history: Vec<(ActorId, u64)> = thread_reply
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
            distributed_tokens: thread.distributed_tokens,
            graph_rep,
        }
    }
}
