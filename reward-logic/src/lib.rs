#![no_std]

use gstd::{async_main, msg};
use reward_logic_io::{RewardLogic, RewardLogicAction, RewardLogicEvent};

static mut REWARD_LOGIC: Option<RewardLogic> = None;

fn reward_logic_mut() -> &'static mut RewardLogic {
    unsafe { REWARD_LOGIC.get_or_insert(Default::default()) }
}

#[no_mangle]
extern fn init() {
    let reward_logic = RewardLogic::new();

    unsafe { REWARD_LOGIC = Some(reward_logic) }

    reward_logic_mut().admin = Some(msg::source());
}

#[async_main]
async fn main() {
    let action: RewardLogicAction = msg::load().expect("");
    let reward_logic = reward_logic_mut();
    match action {
        RewardLogicAction::AddAddressFT(address) => {
            if reward_logic.admin.expect("") != msg::source() {
                panic!("Add Address FT Action can only be called by admin")
            }
            reward_logic.address_ft = Some(address);
            msg::reply(RewardLogicEvent::FTAddressAdded, 0).expect("");
        }

        RewardLogicAction::AddAddressLogic(address) => {
            if reward_logic.admin.expect("") != msg::source() {
                panic!("Add Address Logic Action can only be called by admin")
            }
            reward_logic.address_logic = Some(address);
            msg::reply(RewardLogicEvent::LogicAddressAdded, 0).expect("");
        }

        RewardLogicAction::AddAddressStorage(address) => {
            if reward_logic.admin.expect("") != msg::source() {
                panic!("Add Address Storage Action can only be called by admin")
            }
            reward_logic.address_storage = Some(address);
            msg::reply(RewardLogicEvent::StorageAddressAdded, 0).expect("");
        }

        RewardLogicAction::TriggerRewardLogic(thread_id) => {
            reward_logic.trigger_reward_logic(thread_id).await;
            msg::reply(RewardLogicEvent::RewardLogicTriggered, 0).expect("");
        }
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
