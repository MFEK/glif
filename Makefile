export RUSTFLAGS := -Awarnings
export RUST_LOG := debug
export RUST_BACKTRACE := 1

.PHONY: all
all:
	cargo build

.PHONY: testrun
testrun:
	cargo run -- Q_.glif

.PHONY: fmt
fmt:
	find src -type f -iname '*.rs' | parallel --bar RUST_LOG=error rustfmt {}
