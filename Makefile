run:
	cd virtualfriend_desktop && rm -f instructions.log && cargo run --release
profile:
	cd virtualfriend_desktop && cargo flamegraph --dev --root