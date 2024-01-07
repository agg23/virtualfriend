run:
	cd virtualfriend_desktop && rm -f instructions.log && cargo run --release
profile:
	cd virtualfriend_desktop && cargo flamegraph --dev --root

3ds:
	cd virtualfriend_3ds && cargo 3ds run --address 192.168.1.169 --server --release