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
	cargo run --release -- -t -e testnet/owner.entries -p testnet/owner.key
	cargo run --release -- -t -e testnet/investor.entries -p testnet/investor.key
