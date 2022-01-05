use clap::Parser;
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
    let args = Args::parse();

    let url = if args.testnet {
        BSC_TESTNET
    } else {
        BSC_MAINNET
    };

    let transport = web3::transports::Http::new(url).c(d!())?;
    let web3 = web3::Web3::new(transport);

    let prvk = SecretKey::from_str(&pnk!(fs::read_to_string(args.privkey_path))).c(d!())?;
    let contents = fs::read_to_string(args.entries_path).c(d!())?;

    let mut entries = vec![];
    for l in contents.lines() {
        let en = l.split(',').collect::<Vec<_>>();
        if 2 != en.len() {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }
        if !en[0].starts_with("0x") || 34 != en[0].len() {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }
        let receiver = en[0];
        let amount = en[1]
            .parse::<f64>()
            .or_else(|_| en[1].parse::<u128>().map(|am| am as f64))
            .c(d!("Invalid amount"))?;
        entries.push((receiver, amount));
    }

    for (receiver, amount) in entries.into_iter() {
        let am = (amount * (10u128.pow(18) as f64)) as u128;
        let tx = TransactionParameters {
            to: Some(Address::from_str(receiver).c(d!())?),
            value: U256::from_dec_str(&am.to_string()).c(d!())?,
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
            result, receiver, amount
        );
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    testnet: bool,
    // A file contains how much to transfer:
    // - 0xAAAAAAA...AAAAAAAAAA <amount>
    // - ...
    #[clap(short, long)]
    entries_path: String,
    #[clap(short, long)]
    privkey_path: String,
}
