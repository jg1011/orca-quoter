// PDA utils ///

use solana_program::program_error::ProgramError;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;


// Quick function to get the whirlpool master pubkey
pub fn parse_whirlpool_master_pubkey() -> Pubkey {
    Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap()
}

// The functions below exist in the orca rust-sdk, but use the solana-program::pubkey::Pubkey struct 
    // There are key functional differences between the two, and it ends up being easier to rewrite 
    // the functions for the solana-sdk::pubkey::Pubkey struct than to convert between the two

/// Parameters: 
///     - pool_pubkey: Pointer to the pool's pubkey
///     - start_tick_index: The first tick in the tick array
/// 
/// Returns: 
///     - A tuple containing the tick array's pubkey and the discriminant or an error
pub fn get_tick_array_address(
    pool_pubkey: &Pubkey,
    start_tick_index: i32,
) -> Result<(Pubkey, u8), ProgramError> {
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[
        b"tick_array",
        pool_pubkey.as_ref(),
        start_tick_index_str.as_bytes(),
    ];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    Pubkey::try_find_program_address(seeds, &whirlpool_master_pubkey).ok_or(ProgramError::InvalidSeeds)
}


/// Parameters: 
///     - pool_pubkey: Pointer to the pool's pubkey
/// 
/// Returns: 
///     - A tuple containing the oracle's pubkey and the discriminant or an error
pub fn get_oracle_address(pool_pubkey: &Pubkey) -> Result<(Pubkey, u8), ProgramError> {
    let seeds = &[b"oracle", pool_pubkey.as_ref()];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    Pubkey::try_find_program_address(seeds, &whirlpool_master_pubkey).ok_or(ProgramError::InvalidSeeds)
}




