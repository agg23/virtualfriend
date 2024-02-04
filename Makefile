run:
	cd virtualfriend_desktop && rm -f instructions.log && cargo run --release
flamegraph:
	cd virtualfriend_desktop && export CARGO_PROFILE_RELEASE_DEBUG=true && cargo flamegraph --bench complete --root --min-width 0.0001
profile:
	cd virtualfriend_desktop && export CARGO_PROFILE_RELEASE_DEBUG=true && cargo instruments --bench complete -t "CPU Profiler" --time-limit 20000

leaks:
	cd virtualfriend_desktop && cargo instruments --release -t Leaks --output leak.trace

3ds:
	cd virtualfriend_3ds && cargo 3ds run --address 192.168.1.169 --server --release

3ds-asm:
	cargo asm -p virtualfriend --rust --target armv6k-nintendo-3ds -Z build-std=std --lib virtualfriend::cpu_v810::CpuV810::step > output.asm

vision:
	cd virtualfriend_swift && xargo build --target aarch64-apple-xros --release -v