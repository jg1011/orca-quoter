/// TODO 
///     - accept flag at runtime with RPC url 
///     - Initialise RPC client 
/// 

use std::str::{
    FromStr
};
use std::convert::{
    TryInto
};

use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey
}; 
use solana_client::{
    rpc_client::RpcClient
};

use::pool::{
    structs::PoolState, pool_state::LEGACY_from_client_and_pubkey_via_RPC
};
use::swap::{
    structs::SwapQuote, structs::SwapParams
};

use clap::{
    Parser
};

/// CLI arguments struct
/// 
/// Parameters:
///     - rpc_url: the RPC url to use for on-chain data fetching
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // - It is recommended to use a custom RPC url for better performance
    // - For example, use a custom RPC url from a service like Infura, Alchemy, etc.
    #[arg(long="rpc-url", default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: String,
}


/// CLI entry point
fn main(){
    // Parse runtime CLI args 
    let args = Args::parse();
    let rpc_url = args.rpc_url;

    // Initialise RPC client
        // *rpc_url converts String -> static str
    let rpc_client = RpcClient::new_with_commitment(&*rpc_url, CommitmentConfig::confirmed());


    let pool_address = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";
    let pool_pubkey = Pubkey::from_str(pool_address).expect("Failed to derive pubkey from string");
    let pool_state: PoolState = LEGACY_from_client_and_pubkey_via_RPC(&rpc_client, &pool_pubkey);
    println!("{}", pool_state);
        
    let params = SwapParams {
        pool: pool_state,
        amount: 1_000_000,
        slippage_bps: 50,
    };

    // 4MiB stack thread, default is 1MiB which is insufficient.
    let handle = std::thread::Builder::new()
        .name("quote".into())
        .stack_size(4 * 1024 * 1024)        
        .spawn(move || {
            let quote: SwapQuote = params.try_into()
                .expect("failed to build swap quote");
            println!("Full quote:\n{}", quote);
        })
        .expect("failed to spawn thread");

    handle.join().expect("quote thread panicked");
}


/// Generate Vec<PoolState> from RPC client and pool address vector
/// 
/// Parameters:
///     - rpc_client: a pointer to an RPC client
///     - pool_addresses: a pointer to the vector of pool addresses
/// 
/// Returns:
///     - A vector of the populated PoolState structs corresponding to the pool addresses
/// 
/// Note: This is a synchronous function, so it will block the main thread.
fn LEGACY_populate_pool_states_via_RPC(
    rpc_client: &RpcClient, pool_addresses: &Vec<Pubkey>
) -> Vec<PoolState> {
    let mut pool_states: Vec<_> = Vec::new();
    for pool_address in pool_addresses {
        let pool_state: PoolState = LEGACY_from_client_and_pubkey_via_RPC(rpc_client, pool_address);
        pool_states.push(pool_state);
    }
    pool_states
}