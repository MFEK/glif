COMMA:=,
SOURCES:=$(shell find src -iname '*.rs')
SHELL:=/bin/bash

MFEK_MODULE := MFEKglif
export MFEK_ARGS := $(if $(MFEK_ARGS),$(MFEK_ARGS),'examples/Layered.ufo/glyphs/S_.rhigh.layered.glifjson')

UNAME_FIRST=$(word 1, $(shell uname -a))
DEFAULT_CARGO_FLAGS := $(shell if [[ "$(UNAME_FIRST)" =~ Linux ]]; then echo --features=sdl2-dynamic; else echo --features=sdl2-static; fi)
export CARGO_FLAGS := $(if $(CARGO_FLAGS),$(CARGO_FLAGS),$(DEFAULT_CARGO_FLAGS))
export CARGO_PROFILE := $(if $(CARGO_PROFILE),$(CARGO_PROFILE),debug)
ifneq ($(strip $(CARGO_PROFILE)),debug)
export CARGO_PROFILE_ARG := --$(CARGO_PROFILE)
endif

# Cargo flags
export RUST_LOG := $(if $(RUST_LOG),$(RUST_LOG),MFEKglif=debug$(COMMA)mfek_ipc=trace)
export RUST_BACKTRACE := $(if $(RUST_BACKTRACE),$(RUST_BACKTRACE),)

all: build

.PHONY: cargo
cargo:
	@env | grep -E 'MFEK|RUST|CARGO' &&\
	RUST_LOG="$(RUST_LOG)" RUST_BACKTRACE="$(RUST_BACKTRACE)" env cargo $(CARGO_CMD) $(CARGO_PROFILE_ARG) $(CARGO_FLAGS) $(MFEK_FLAGS)

.PHONY: clean
clean:
	cargo clean

target/$(CARGO_PROFILE)/$(MFEK_MODULE): $(SOURCES)
	$(MAKE) CARGO_CMD=build cargo

.PHONY .SILENT: build
build:
	$(MAKE) target/$(CARGO_PROFILE)/$(MFEK_MODULE)

.PHONY .SILENT: testrun
testrun:
	$(MAKE) build &&\
	target/$(CARGO_PROFILE)/$(MFEK_MODULE) $(MFEK_ARGS)

.PHONY .SILENT: echo-%
echo-%:
	@$(MAKE) -s --just-print $*

# --lzma due to upx/upx#224 (GitHub)
.PHONY: dist
dist:
	$(MAKE) CARGO_PROFILE=release build &&\
	which upx || (>&2 echo "Error: upx not installed." && exit 1) &&\
	mkdir -p target/release-upx &&\
	(upx --best --lzma -o target/release-upx/$(MFEK_MODULE) target/release/$(MFEK_MODULE) || (>&2 echo "Error: upx failed." && exit 1))

.PHONY: fmt
fmt:
	@FILES="`git ls-files | grep -E '.rs$$'`" &&\
	parallel --bar RUST_LOG=error rustfmt {} <<< "$$FILES" &&\
	cargo fmt --all -- --check &&\
	echo ✅

