# plgr

`make test`

```shell
zsh 757  (git)-[master]-% make test
cargo run --release -- -t -e testnet/owner.entries -p testnet/owner.key
   Compiling plgr v0.1.0 (/Users/fh/plgr)
    Finished release [optimized] target(s) in 3.73s
     Running `target/release/plgr -t -e testnet/owner.entries -p testnet/owner.key`

=> Mint 174.44588

=> Sending from: 0x13f81fa8d4bda1b7dd560d579a099aafc1493f56

=> Result: GOOD, Amount: 99.1, SendTo: 0xcbf28e280eef69d0854159846ab6e154c6b3db6a, TxHash: 0x3856…2c3a

=> Result: GOOD, Amount: 77, SendTo: 0x19c342e80d87cdc6fff849762bd8df006a26264f, TxHash: 0xce56…5863

=> Check on-chain results...

=> Result: GOOD, Amount: 99.1, BalanceDiff: 99.1, NewBalance: 51208.134080000001323065, OldBalance: 51307.234080000001323065, SendTo: 0xcbf28e280eef69d0854159846ab6e154c6b3db6a

=> Result: GOOD, Amount: 77, BalanceDiff: 77., NewBalance: 2541., OldBalance: 2618., SendTo: 0x19c342e80d87cdc6fff849762bd8df006a26264f
cargo run --release -- -t -e testnet/investor.entries -p testnet/investor.key
    Finished release [optimized] target(s) in 0.05s
     Running `target/release/plgr -t -e testnet/investor.entries -p testnet/investor.key`

=> Sending from: 0xcbf28e280eef69d0854159846ab6e154c6b3db6a

=> Result: GOOD, Amount: 77, SendTo: 0x19c342e80d87cdc6fff849762bd8df006a26264f, TxHash: 0x8817…c036

=> Result: GOOD, Amount: 1.65412, SendTo: 0x13f81fa8d4bda1b7dd560d579a099aafc1493f56, TxHash: 0x984b…fc55

=> Check on-chain results...

=> Result: GOOD, Amount: 77, BalanceDiff: 77., NewBalance: 2618., OldBalance: 2695., SendTo: 0x19c342e80d87cdc6fff849762bd8df006a26264f

=> Result: GOOD, Amount: 1.65412, BalanceDiff: 1.65412, NewBalance: 2., OldBalance: 3.65412, SendTo: 0x13f81fa8d4bda1b7dd560d579a099aafc1493f56
```

## Usage

```
plgr 0.1.0

USAGE:
    plgr [OPTIONS] --entries-path <ENTRIES_PATH> --privkey-path <PRIVKEY_PATH>

OPTIONS:
    -e, --entries-path <ENTRIES_PATH>
    -h, --help                           Print help information
    -p, --privkey-path <PRIVKEY_PATH>
    -t, --testnet
    -V, --version                        Print version information
```
