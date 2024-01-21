#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new() -> VirtualFriend;

        fn run(&self, rom_path: String);
    }
}

pub struct VirtualFriend {}

impl VirtualFriend {
    fn new() -> Self {
        VirtualFriend {}
    }

    fn run(&self, rom_path: String) {
        virtualfriend::run(rom_path)
    }
}
