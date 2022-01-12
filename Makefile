all: release

lint:
	cargo clippy

build:
	cargo build

release:
	if [[ "Linux" == `uname -s` ]]; then\
	    cargo build --release --target=x86_64-unknown-linux-musl;\
	else\
	    cargo build --release;\
	fi

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
