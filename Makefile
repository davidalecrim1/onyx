.PHONY: install dev build test lint check format clean

install:
	npm install

dev:
	npm run tauri dev

build:
	npm run tauri build

test:
	cd src-tauri && cargo test

lint:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
	npx eslint --ext .ts,.tsx src/

check:
	cd src-tauri && cargo check --all-targets

format:
	cd src-tauri && cargo fmt
	npx prettier --write src/

clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules
