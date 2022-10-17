all: install

lint:
	cargo clippy

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

fmt:
	cargo fmt

clean:
	git clean -fdx
	cargo clean

update:
	cargo update

test:
	cargo run --release -- --bsc-testnet -p testnet/owner.entries -K testnet/owner.key

test2:
	cargo run --release -- --bsc-testnet -p testnet/investor.entries -K testnet/investor.key
