use std::time::{
    SystemTime, UNIX_EPOCH
};
use std::fmt;
use std::boxed::{
    Box
};

use orca_whirlpools_core::{
    swap_quote_by_input_token, swap_quote_by_output_token,
    ExactInSwapQuote,    
    ExactOutSwapQuote,   
    WhirlpoolFacade,     
    OracleFacade,   
    TickArrays     
};

use pool::structs::PoolState;

/// Bid‐Ask information for a single "swap" quote against a whirlpool.
///
/// - **ask**: How much token B you must pay to receive _exactly_ `amount_a` units of token A  
///           (i.e. an _exact‐output_ quote).  
/// - **bid**: How much token B you would receive if you _sold_ `amount_a` units of token A  
///           (i.e. an _exact‐input_ quote).
pub struct SwapQuote {
    pub amount:       u64,
    pub slippage_bps:   u16,
    pub bid:            ExactInSwapQuote,
    pub ask:            ExactOutSwapQuote,
}

pub struct SwapParams {
    pub pool: PoolState,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// TryFrom implementation for SwapParams to SwapQuote
///
/// Note: This abuses the stack, still looking into a better approach here. Running in a 8MiB stack thread works, 
/// but otherwise we get stack overflow. (In fact, a 2MiB stack thread is enough for the swap quote to be computed)
/// 
/// 
impl<'a> TryFrom<SwapParams> for SwapQuote {
    type Error = &'static str;

    fn try_from(params: SwapParams) -> Result<Self, Self::Error> {
        let SwapParams {
            pool,
            amount,
            slippage_bps,
        } = params;

        // 1) Convert on-chain types into the core SDK "facade" types
        // Box as early as possible to avoid large stack allocations
        let whirlpool_f_box = Box::new(WhirlpoolFacade::from(pool.whirlpool.clone()));
        let oracle_f_clone = pool.oracle.clone();
        let tick_array_box_1 = Box::new(pool.current_tick_array.clone());
        let tick_array_box_2 = Box::new(pool.current_tick_array.clone());

        // 2) Current UNIX timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system clock error")?
            .as_secs();
        

        let bid = swap_quote_by_input_token(
            amount,        // token_in amount
            true,            // specified_token_a = true
            slippage_bps,
            *whirlpool_f_box,
            oracle_f_clone,
            *tick_array_box_1,
            timestamp,
            None,            // no transfer fee on A
            None,            // no transfer fee on B
        )
        .map_err(|_| "failed to compute bid quote")?;

        // 4) Exact-out (ask): want `amount_a` of A → pay B
        let ask = swap_quote_by_output_token(
            amount,        // token_out amount
            true,            // specified_token_a = true
            slippage_bps,
            *whirlpool_f_box,
            oracle_f_clone,
            *tick_array_box_2,
            timestamp,
            None, 
            None,
        )
        .map_err(|_| "failed to compute ask quote")?;

        Ok(SwapQuote {
            amount,
            slippage_bps,
            bid,
            ask,
        })
    }
}

/// Pretty printing for SwapQuote str
/// Prints as follows if we just run a "println!("{}", quote);"
impl fmt::Display for SwapQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SwapQuote {{")?;
        writeln!(f, "  amount_a:     {}", self.amount)?;
        writeln!(f, "  slippage_bps: {}", self.slippage_bps)?;
        writeln!(f, "  bid:          {:?}", self.bid)?;
        write!(f,   "  ask:          {:?}", self.ask)?;
        writeln!(f, "}}")
    }
}