use clap::Parser;
use ruc::*;
use secp256k1::SecretKey;
use std::{fs, str::FromStr};
use tokio::runtime::Runtime;

use web3::{
    contract::{Contract, Options},
    signing::SecretKeyRef,
    types::{Address, U256},
};

const BSC_MAINNET: &str = "https://bsc-dataseed1.binance.org";
const CONTRACT_MAINNET: &str = "0x6aa91cbfe045f9d154050226fcc830ddba886ced";

const BSC_TESTNET: &str = "https://data-seed-prebsc-1-s1.binance.org:8545";
const CONTRACT_TESTNET: &str = "0x816d8FB30bD109e75E339f341965f7B46E140C9a";

fn main() {
    pnk!(run());
}

fn run() -> Result<()> {
    let rt = Runtime::new().c(d!())?;
    let args = Args::parse();

    let (url, contract_addr) = if args.testnet {
        (BSC_TESTNET, CONTRACT_TESTNET)
    } else {
        (BSC_MAINNET, CONTRACT_MAINNET)
    };

    let transport = web3::transports::Http::new(url).c(d!())?;
    let web3 = web3::Web3::new(transport);

    let prvk = fs::read_to_string(args.privkey_path)
        .c(d!())
        .and_then(|c| SecretKey::from_str(c.trim()).c(d!()))?;
    let contract = Contract::from_json(
        web3.eth(),
        Address::from_str(contract_addr).c(d!())?,
        include_bytes!("token.json"),
    )
    .c(d!())?;

    let contents = fs::read_to_string(args.entries_path).c(d!())?;

    let mut entries = vec![];
    for l in contents.lines() {
        let line = l.replace(" ", "");
        let en = line.split(',').collect::<Vec<_>>();
        if 2 != en.len() {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }
        if !en[0].starts_with("0x") || 42 != en[0].len() {
            return Err(eg!(format!("Invalid entry: {}", l)));
        }
        let receiver = Address::from_str(en[0]).c(d!(format!("Invalid address: {}", l)))?;
        let amount = en[1]
            .parse::<f64>()
            .or_else(|_| en[1].parse::<u128>().map(|am| am as f64))
            .c(d!(format!("Invalid amount: {}", l)))?;
        entries.push((receiver, amount));
    }

    for (receiver, amount) in entries.into_iter() {
        let am = (amount * (10u128.pow(18) as f64)) as u128;
        let options = Options {
            gas: Some(U256::from_dec_str("2000000").unwrap()),
            ..Default::default()
        };
        let ret = rt
            .block_on(contract.signed_call_with_confirmations(
                "transfer",
                (receiver, am),
                options,
                1,
                SecretKeyRef::new(&prvk),
            ))
            .c(d!())?;

        println!(
            "Tx hash: {}, send_to: {}, amount: {}, result: {}",
            ret.transaction_hash,
            receiver,
            amount,
            ret.status
                .map(|r| alt!(1 == r.as_u32(), "success", "fail"))
                .unwrap_or("fail")
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
