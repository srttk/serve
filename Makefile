dev:
	cargo run -- $(ARGS)

## build: Build the production binary
build:
	cargo build --release

## install: Install the binary to the local system
install:
	cargo install --path .

## test: Run all unit tests
test:
	cargo test

uninstall:
	cargo uninstall serve