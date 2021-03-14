all:
	RUSTFLAGS=-Awarnings RUST_LOG=debug RUST_BACKTRACE=1 cargo build

.PHONY: testrun
testrun:
	RUSTFLAGS=-Awarnings RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- Q_.glif
