export RUSTFLAGS :=
export RUST_LOG := debug
export RUST_BACKTRACE := 1
export TESTFILE := $(if $(TESTFILE),$(TESTFILE),examples/Q_.glif)

RUST_APP := MFEKglif

all: build

.PHONY: build
build:
	cargo build $(CARGOFLAGS)

.PHONY: testrun
testrun:
	cargo run -- $(TESTFILE)

.PHONY: dist
dist:
	make CARGOFLAGS='--release' build
	mkdir -p target/release-upx
	upx --best -o target/release-upx/$(RUST_APP) target/release/$(RUST_APP)

.PHONY: fmt
fmt:
	find src -type f -iname '*.rs' | parallel --bar RUST_LOG=error rustfmt {}
