# plgr

## Usage

```
plgr 0.1.0

USAGE:
    plgr [OPTIONS] --entries-path <ENTRIES_PATH> --privkey-path <PRIVKEY_PATH>

OPTIONS:
    -a, --rpc-addr <RPC_ADDR>            Optional, like: http://***:8545
        --bsc-testnet                    Optional, default to BSC mainnet
    -c, --contract <CONTRACT>            Optional, like: 0x816d8...40C9a
    -h, --help                           Print help information
    -K, --privkey-path <PRIVKEY_PATH>    A file containing your private key
    -p, --entries-path <ENTRIES_PATH>    A file containing who and how much to transfer
    -V, --version                        Print version information
```

## Example

> Sample contents of an entry file:
>
> ```
> 0xCBf28E280eeF69d0854159846AB6e154c6b3DB6a,99.1
> 0x19c342e80D87cDC6FfF849762BD8dF006A26264F, 77
> 0xFFc342e80D87cDC6FfF849762BD8dF006A262600, 0.1
>```
>
> Sample contents of a private key file:
>
> ```
> 9aa840439e7268ecb2d27876c57bd9a20e9605144f1672958e14f53eafb6e9fa
>```

#### BSC testnet

```shell
make release

# use your own contact on the BSC testnet
./target/release/plgr --bsc-testnet \
                      -p <file path to your entries> \
                      -K <file path to private key> \
                      -c <PLGR contract address>
```

Quick testing on the BSC testnet:

```shell
make test
```

#### BSC mainnet

```shell
make release

# maybe need some proxy settings:
# - export HTTP_PROXY="127.0.0.1:19180"
# - export HTTPS_PROXY="127.0.0.1:19180"
# - ...

# use the official contract
./target/release/plgr -p <file path to your entries> -K <file path to private key>

# use a custom contract
./target/release/plgr -p <file path to your entries> \
                      -K <file path to private key> \
                      -c <PLGR contract address>
```

## Compilation

`make release`
