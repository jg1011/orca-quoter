use std::fmt;

use solana_sdk::pubkey::Pubkey;

pub struct MintData {
    pub pubkey: Pubkey,
    pub authority:   Option<String>,
    pub supply:           u64,
    pub decimals:         u8,
    pub is_initialized:   bool,
    pub freeze_authority: Option<String>,
}

/// Pretty printing implementation for MintData
///     - LLM generated code, works just fine was too lazy to write it myself
impl fmt::Display for MintData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  pubkey:           {}", self.pubkey)?;
        writeln!(
            f,
            "  authority:        {}",
            self.authority.as_deref().unwrap_or("None")
        )?;
        writeln!(f, "  supply:           {}", self.supply)?;
        writeln!(f, "  decimals:         {}", self.decimals)?;
        writeln!(f, "  is_initialized:   {}", self.is_initialized)?;
        write!(
            f,
            "  freeze_authority: {}",
            self.freeze_authority.as_deref().unwrap_or("None")
        )
    }
}