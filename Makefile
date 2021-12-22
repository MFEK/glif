COMMA:=,

export CARGOFLAGS := $(if $(CARGOFLAGS),$(CARGOFLAGS),)
export RUSTFLAGS := $(if $(RUSTFLAGS),$(RUSTFLAGS),)
export RUST_LOG := $(if $(RUST_LOG),$(RUST_LOG),MFEKglif=debug$(COMMA)mfek_ipc=trace)
export RUST_BACKTRACE := $(if $(RUST_BACKTRACE),$(RUST_BACKTRACE),)
export TESTFILE := $(if $(TESTFILE),$(TESTFILE),examples/Q_.glif)

MFEK_MODULE := MFEKglif

all: build

.PHONY: datestamp
datestamp:
	./mk/cargo_config.sh || git checkout .cargo/config.toml

.PHONY: clean-datestamp
clean-datestamp:
	git checkout .cargo/config.toml

.PHONY: build
build: datestamp
	RUST_LOG="$(RUST_LOG)" RUST_BACKTRACE="$(RUST_BACKTRACE)" cargo build $(CARGOFLAGS)

.PHONY: testrun
testrun:
	RUST_LOG="$(RUST_LOG)" RUST_BACKTRACE="$(RUST_BACKTRACE)" cargo run $(CARGOFLAGS) -- $(TESTFILE)

.PHONY: echo-testrun
echo-testrun:
	echo RUST_LOG="$(RUST_LOG)" RUST_BACKTRACE="$(RUST_BACKTRACE)" cargo run $(CARGOFLAGS) -- $(TESTFILE)

.PHONY: dist
dist: build
	mkdir -p target/release-upx
	upx --best -o target/release-upx/$(MFEK_MODULE) target/release/$(MFEK_MODULE)

.PHONY: fmt
fmt:
	find src -type f -iname '*.rs' | parallel --bar RUST_LOG=error rustfmt {}

resources/fonts/icons.ttf:
	fontmake -u resources/fonts/$(MFEK_MODULE)IconFont.ufo -o ttf --output-path $@

.PHONY: iconfont
iconfont: resources/fonts/icons.ttf
