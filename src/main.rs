use clap::Parser;
use ruc::*;
use secp256k1::SecretKey;
use std::{
    collections::BTreeMap,
    fs,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
    sync::mpsc::channel,
};
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
const CONTRACT_TESTNET: &str = "0xffe5548b5c3023b3277c1a6f24ac6382a0087db5";

const GOOD: &str = "\x1b[35;01mGOOD\x1b[0m";
const FAIL: &str = "\x1b[31;01mFAIL\x1b[0m";
const UNKNOWN: &str = "\x1b[39;01mUNKNOWN\x1b[0m";

static PRINT_IDX: AtomicU64 = AtomicU64::new(0);

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

    let mut entries = BTreeMap::new();
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
        *entries.entry(receiver).or_insert(0.0) += amount;
    }

    for batch in entries.into_iter().collect::<Vec<_>>().chunks(100) {
        run_batch(&web3, &rt, batch, prvk, &contract).c(d!())?;
    }

    Ok(())
}

fn run_batch(
    web3: &Web3<Http>,
    rt: &Runtime,
    entries: &[(Address, f64)],
    prvk: SecretKey,
    contract: &Contract<Http>,
) -> Result<()> {
    let (s, r) = channel();
    for (idx, en) in entries.iter().enumerate() {
        let ss = s.clone();
        let data = (en.0,);
        let c = contract.clone();
        rt.spawn(async move {
            sleep(Duration::from_millis(10 * idx as u64)).await;
            let balance: U256 = c
                .query("balanceOf", data, None, Options::default(), None)
                .await
                .unwrap();
            ss.send((idx, balance.as_u128())).unwrap();
        });
    }

    let mut entries_pre_balances = BTreeMap::new();
    for idx in 0..entries.len() {
        let (i, balance) = r.recv().unwrap();
        entries_pre_balances.insert(i, balance);
        println!("Querying pre-balances nth-{}", idx);
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
        println!("=> Minting: {}", to_float_str(mint_am));
        rt.block_on(contract.signed_call_with_confirmations(
            "mint",
            (mint_am,),
            Options::default(),
            2,
            SecretKeyRef::new(&prvk),
        ))
        .c(d!("Insufficient balance, and mint failed!"))?;
    }

    let nonce = rt
        .block_on(web3.eth().transaction_count(sender, None))
        .c(d!("Fail to get nonce"))?
        .as_u128();

    println!("=> \x1b[37;1mSending from: 0x{:x}\x1b[0m", sender);
    for (idx, (receiver, amount)) in entries.iter().copied().enumerate() {
        rt.block_on(async {
            let am = (amount * (10u128.pow(18) as f64)) as u128;
            let options = Options {
                nonce: Some((idx as u128 + nonce).into()),
                ..Default::default()
            };
            let ret = contract
                .signed_call_with_confirmations(
                    "transfer",
                    (receiver, am),
                    options,
                    0,
                    SecretKeyRef::new(&prvk),
                )
                .await
                .unwrap();
            println!(
                "=> Result-{}: {}, Amount: {}, SendTo: 0x{:x}, TxHash: {}",
                idx,
                ret.status
                    .map(|r| alt!(1 == r.as_u32(), GOOD, FAIL))
                    .unwrap_or(UNKNOWN),
                am,
                receiver,
                ret.transaction_hash,
            );
        });
    }

    println!("=> \x1b[37;1mCheck on-chain results...\x1b[0m");
    PRINT_IDX.store(0, Ordering::Relaxed);
    for (idx, ((receiver, amount), pre_balance)) in entries
        .iter()
        .copied()
        .zip(entries_pre_balances.into_iter().map(|(_, v)| v))
        .enumerate()
    {
        let c = contract.clone();
        rt.spawn(async move {
            let am = (amount * (10u128.pow(18) as f64)) as u128;
            sleep(Duration::from_millis(10 * idx as u64)).await;
            let balance: U256 = c.query("balanceOf", (receiver,), None, Options::default(), None).await.unwrap();
            let balance = balance.as_u128();
            println!(
                "=> Result-{}: {}, Amount: {}, BalanceDiff: {}, NewBalance: {}, OldBalance: {}, Receiver: 0x{:x}",
                PRINT_IDX.fetch_add(1, Ordering::Relaxed),
                alt!(am == balance - pre_balance, GOOD, FAIL),
                amount,
                to_float_str(balance - pre_balance),
                to_float_str(balance),
                to_float_str(pre_balance),
                receiver,
            );
        });
    }

    Ok(())
}

fn to_float_str(n: u128) -> String {
    let base = 10u128.pow(18);
    let i = n / base;
    let j = n - i * base;

    let pads = if 0 == i {
        18 - (1..=18)
            .into_iter()
            .find(|&k| 0 == j / 10u128.pow(k))
            .unwrap()
    } else {
        0
    };
    let pads = (0..pads).map(|_| '0').collect::<String>();

    (i.to_string() + "." + &pads + j.to_string().trim_end_matches('0'))
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
