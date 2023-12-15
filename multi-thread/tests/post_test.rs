use std::string::ToString;
use gstd::{ActorId};
use gtest::{Program, System};
use io::{InitThread, ThreadAction, ThreadType};

const ADDRESS: &str = "0xc2a1ec37748d434fc24687a656b6f8ac5ba8af088b4a62aeb82db75fd6dfa467";
const ID: &str = "sf123x";
const THREAD_TYPE: ThreadType = ThreadType::Challenge;
const CONTENT: &str = "Neque porro quisquam est qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit...";

#[test]
fn init() {
    let sys = System::new();
    sys.init_logger();
    let thread = Program::current(&sys);
    let res = thread.send(2,
    InitThread {
        id: ID.to_string(),
        thread_type: THREAD_TYPE,
        title: "dfdf".to_string(),
        content: CONTENT.parse().unwrap()
    }
    );
    assert!(!res.main_failed());
}

#[test]
fn handle() {
    let sys = System::new();
    sys.init_logger();
    let thread = Program::current(&sys);
    let res = thread.send(2, ThreadAction::EndThread);
    assert!(!res.main_failed());
}
