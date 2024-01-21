pub mod bus;
pub mod constants;
pub mod cpu_internals;
pub mod cpu_v810;
pub mod gamepad;
pub mod hardware;
pub mod interrupt;
pub mod rom;
pub mod timer;
pub mod util;
pub mod vip;
mod virtualfriend;
pub mod vsu;

#[no_mangle]
pub extern "C" fn virtualfriend_test() {
    println!("Called from Swift");
}
