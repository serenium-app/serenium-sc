#![no_std]

use gstd::{msg, prelude::*};
use storage_io::{IoThreadStorage, StorageAction, ThreadStorage};

static mut THREAD_STORAGE: Option<ThreadStorage> = None;

fn thread_storage_mut() -> &'static mut ThreadStorage {
    unsafe { THREAD_STORAGE.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let thread_storage = ThreadStorage::new();

    unsafe { THREAD_STORAGE = Some(thread_storage) }
}

#[no_mangle]
extern fn handle() {
    let action: StorageAction = msg::load().expect("Could not load Action");
    let thread_storage = thread_storage_mut();

    match action {
        StorageAction::AddLogicContractAddress(address) => {
            thread_storage.add_logic_contract_address(address)
        }
        StorageAction::PushThread(thread) => thread_storage.push_thread(thread.into()),
        StorageAction::PushReply(post_id, reply) => {
            thread_storage.push_reply(post_id, reply.into())
        }
        StorageAction::LikeReply(thread_id, reply_id, like_count) => {
            thread_storage.like_reply(thread_id, reply_id, like_count)
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
