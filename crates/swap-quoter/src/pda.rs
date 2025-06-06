use solana_program::program_error::ProgramError;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn parse_whirlpool_master_pubkey() -> Pubkey {
    Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")
        .expect("invalid Pubkey string")
}

// These functions exist in the orca rust-sdk, but use the solana-program::pubkey::Pubkey struct 

// There are key functional differences between the two, and it ends up being easier to rewrite the functions 
// for the solana-sdk::pubkey::Pubkey struct than to convert between the two

pub fn get_tick_array_address(
    whirlpool: &Pubkey,
    start_tick_index: i32,
) -> Result<(Pubkey, u8), ProgramError> {
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[
        b"tick_array",
        whirlpool.as_ref(),
        start_tick_index_str.as_bytes(),
    ];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    Pubkey::try_find_program_address(seeds, &whirlpool_master_pubkey).ok_or(ProgramError::InvalidSeeds)
}

pub fn get_oracle_address(whirlpool: &Pubkey) -> Result<(Pubkey, u8), ProgramError> {
    let seeds = &[b"oracle", whirlpool.as_ref()];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    Pubkey::try_find_program_address(seeds, &whirlpool_master_pubkey).ok_or(ProgramError::InvalidSeeds)
}




