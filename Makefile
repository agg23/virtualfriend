run:
	cd virtualfriend_desktop && rm -f instructions.log && cargo run --release
flamegraph:
	cd virtualfriend_desktop && export CARGO_PROFILE_RELEASE_DEBUG=true && cargo flamegraph --root
profile:
	cd virtualfriend_desktop && export CARGO_PROFILE_RELEASE_DEBUG=true && cargo instruments -t "CPU Profiler" --time-limit 5000

3ds:
	cd virtualfriend_3ds && cargo 3ds run --address 192.168.1.169 --server --release