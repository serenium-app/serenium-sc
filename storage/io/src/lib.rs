#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{collections::HashMap as GHashMap, msg, prelude::*, ActorId};
use io::{Post, PostId, Thread, ThreadGraph, ThreadNode, ThreadReply, ThreadStatus, ThreadType};

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct QueryThread {
    pub post_data: Post,
    pub thread_type: ThreadType,
    pub thread_status: ThreadStatus,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct QueryReply {
    pub post_data: Post,
    pub thread_id: PostId,
}

#[derive(Default)]
pub struct ThreadStorage {
    pub threads: GHashMap<PostId, Thread>,
    pub admin: Option<ActorId>,
    pub address_logic_contract: Option<ActorId>,
}

impl ThreadStorage {
    pub fn new() -> Self {
        ThreadStorage {
            threads: GHashMap::new(),
            admin: None,
            address_logic_contract: None,
        }
    }

    pub fn push_thread(&mut self, mut thread: Thread) {
        // Create a new node on graph
        let new_node: ThreadNode = (thread.post_data.post_id, thread.post_data.owner);

        thread.graph_rep.add_node(new_node);

        self.threads.insert(thread.post_data.post_id, thread);
    }

    pub fn push_reply(&mut self, thread_id: PostId, reply: ThreadReply, ref_node: PostId) {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            // Push to graph_rep
            let new_node: ThreadNode = (reply.post_data.post_id, reply.post_data.owner);

            thread.graph_rep.add_edge(ref_node, new_node);

            thread.replies.push((reply.post_data.post_id, reply));
        }
    }

    pub fn like_reply(&mut self, thread_id: PostId, reply_id: PostId, like_count: u128) {
        // Retrieve the mutable reference to the thread by its `thread_id`
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            // Find the mutable reference to the `ThreadReply` tuple within the thread
            if let Some((_, reply)) = thread.replies.iter_mut().find(|(id, _)| *id == reply_id) {
                // Increment the reply's likes by the specified amount
                reply.likes += like_count;
            }
        }
    }

    pub fn change_status_thread(&mut self, thread_id: PostId) {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread.thread_status = ThreadStatus::Expired;
        }
    }

    pub fn add_logic_contract_address(&mut self, address: ActorId) {
        self.address_logic_contract = Some(address);
    }

    pub fn remove_thread(&mut self, post_id: PostId) {
        if msg::source() != self.admin.expect("Unable to retrieve admin ActorId") {
            panic!("Thread may only be removed by admin")
        }
        self.threads.remove(&post_id);
    }

    pub fn remove_reply(&mut self, thread_id: PostId, reply_id: PostId) {
        // Check if the caller is the admin
        let admin_id = self.admin.expect("Admin ActorId must be set");
        if msg::source() != admin_id {
            panic!("Reply may only be removed by admin");
        }

        // Attempt to retrieve the thread and remove the reply
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            if let Some(index) = thread.replies.iter().position(|(id, _)| *id == reply_id) {
                thread.replies.remove(index);
                // delete reply from graph
                thread.graph_rep.remove_node(reply_id);
            }
        }
    }

    pub fn get_featured_reply(&self, thread_id: PostId) -> Option<&ThreadReply> {
        self.threads.get(&thread_id).and_then(|thread| {
            if thread.replies.is_empty() {
                None
            } else {
                thread
                    .replies
                    .iter()
                    .min_by_key(|(_, reply)| reply.likes)
                    .map(|(_, reply)| reply)
            }
        })
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageAction {
    AddLogicContractAddress(ActorId),
    PushThread(Thread),
    PushReply(PostId, ThreadReply, PostId),
    LikeReply(PostId, PostId, u128),
    ChangeStatusState(PostId),
    RemoveThread(PostId),
    RemoveReply(PostId, PostId),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageEvent {
    LogicContractAddressAdded,
    StorageError,
    ThreadPush(PostId),
    ReplyPush(PostId),
    ReplyLiked,
    StatusStateChanged,
    ThreadRemoved,
    ReplyRemoved,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageQuery {
    // For winner (rule no. 1)
    AllRepliesWithLikes(PostId),
    // For path to the winner (rule no. 2)
    GraphRep(PostId),
    // For top liker of winner (rule no. 3)
    LikeHistoryOf(PostId, PostId),
    // Fetch all threads with the title, content, owner and a single reply
    AllThreadsFE,
    // Fetch all replies for a given thread in a post_data format
    AllRepliesFE(PostId),
    // Fetch the distributed tokens for a given thread
    DistributedTokens(PostId),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StorageQueryReply {
    // For winner (rule no. 1)
    AllRepliesWithLikes(Vec<(PostId, ActorId, u128)>),
    // For path to the winner (rule no. 2)
    GraphRep(ThreadGraph),
    // For top liker of winner (rule no. 3)
    LikeHistoryOf(Vec<(ActorId, u128)>),
    // Fetch all threads with the title, content, owner and a single reply
    AllThreadsFE(Vec<(QueryThread, Option<Post>)>),
    // Fetch all replies and the thread itself for a given thread in a post_data format
    AllRepliesFE(QueryThread, Vec<QueryReply>),
    DistributedTokens(u128),
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = InOut<StorageAction, StorageEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = InOut<StorageQuery, StorageQueryReply>;
}
