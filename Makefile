run:
	rm -f instructions.log && cargo run --release
profile:
	cargo flamegraph --dev --root