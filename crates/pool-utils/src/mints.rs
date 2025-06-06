use solana_sdk::{
    pubkey::Pubkey, account::Account, program_pack::Pack, program_option::COption
};

use solana_client::{
    rpc_client::RpcClient
};

use orca_whirlpools_client::{
    Whirlpool
};

use spl_token::state::{
    Mint
}; 

use std::{
    str::FromStr
};

use serde::Serialize;
use serde_json;
use std::fs;

#[derive(Serialize)]
struct MintJson {
    mint_authority:   Option<String>,
    supply:           u64,
    decimals:         u8,
    is_initialized:   bool,
    freeze_authority: Option<String>,
}

pub fn fetch_pool_mint_pubkeys(
    rpc_client: &RpcClient, pool_pubkey: Pubkey
) -> (Pubkey, Pubkey ) {
    // Fetch raw whirlpool data (full struct, not the incomplete facade)
    let pool_account: Account = rpc_client.get_account(&pool_pubkey).unwrap(); 
    let raw_whirlpool: Whirlpool =
        Whirlpool::from_bytes(&pool_account.data).expect("failed to parse Whirlpool");
    // Derive solana_sdk::pubkey::Pubkey struct from solana_program::pubkey::Pubkey struct
    let mint_a_pubkey = Pubkey::from_str(raw_whirlpool.token_mint_a.to_string().as_str())
        .expect("Failed to derive token A mint pubkey");
    let mint_b_pubkey = Pubkey::from_str(raw_whirlpool.token_mint_b.to_string().as_str())
        .expect("Failed to derive token B mint pubkey");
    (mint_a_pubkey, mint_b_pubkey)
}

// Shout out to OpenAI's o4-mini-high model for providing most of the logic here
pub fn write_mint_data_to_json(mint_account: Account){
    // Unpack account data into Mint struct
    let mint_state: Mint = Mint::unpack(&mint_account.data)
        .expect("Failed to deserialize SPL‐Token Mint account");

    // Convert authorities to Option<String> structs for JSON  
    let mint_authority: Option<String> = match mint_state.mint_authority {
        COption::Some(pk) => Some(pk.to_string()),
        COption::None     => None,
    };
    let freeze_authority: Option<String> = match mint_state.freeze_authority {
        COption::Some(pk) => Some(pk.to_string()),
        COption::None     => None,
    };

    // Serialise into json struct 
        // Remark: We use American (wrong) English by convention
    let to_serialize = MintJson {
        mint_authority,
        supply:           mint_state.supply,
        decimals:         mint_state.decimals,
        is_initialized:   mint_state.is_initialized,
        freeze_authority,
    };

    // Convert to a pretty json string
    let json_str = serde_json::to_string_pretty(&to_serialize)
        .expect("Failed to serialize Mint data to JSON");

    // Write to disk 
    fs::write("mint_data.json", json_str).expect("Failed to write mint_data.json");
}