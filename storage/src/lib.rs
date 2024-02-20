#![no_std]

use gstd::{msg, prelude::*};
use storage_io::{IoThreadStorage, StorageAction, StorageEvent, ThreadStorage};

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
            thread_storage.push_thread(thread.into());
            msg::reply(StorageEvent::ThreadPush(thread_id), 0)
                .expect("Failed to reply to PushThread Action");
        }
        StorageAction::PushReply(thread_id, reply) => {
            let reply_id = reply.post_data.post_id;
            thread_storage.push_reply(thread_id, reply.into());
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
    }
}

#[no_mangle]
extern fn state() {
    let thread_storage = unsafe {
        THREAD_STORAGE
            .take()
            .expect("Unexpected error in taking state")
    };
    msg::reply::<IoThreadStorage>(thread_storage.into(), 0).expect(
        "Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`",
    );
}
