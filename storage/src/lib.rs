#![no_std]

use gstd::{msg, prelude::*, ActorId};
use io::{Post, PostId};
use storage_io::{
    QueryThread, StorageAction, StorageEvent, StorageQuery, StorageQueryReply, ThreadStorage,
};

static mut THREAD_STORAGE: Option<ThreadStorage> = None;

fn thread_storage_mut() -> &'static mut ThreadStorage {
    unsafe { THREAD_STORAGE.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let thread_storage = ThreadStorage::new();

    unsafe { THREAD_STORAGE = Some(thread_storage) }

    thread_storage_mut().admin = Some(msg::source());
}

#[no_mangle]
extern fn handle() {
    let action: StorageAction = msg::load().expect("Could not load Action");
    let thread_storage = thread_storage_mut();

    match action {
        StorageAction::AddLogicContractAddress(address) => {
            if thread_storage.admin.expect("") != msg::source() {
                panic!("AddLogicContractAddress action can only be called by admin")
            }
            thread_storage.add_logic_contract_address(address);
            msg::reply(StorageEvent::LogicContractAddressAdded, 0)
                .expect("Failed to reply to AddLogicContractAddress Action");
        }
        StorageAction::PushThread(thread) => {
            let thread_id = thread.post_data.post_id;
            thread_storage.push_thread(thread);
            msg::reply(StorageEvent::ThreadPush(thread_id), 0)
                .expect("Failed to reply to PushThread Action");
        }
        StorageAction::PushReply(thread_id, reply, ref_node) => {
            let reply_id = reply.post_data.post_id;
            thread_storage.push_reply(thread_id, reply, ref_node);
            msg::reply(StorageEvent::ReplyPush(reply_id), 0)
                .expect("Failed to reply to PushReply Action");
        }
        StorageAction::LikeReply(thread_id, reply_id, like_count) => {
            thread_storage.like_reply(thread_id, reply_id, like_count);
            msg::reply(StorageEvent::ReplyLiked, 0).expect("Failed to reply to LikeReply Action");
        }
        StorageAction::ChangeStatusState(thread_id) => {
            thread_storage.change_status_thread(thread_id);
            msg::reply(StorageEvent::StatusStateChanged, 0)
                .expect("Failed to reply to ChangeStatusState Action");
        }
        StorageAction::RemoveThread(post_id) => {
            thread_storage.remove_thread(post_id);
            msg::reply(StorageEvent::ThreadRemoved, 0)
                .expect("Failed to reply to RemoveThread Action");
        }
        StorageAction::RemoveReply(thread_id, reply_id) => {
            thread_storage.remove_reply(thread_id, reply_id);
            msg::reply(StorageEvent::ReplyRemoved, 0)
                .expect("Failed to reply to RemoveReply Action");
        }
    }
}

#[no_mangle]
extern fn state() {
    let thread_storage = unsafe {
        THREAD_STORAGE
            .take()
            .expect("Unexpected error in taking state")
    };
    let query: StorageQuery = msg::load().expect("Unable to decode query");
    let reply = match query {
        StorageQuery::AllRepliesWithLikes(thread_id) => {
            let reduced_replies: Vec<(PostId, ActorId, u128)> = thread_storage
                .threads
                .get(&thread_id)
                .map(|thread| {
                    thread
                        .replies
                        .iter()
                        .map(|(post_id, reply)| (*post_id, reply.post_data.owner, reply.likes))
                        .collect::<Vec<_>>()
                })
                .expect("thread not found");

            StorageQueryReply::AllRepliesWithLikes(reduced_replies)
        }
        StorageQuery::GraphRep(thread_id) => {
            let graph_rep = thread_storage
                .threads
                .get(&thread_id)
                .map(|thread| &thread.graph_rep)
                .expect("thread not found");

            StorageQueryReply::GraphRep(graph_rep.clone())
        }
        StorageQuery::LikeHistoryOf(thread_id, reply_id) => {
            let like_history = thread_storage
                .threads
                .get(&thread_id)
                .and_then(|thread| thread.replies.iter().find(|(id, _)| *id == reply_id))
                .map(|(_, reply)| &reply.like_history);
            StorageQueryReply::LikeHistoryOf(like_history.unwrap().clone())
        }
        StorageQuery::AllThreadsFE => {
            let threads_fe: Vec<(QueryThread, Option<Post>)> = thread_storage
                .threads
                .iter()
                .map(|(post_id, thread)| {
                    let featured_reply_fe = thread_storage
                        .get_featured_reply(*post_id)
                        .map(|reply| reply.post_data.clone());

                    let query_thread: QueryThread = QueryThread {
                        post_data: thread.post_data.clone(),
                        thread_type: thread.thread_type.clone(),
                        thread_status: thread.thread_status.clone(),
                    };

                    (query_thread, featured_reply_fe)
                })
                .collect();

            StorageQueryReply::AllThreadsFE(threads_fe)
        }
        StorageQuery::AllRepliesFE(thread_id) => {
            let thread_fe: Post = thread_storage
                .threads
                .get(&thread_id)
                .expect("")
                .post_data
                .clone();

            let replies_fe: Vec<Post> = thread_storage
                .threads
                .get(&thread_id)
                .expect("")
                .replies
                .iter()
                .map(|(_post_id, thread_reply)| thread_reply.post_data.clone())
                .collect();

            StorageQueryReply::AllRepliesFE(thread_fe, replies_fe)
        }
        StorageQuery::DistributedTokens(thread_id) => {
            let distributed_tokens: u128 = thread_storage
                .threads
                .get(&thread_id)
                .expect("")
                .distributed_tokens;

            StorageQueryReply::DistributedTokens(distributed_tokens)
        }
    };
    msg::reply(reply, 0).expect("Error in sharing state");
}
