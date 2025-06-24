/// Mint related utils ///
/// 
use std::fmt;

use solana_client::{
    rpc_client::RpcClient
};
use solana_sdk::{
    pubkey::Pubkey, account::Account, program_pack::Pack, program_option::COption
};
use spl_token::state::{
    Mint
}; 

use crate::structs::MintData;

///  Populates, via given RPC client, and serialises MintData struct from the mint's pubkey
/// 
/// Parameters: 
///     - client: The RPC client used for population 
///     - mint_pubkey: The pubkey of the mint used
/// 
/// Returns: 
///     - A populated MintData struct
pub fn mint_data_from_client_and_pubkey(client: &RpcClient, mint_pubkey: &Pubkey) -> MintData{
    // Fetch account with client & serialise into Mint struct
    let account: Account = client.get_account(&mint_pubkey).unwrap(); 
    let state: Mint = Mint::unpack(&account.data)
        .expect("Failed to deserialize SPL‐Token Mint account");

    // Convert authorities to Option<String> structs
    let authority: Option<String> = match state.mint_authority {
        COption::Some(pk) => Some(pk.to_string()),
        COption::None     => None,
    };
    let freeze_authority: Option<String> = match state.freeze_authority {
        COption::Some(pk) => Some(pk.to_string()),
        COption::None     => None,
    };

    MintData {
        pubkey: mint_pubkey.clone(),
        authority: authority,
        supply: state.supply,
        decimals: state.decimals,
        is_initialized: state.is_initialized,
        freeze_authority: freeze_authority,
    }
}