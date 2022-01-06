all: lint

lint:
	cargo clippy

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt

clean:
	git clean -fdx
	cargo clean

update:
	cargo update

test:
	cargo run --release -- -t -e testnet/receiver.list -p testnet/owner.key
