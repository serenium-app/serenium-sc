#![no_std]

use gstd::{exec, msg, prelude::*, ActorId};

pub type PostId = u32;
pub type Timestamp = u64;
pub type URL = String;

pub type ThreadNode = (PostId, ActorId);

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
    pub graph_rep: ThreadGraph,
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

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct ThreadGraph {
    pub graph: Vec<(ThreadNode, Vec<ThreadNode>)>,
}

impl ThreadGraph {
    pub fn new() -> Self {
        ThreadGraph { graph: Vec::new() }
    }

    pub fn add_edge(&mut self, from_post_id: PostId, to: ThreadNode) {
        // Find the node corresponding to the from_post_id
        for (node, adj_list) in &mut self.graph {
            if node.0 == from_post_id {
                // If the node exists, append the new node to its adjacency list
                adj_list.push(to);
                return;
            }
        }
    }

    pub fn add_node(&mut self, node: ThreadNode) {
        // Check if the node already exists in the graph
        for (existing_node, _) in &self.graph {
            if *existing_node == node {
                // If it already exists, do nothing
                return;
            }
        }
        // If the node does not exist, add it with an empty adjacency list
        self.graph.push((node, Vec::new()));
    }
}

impl Default for ThreadGraph {
    fn default() -> Self {
        Self::new()
    }
}
