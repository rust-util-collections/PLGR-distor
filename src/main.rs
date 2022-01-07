use clap::Parser;
use ruc::*;
use secp256k1::SecretKey;
use std::{fs, str::FromStr};
use tokio::{
    runtime::Runtime,
    time::{sleep, Duration},
};

use web3::{
    contract::{Contract, Options},
    signing::{Key, SecretKeyRef},
    transports::Http,
    types::{Address, U256},
    Web3,
};

const BSC_MAINNET: &str = "https://bsc-dataseed1.binance.org";
const CONTRACT_MAINNET: &str = "0x6aa91cbfe045f9d154050226fcc830ddba886ced";

const BSC_TESTNET: &str = "https://data-seed-prebsc-1-s1.binance.org:8545";
const CONTRACT_TESTNET: &str = "0x816d8FB30bD109e75E339f341965f7B46E140C9a";

const GOOD: &str = "\x1b[35;01mGOOD\x1b[0m";
const FAIL: &str = "\x1b[31;01mFAIL\x1b[0m";

fn main() {
    pnk!(run());
}

fn run() -> Result<()> {
    let rt = Runtime::new().c(d!())?;
    let args = Args::parse();

    let url = args
        .rpc_addr
        .as_deref()
        .unwrap_or_else(|| alt!(args.bsc_testnet, BSC_TESTNET, BSC_MAINNET));
    let contract_addr = args
        .contract
        .as_deref()
        .unwrap_or_else(|| alt!(args.bsc_testnet, CONTRACT_TESTNET, CONTRACT_MAINNET));

    let transport = Http::new(url).c(d!())?;
    let web3 = Web3::new(transport);

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
        alt!(line.is_empty(), continue);
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

    let mut entries_old_balances = vec![];
    for en in entries.iter() {
        let balance: U256 = rt
            .block_on(contract.query("balanceOf", (en.0,), None, Options::default(), None))
            .c(d!())?;
        entries_old_balances.push(balance.as_u128());
    }

    let sender = SecretKeyRef::new(&prvk).address();
    let total_am = (entries
        .iter()
        .fold(entries.len() as f64, |acc, i| acc + i.1)
        * (10u128.pow(18) as f64)) as u128;
    let balance: U256 = rt
        .block_on(contract.query("balanceOf", (sender,), None, Options::default(), None))
        .c(d!())?;
    let balance = balance.as_u128();
    if total_am > balance {
        let mint_am = total_am - balance;
        rt.block_on(contract.signed_call_with_confirmations(
            "mint",
            (mint_am,),
            Options::default(),
            3,
            SecretKeyRef::new(&prvk),
        ))
        .c(d!("Insufficient balance, and mint failed!"))?;
        println!("=> Mint {}", to_float_str(mint_am));
    }

    let nonce = rt
        .block_on(web3.eth().transaction_count(sender, None))
        .c(d!("Fail to get nonce"))?
        .as_u128();

    println!("=> \x1b[37;1mSending from: 0x{:x}\x1b[0m", sender);
    let mut res = vec![];
    for (i, (receiver, amount)) in entries.clone().into_iter().enumerate() {
        let am = (amount * (10u128.pow(18) as f64)) as u128;
        let options = Options {
            gas: Some(200_0000.into()),
            nonce: Some((i as u128 + nonce).into()),
            ..Default::default()
        };
        let c = contract.clone();
        let hdr = rt.spawn(async move {
            sleep(Duration::from_millis(100 * i as u64)).await;
            c.signed_call_with_confirmations(
                "transfer",
                (receiver, am),
                options,
                3,
                SecretKeyRef::new(&prvk),
            )
            .await
        });
        res.push((hdr, amount, receiver));
    }

    for (hdr, am, receiver) in res.into_iter() {
        let ret = rt.block_on(hdr).unwrap().c(d!())?;
        println!(
            "=> Result: {}, Amount: {}, SendTo: 0x{:x}, TxHash: {}",
            ret.status
                .map(|r| alt!(1 == r.as_u32(), GOOD, FAIL))
                .unwrap_or(FAIL),
            am,
            receiver,
            ret.transaction_hash,
        );
    }

    println!("=> \x1b[37;1mCheck on-chain results...\x1b[0m");
    for ((receiver, amount), old_balance) in
        entries.into_iter().zip(entries_old_balances.into_iter())
    {
        let am = (amount * (10u128.pow(18) as f64)) as u128;
        let balance: U256 = rt
            .block_on(contract.query("balanceOf", (receiver,), None, Options::default(), None))
            .c(d!())?;
        let balance = balance.as_u128();
        println!(
            "=> Result: {}, Amount: {}, BalanceDiff: {}, NewBalance: {}, OldBalance: {}, Receiver: 0x{:x}",
            alt!(am == balance - old_balance, GOOD, FAIL),
            amount,
            to_float_str(balance - old_balance),
            to_float_str(balance),
            to_float_str(old_balance),
            receiver,
        );
    }

    Ok(())
}

fn to_float_str(n: u128) -> String {
    let i = n / 10u128.pow(18);
    let j = n - i * 10u128.pow(18);
    (i.to_string() + "." + j.to_string().trim_end_matches('0'))
        .trim_end_matches('.')
        .to_owned()
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(long, help = "Optional, default to BSC mainnet")]
    bsc_testnet: bool,
    #[clap(
        short = 'p',
        long,
        help = "A file containing who and how much to transfer"
    )]
    entries_path: String,
    #[clap(short = 'K', long, help = "A file containing your private key")]
    privkey_path: String,
    #[clap(short = 'a', long, help = "Optional, like: http://***:8545")]
    rpc_addr: Option<String>,
    #[clap(short = 'c', long, help = "Optional, like: 0x816d8...40C9a")]
    contract: Option<String>,
}
