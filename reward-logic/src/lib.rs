#![no_std]

use gstd::async_main;

#[no_mangle]
extern fn init() {}

#[async_main]
async fn main() {}
