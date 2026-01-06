.PHONY: fmt fmt-check build

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

build:
	cargo build --locked
