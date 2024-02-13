#![no_std]

use gstd::{async_main, msg};
use logic_io::{ThreadLogic, ThreadLogicAction, ThreadLogicEvent};

static mut THREAD_LOGIC: Option<ThreadLogic> = None;

fn thread_logic_mut() -> &'static mut ThreadLogic {
    unsafe { THREAD_LOGIC.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let thread_logic = ThreadLogic::new();

    unsafe { THREAD_LOGIC = Some(thread_logic) }

    thread_logic_mut().admin = Some(msg::source());
}

#[async_main]
async fn main() {
    let action: ThreadLogicAction = msg::load().expect("Could not load Action");
    let thread_logic = thread_logic_mut();
    match action {
        ThreadLogicAction::AddAddressFT(address) => {
            if thread_logic.admin.expect("") != msg::source() {
                panic!("Add Address FT Action can only be called by admin")
            }
            thread_logic.address_ft = Some(address);
            msg::reply(ThreadLogicEvent::FTAddressAdded, 0).expect("");
        }
        ThreadLogicAction::AddAddressStorage(address) => {
            if thread_logic.admin.expect("") != msg::source() {
                panic!("Add Address Storage Action can only be called by admin")
            }
            thread_logic.address_storage = Some(address);
            msg::reply(ThreadLogicEvent::StorageAddressAdded, 0).expect("");
        }
        ThreadLogicAction::AddAddressRewardLogic(address) => {
            if thread_logic.admin.expect("") != msg::source() {
                panic!("Add Address Reward Logic Action can only be called by admin")
            }
            thread_logic.address_reward_logic = Some(address);
            msg::reply(ThreadLogicEvent::RewardLogicAddressAdded, 0).expect("");
        }
        ThreadLogicAction::NewThread(init_thread) => thread_logic.new_thread(init_thread).await,
        ThreadLogicAction::AddReply(init_reply) => thread_logic.add_reply(init_reply).await,
        ThreadLogicAction::EndThread(_post_id) => {}
        ThreadLogicAction::LikeReply(thread_id, reply_id, like_count) => thread_logic.like_reply(thread_id, reply_id, like_count).await,
    }
}

#[no_mangle]
extern fn state() {
    let thread_logic = unsafe {
        THREAD_LOGIC
            .take()
            .expect("Unexpected error in taking state")
    };
    msg::reply::<ThreadLogic>(thread_logic, 0).expect(
        "Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`",
    );
}
