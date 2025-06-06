use std::{
    str::FromStr, time::Duration, time::SystemTime, time::UNIX_EPOCH
};

use clap::Parser;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, account::Account, pubkey::Pubkey
};

use orca_whirlpools_client::{
    Whirlpool, TickArray, Oracle, 
};
use orca_whirlpools_core::{
    WhirlpoolFacade, TickArrayFacade, TickArrays, OracleFacade, TransferFee, ExactOutSwapQuote, ExactInSwapQuote, 
    get_tick_array_start_tick_index, swap_quote_by_output_token, swap_quote_by_input_token
};

mod pda;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long = "rpc-url", default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: String,

    // Default pool is SOL/USDC
    #[arg(long = "pool-address", default_value = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE")]
    pool_address: String,

    #[arg(long = "detailed-time-log")]
    detailed_time_log: bool,

    #[arg(long = "volume-out")]
    volume_out: u64,

    #[arg(long = "slippage-tolerance", default_value = "9999")]
    slippage_tolerance: u16
}


fn main() {
    let args = Args::parse(); // Parse CLI args

    if args.detailed_time_log{
        let start_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let start_secs: u64 = start_unix.as_secs();
        let start_millis: u64 = start_unix.subsec_millis() as u64; 
        println!("Start Unix time: {}.{}", start_secs, start_millis);
    };

    // Initialise connection 

    let rpc_url = args.rpc_url;
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    if args.detailed_time_log{
        let connection_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let connection_secs: u64 = connection_unix.as_secs();
        let connection_millis: u64 = connection_unix.subsec_millis() as u64; 
        println!("Connection Unix time: {}.{}", connection_secs, connection_millis);
    };

    // Fetch pool account

    let pool_pubkey_str = args.pool_address.as_str(); 
    let pool_pubkey = Pubkey::from_str(pool_pubkey_str).expect("Failed to derive pubkey from string");
    let pool_account: Account = rpc_client.get_account(&pool_pubkey).unwrap(); 

    println!("\n=== Full Pool Account Data ===\n {:#?}", pool_account.data);

    if args.detailed_time_log{
        let pool_fetch_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let pool_fetch_secs: u64 = pool_fetch_unix.as_secs();
        let pool_fetch_millis: u64 = pool_fetch_unix.subsec_millis() as u64; 
        println!("Pool Fetch Unix time: {}.{}", pool_fetch_secs, pool_fetch_millis);
    };

    // Decode pool account into WhirlpoolFacade struct

    let whirlpool: Whirlpool = Whirlpool::from_bytes(&pool_account.data).unwrap();
    let whirlpool_facade: WhirlpoolFacade = WhirlpoolFacade::from(whirlpool);

    if args.detailed_time_log{
        let pool_decode_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let pool_decode_secs: u64 = pool_decode_unix.as_secs();
        let pool_decode_millis: u64 = pool_decode_unix.subsec_millis() as u64; 
        println!("Pool Decode Unix time: {}.{}", pool_decode_secs, pool_decode_millis);
    };

    // Find start tick index of current tick array

    let current_tick_array_start_index = get_tick_array_start_tick_index(
        whirlpool_facade.tick_current_index, whirlpool_facade.tick_spacing
    );
    
    if args.detailed_time_log{
        let tick_start_index_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let tick_start_index_secs: u64 = tick_start_index_unix.as_secs();
        let tick_start_index_millis: u64 = tick_start_index_unix.subsec_millis() as u64; 
        println!("Tick Start Index Unix time: {}.{}", tick_start_index_secs, tick_start_index_millis);
    };

    // Derive the tick array address & fetch account

    let (tick_array_pubkey, _tick_array_discriminant): (Pubkey, u8) = pda::get_tick_array_address(
        &pool_pubkey, current_tick_array_start_index)
        .expect("Failed to derive tick array address"); 
        // Remark: Once again we need to switch pubkey structs here, there is no class of human I hate more than 
        // the crypto developer.
    // TICK ARRAY PUBKEY FOR SOLANA HERE 
    let tick_array_account: Account = rpc_client.get_account(&tick_array_pubkey).unwrap(); 

    if args.detailed_time_log{
        let tick_array_fetch_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let tick_array_fetch_secs: u64 = tick_array_fetch_unix.as_secs();
        let tick_array_fetch_millis: u64 = tick_array_fetch_unix.subsec_millis() as u64; 
        println!("Tick Array Fetch Unix time: {}.{}", tick_array_fetch_secs, tick_array_fetch_millis);
    };

    // Decode tick array account into TickArrayFacade struct 

    let tick_array: TickArray = TickArray::from_bytes(&tick_array_account.data).unwrap();
    let tick_array_facade: TickArrayFacade = TickArrayFacade::from(tick_array);
    let tick_arrays: TickArrays = TickArrays::One(tick_array_facade);

    if args.detailed_time_log{
        let tick_array_decode_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let tick_array_decode_secs: u64 = tick_array_decode_unix.as_secs();
        let tick_array_decode_millis: u64 = tick_array_decode_unix.subsec_millis() as u64; 
        println!("Tick Array Decode Unix time: {}.{}", tick_array_decode_secs, tick_array_decode_millis);
    };

    // Derive the oracle address & fetch account (w/ error catching)

    let (oracle_pubkey, _oracle_discriminant): (Pubkey, u8) = pda::get_oracle_address(
        &pool_pubkey).unwrap(); 
        // Remark: Again, we need to swap structs. Perhaps there is a brain eating parasite hosted on the blockchain?
    // ORACLE PUBKEY FOR ORCA HERE
    let oracle_account_result: Result<Account, _> = rpc_client.get_account(&oracle_pubkey);

    if args.detailed_time_log{
        let oracle_fetch_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let oracle_fetch_secs: u64 = oracle_fetch_unix.as_secs();
        let oracle_fetch_millis: u64 = oracle_fetch_unix.subsec_millis() as u64; 
        println!("Oracle Fetch Unix time: {}.{}", oracle_fetch_secs, oracle_fetch_millis);
    };

    // Decode oracle account into Option<OracleFacade> struct

    let oracle_account: Option<Account> = match oracle_account_result {
        Ok(acc) => Some(acc),
        Err(err) => {
            eprintln!("failed to fetch oracle account: {}", err);
            None
        }
    };
    let oracle_facade: Option<OracleFacade> = oracle_account.map(|acct| {
        let oracle: Oracle = Oracle::from_bytes(&acct.data).unwrap();
        OracleFacade::from(oracle)
    });

    if args.detailed_time_log{
        let oracle_decode_unix: Duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let oracle_decode_secs: u64 = oracle_decode_unix.as_secs();
        let oracle_decode_millis: u64 = oracle_decode_unix.subsec_millis() as u64; 
        println!("Oracle Decode Unix time: {}.{}", oracle_decode_secs, oracle_decode_millis);
    };

    // Check Facades

    println!("\n=== Full WhirlpoolFacade ===\n{:#?}", whirlpool_facade);
    println!("\n=== Full TickArrayFacade ===\n{:#?}", tick_array_facade);
    println!("\n=== Full OracleFacade ===\n{:#?}", oracle_facade);

    // Get remaining parameters required by swap quote function

    let timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // transfee_fee_a/b vars are cooked, me thinks gas?? pending on dev input.
    let transfer_fee_a: Option<TransferFee> = Some(TransferFee::new(whirlpool_facade.fee_rate));
    let transfer_fee_b: Option<TransferFee> = Some(TransferFee::new(whirlpool_facade.fee_rate));
    let token_out: u64 = args.volume_out;
    let slippage_tolerance_bps: u16 = args.slippage_tolerance;

    let quote_result: Result<ExactInSwapQuote, _> = swap_quote_by_input_token(
        token_out, 
        true, // Specfied token = a flag, not sure how to know what way round yet 
        slippage_tolerance_bps, 
        whirlpool_facade, 
        oracle_facade,
        tick_arrays,
        timestamp,
        None,
        None,
    );

    let quote: Option<ExactInSwapQuote> = match quote_result{
        Ok(quote) => Some(quote), 
        Err(err) => {
            eprintln!("Failed to obtain quote: {}", err);
            None
        }
    };

    println!("\n=== Full ExactOutSwapQuote Result ===\n {:#?}", quote);
}   