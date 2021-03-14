export RUSTFLAGS := -Awarnings
export RUST_LOG := debug
export RUST_BACKTRACE := 1

.PHONY: all
all:
	cargo build

.PHONY: testrun
testrun:
	cargo run -- Q_.glif
