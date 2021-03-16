export RUSTFLAGS := -Awarnings
export RUST_LOG := debug
export RUST_BACKTRACE := 1

.PHONY: all
all:
	cargo build

TESTFILE := $(if $(TESTFILE),$(TESTFILE),Q_.glif)

.PHONY: testrun
testrun:
	cargo run -- $(TESTFILE)

.PHONY: fmt
fmt:
	find src -type f -iname '*.rs' | parallel --bar RUST_LOG=error rustfmt {}
