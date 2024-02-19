#![no_std]

use gstd::{async_main, msg};
use logic_io::{ThreadLogic, ThreadLogicAction};

static mut THREAD_LOGIC: Option<ThreadLogic> = None;

fn thread_logic_mut() -> &'static mut ThreadLogic {
    unsafe { THREAD_LOGIC.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let thread_logic = ThreadLogic::new();

    unsafe { THREAD_LOGIC = Some(thread_logic) }
}

#[async_main]
async fn main() {
    let action: ThreadLogicAction = msg::load().expect("Could not load Action");
    let thread_logic = thread_logic_mut();
    match action {
        ThreadLogicAction::AddAddressFT(address) => {
            thread_logic.address_ft = Some(address);
        }
        ThreadLogicAction::AddAddressStorage(address) => {
            thread_logic.address_storage = Some(address);
        }
        ThreadLogicAction::AddAddressRewardLogic(address) => {
            thread_logic.address_reward_logic = Some(address);
        }
        ThreadLogicAction::NewThread(_post_data) => {}
        ThreadLogicAction::AddReply(_post_data) => {}
        ThreadLogicAction::EndThread(_post_id) => {}
        ThreadLogicAction::LikeReply(_post_id, _like_count) => {}
    }
}
