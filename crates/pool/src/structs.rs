use std::fmt::{self, Display};
use std::collections::{
    HashMap
};
use std::time::{
    UNIX_EPOCH, Duration
};

use orca_whirlpools_core::{
    TickArrays, OracleFacade, WhirlpoolFacade
};

use mint::{
   structs::MintData
};

pub struct PoolState {
    pub whirlpool: WhirlpoolFacade, 
    pub current_tick_array: TickArrays,
    pub oracle: Option<OracleFacade>,
    pub mint_a_data: MintData,
    pub mint_b_data: MintData,
    pub timestamps: HashMap<&'static str, u64>
}

/// Pretty printing implementation for PoolState
/// Prints as follows if we just run a "println!("{}", pool_state);"
impl Display for PoolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== PoolState ===")?;

        // Print full Whirlpool struct via Debug
        writeln!(f, "\nWhirlpool:\n{:#?}", self.whirlpool)?;

        // Print full current tick array (may be large)
        writeln!(f, "\nCurrent Tick Array:\n{:#?}", self.current_tick_array)?;

        // 3) Oracle: address or “not found”
        writeln!(
            f,
            "\nOracle: {}",
            match &self.oracle {
                Some(_) => "NEED TO DEAL W/ ORACLE, CANNY BE ARSED",
                None => "oracle not found".into(),
            }
        )?;

        // 4) Mint A
        writeln!(f, "\nMint A Data:")?;
        writeln!(f, "{}", self.mint_a_data)?;

        // 5) Mint B
        writeln!(f, "\nMint B Data:")?;
        writeln!(f, "{}", self.mint_b_data)?;

        // 6) Timestamps converted to ISO‐style datetime
        writeln!(f, "\nTimestamps:")?;
        for (label, ts) in &self.timestamps {
            // Interpret the u64 as seconds since UNIX_EPOCH:
            let time = UNIX_EPOCH
                .checked_add(Duration::from_secs(*ts))
                .unwrap_or(UNIX_EPOCH);
            // Debug‐print it; you’ll get something like:
            //   SystemTime { tv_sec: 1_629_163_200, tv_nsec: 0 }
            writeln!(f, "  {}: {:?}", label, time)?;
        }
        Ok(())
    }
}
