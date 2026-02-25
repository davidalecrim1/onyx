.PHONY: build run test lint check clean format

build:
	cargo build

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check --all-targets

format:
	cargo fmt --all

clean:
	cargo clean
