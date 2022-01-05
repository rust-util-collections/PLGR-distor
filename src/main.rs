use futures::executor::block_on;
use ruc::*;
use secp256k1::SecretKey;
use std::{fs, str::FromStr};

use web3::{
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters},
};

const BSC_MAINNET: &str = "https://bsc-dataseed1.binance.org:8545";
const BSC_TESTNET: &str = "https://data-seed-prebsc-1-s1.binance.org:8545";

fn main() {
    pnk!(run());
}

fn run() -> Result<()> {
    let cmdline = parse_cmdline().c(d!())?;
    let url = if cmdline.use_testnet {
        BSC_TESTNET
    } else {
        BSC_MAINNET
    };

    let transport = web3::transports::Http::new(url).c(d!())?;
    let web3 = web3::Web3::new(transport);

    let prvk = SecretKey::from_str(&pnk!(fs::read_to_string(cmdline.privkey_path))).c(d!())?;

    for l in fs::read_to_string(cmdline.entries_path).c(d!())?.lines() {
        let res = l.split(',').collect::<Vec<_>>();

        if 2 != res.len() {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }

        let receiver = res[0];
        if !res[0].starts_with("0x") {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }

        let amount = res[1]
            .parse::<f64>()
            .c(d!())
            .or_else(|e| res[1].parse::<u128>().map(|am| am as f64).c(d!(e)))?;

        let am = amount as u128 * 10u128.pow(18);

        let tx = TransactionParameters {
            to: Some(pnk!(Address::from_str(receiver))),
            value: pnk!(U256::from_dec_str(&am.to_string())),
            ..Default::default()
        };

        let signed =
            block_on(async { web3.accounts().sign_transaction(tx, &prvk).await }).c(d!())?;

        let result = block_on(async {
            web3.eth()
                .send_raw_transaction(signed.raw_transaction)
                .await
        })
        .c(d!())?;

        println!(
            "Tx succeeded with hash: {}, to: {}, amount: {}",
            result, receiver, am
        );
    }

    Ok(())
}

fn parse_cmdline() -> Result<CmdLine> {
    todo!()
}

struct CmdLine {
    use_testnet: bool,
    // A file contains how much to transfer:
    // - 0xAAAAAAA...AAAAAAAAAA <amount>
    // - ...
    entries_path: String,
    privkey_path: String,
}
