.PHONY: install install-tools dev build test lint check format clean coverage coverage-frontend coverage-backend

install:
	npm install

install-tools:
	cargo install cargo-llvm-cov

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

coverage-frontend:
	npm run coverage

coverage-backend:
	cd src-tauri && cargo llvm-cov --html && cargo llvm-cov report

coverage: coverage-frontend coverage-backend

clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules
