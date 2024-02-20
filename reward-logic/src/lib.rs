#![no_std]

use gstd::{async_main, msg};
use reward_logic_io::{RewardLogic, RewardLogicAction};

static mut REWARD_LOGIC: Option<RewardLogic> = None;

fn reward_logic_mut() -> &'static mut RewardLogic {
    unsafe { REWARD_LOGIC.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let reward_logic = RewardLogic::new();

    unsafe { REWARD_LOGIC = Some(reward_logic) }
}

#[async_main]
async fn main() {
    let action: RewardLogicAction = msg::load().expect("");
    let _reward_logic = reward_logic_mut();
    match action {
        RewardLogicAction::TriggerRewardLogic(_thread) => {}
    }
}

#[no_mangle]
extern fn state() {
    let thread_logic = unsafe {
        REWARD_LOGIC
            .take()
            .expect("Unexpected error in taking state")
    };
    msg::reply::<RewardLogic>(thread_logic, 0).expect(
        "Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`",
    );
}
