    // Entrypoint // 

use solana_client::{
    rpc_client::RpcClient
};

use solana_sdk::{
    commitment_config::CommitmentConfig, account::Account, pubkey::Pubkey
};

use std::{
    str::FromStr
};

mod mints;

fn main(){
    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=f05f8738-4aff-41ed-9eaa-d5596a4fc955";
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let pool_address = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";
    let pool_pubkey = Pubkey::from_str(pool_address).expect("Failed to derive pubkey from string");
    let (mint_a_pubkey, mint_b_pubkey) = mints::fetch_pool_mint_pubkeys(&rpc_client, pool_pubkey);
    let mint_b_account = rpc_client.get_account(&mint_b_pubkey).unwrap();
    mints::write_mint_data_to_json(mint_b_account);
}