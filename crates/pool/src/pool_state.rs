use std::str::{
    FromStr
};
use std::time::{
    SystemTime, UNIX_EPOCH
};
use std::collections::{
    HashMap
};

use solana_client::{
    rpc_client::RpcClient, client_error::ClientError
};
use solana_sdk::{
    account::Account, pubkey::Pubkey
};

use orca_whirlpools_client::{
    Whirlpool, TickArray, Oracle, 
};
use orca_whirlpools_core::{
    WhirlpoolFacade, TickArrayFacade, TickArrays, OracleFacade,
    get_tick_array_start_tick_index
};

use crate::pda::{
    get_tick_array_address, get_oracle_address
};

use crate::structs::{
    PoolState
};

use mint::{
    structs::MintData, mint::mint_data_from_client_and_pubkey
};


/// Fetches data for PoolState struct and serialises into PoolState struct
/// 
/// Parameters: 
///     - client: The RPC client used to fetch data 
///     - pool_pubkey: The pool's pubkey
/// 
/// Returns: 
///     - The populated PoolState struct
pub fn LEGACY_from_client_and_pubkey_via_RPC(client: &RpcClient, pool_pubkey: &Pubkey) -> PoolState{
    // Fetch whirlpool account with client and serialise into Whirlpool struct 
    let pool_account: Account = client.get_account(pool_pubkey)
        .expect("Failed to fetch whirlpool account.");
    let whirlpool_timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("whirlpool fetch timestamp failed")
        .as_secs();
    let whirlpool: Whirlpool = Whirlpool::from_bytes(&pool_account.data).unwrap();

    // Derive current tick array pubkey
    let whirlpool_facade: WhirlpoolFacade = WhirlpoolFacade::from(whirlpool.clone());
    let current_tick_array_start_index: i32 = get_tick_array_start_tick_index(
        whirlpool_facade.tick_current_index, whirlpool_facade.tick_spacing
    );
    let (tick_array_pubkey, _tick_array_discriminant): (Pubkey, u8) = get_tick_array_address(
        &pool_pubkey, current_tick_array_start_index)
        .expect("Failed to derive tick array address"); 
    
    // Fetch current tick array account and serialise into TickArrays struct 
    let tick_array_account: Account = client.get_account(&tick_array_pubkey).unwrap();
    let tick_array_timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("tick array fetch timestamp failed")
        .as_secs();
    let tick_array: TickArray = TickArray::from_bytes(&tick_array_account.data).unwrap();
    let tick_array_facade: TickArrayFacade = TickArrayFacade::from(tick_array);
    let tick_arrays: TickArrays = TickArrays::One(tick_array_facade);

    // Derive oracle pubkey
    let (oracle_pubkey, _oracle_discriminant): (Pubkey, u8) = get_oracle_address(
    &pool_pubkey).unwrap(); 

    // Fetch oracle account and serialise into Option<Oracle> enum
    let oracle_account_result: Result<Account, _> = client.get_account(&oracle_pubkey);
    let oracle_timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("oracle fetch timestamp failed")
        .as_secs();
    let oracle_account: Option<Account> = match oracle_account_result {
        Ok(acc) => Some(acc),
        Err(err) => {
            eprintln!("failed to fetch oracle account: {}", err);
            None
        }
    };
    let oracle: Option<OracleFacade> = oracle_account.map(|acct| {
        let oracle: Oracle = Oracle::from_bytes(&acct.data).unwrap();
        OracleFacade::from(oracle)
    });

    // Derive mint pubkeys
    let mint_a_pubkey = Pubkey::from_str(&whirlpool.token_mint_a.to_string().as_str())
        .expect("Failed to derive token A mint pubkey");
    let mint_b_pubkey = Pubkey::from_str(&whirlpool.token_mint_b.to_string().as_str())
        .expect("Failed to derive token B mint pubkey");

    // Populate MintData structs
    let mint_a_data: MintData = mint_data_from_client_and_pubkey(
        &client, &mint_a_pubkey 
    ); 
    let mint_a_timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("mint a fetch timestamp failed")
        .as_secs();
    let mint_b_data: MintData = mint_data_from_client_and_pubkey(
        &client, &mint_b_pubkey 
    ); 
    let mint_b_timestamp: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("mint b fetch timestamp failed")
        .as_secs();

    // Construct timestamp hashmap
    let mut timestamps: HashMap<&'static str, u64> = HashMap::new();
    timestamps.insert("whirlpool",   whirlpool_timestamp);
    timestamps.insert("current_tick_array",  tick_array_timestamp);
    timestamps.insert("oracle",      oracle_timestamp);
    timestamps.insert("mint_a",      mint_a_timestamp);
    timestamps.insert("mint_b",      mint_b_timestamp);

    PoolState { 
        whirlpool: whirlpool_facade,
        current_tick_array: tick_arrays,
        oracle: oracle,
        mint_a_data: mint_a_data,
        mint_b_data: mint_b_data,
        timestamps: timestamps
    }
}


/// Populate PoolState struct from RPC client and pool address
/// 
/// Parameters:
///     - client: a pointer to an RPC client
///     - pool_pubkeys: a pointer to a vector of pool addresses
///     - require_all_accounts: a boolean indicating whether to return an error if any pool account is None
///     - require_all_tick_arrays: a boolean indicating whether to return an error if any left/right tick array pubkey is None
///     - fetch_mint_data: a boolean indicating whether to fetch mint data via RPC client
/// 
/// Returns:
///     - A vector of populated PoolState structs
pub fn populate_pool_states(
    client: &RpcClient, pool_pubkeys: &Vec<Pubkey>, require_all_accounts: bool, require_all_tick_arrays: bool, 
    fetch_mint_data: bool
) -> Result<Vec<PoolState>, String> {
    // Verify that there are <= 33 pool pubkeys, get_multiple_accounts works only up to 100 
    // accounts, and we fetch 3 * num_pools when collecting tickarrays 
    if pool_pubkeys.len() > 33 {
        return Err("Too many pool pubkeys, max 33".to_string());
    }

    // Phase 1: Construct Vec<WhirlpoolFacade>

    // Phase 1.1: Fetch Vec<Option<Account>> with RPC client
        // We use the private fn fetch_pool_accounts_via_rpc
    let pool_accounts_result: Result<Vec<Account>, String> = fetch_pool_accounts_via_rpc(
        client, pool_pubkeys, require_all_accounts
    );
        // Failure to fetch pool accounts is a critical error
    if pool_accounts_result.is_err() {
        return Err(pool_accounts_result.err().unwrap());
    }
    let pool_accounts: Vec<Account> = pool_accounts_result.unwrap();

    // Phase 1.2: Construct Vec<WhirlpoolFacade>
        // We use the private fn deserialise_into_whirlpool_facades
    let whirlpool_facades_result: Result<Vec<WhirlpoolFacade>, String> 
        = deserialise_into_whirlpool_facades(
            pool_accounts, require_all_accounts
        );
        // Error Catch
    if whirlpool_facades_result.is_err() {
        return Err(whirlpool_facades_result.err().unwrap());
    }
    let whirlpool_facades: Vec<WhirlpoolFacade> = whirlpool_facades_result.unwrap();

    // Phase 2: Construct Vec<TickArrays>

    // Phase 2.1: Derive left, right, and current tick array pubkeys from Vec<WhirlpoolFacade> and Vec<Pubkey> (pools)#
        // We use the private fn left_right_and_current_tick_array_pubkeys_from_whirlpool_facades
    let tick_array_pubkeys_result: Result<Vec<(Pubkey, Pubkey, Pubkey)>, String> 
        = left_right_and_current_tick_array_pubkeys_from_whirlpool_facades(
            &whirlpool_facades, pool_pubkeys, require_all_tick_arrays
        );
    // Error Catch
    if tick_array_pubkeys_result.is_err() {
        return Err(tick_array_pubkeys_result.err().unwrap());
    }

    // Phase 2.2: Fetch Vec<(Account, Account, Account)> with RPC client
        // We use the private fn left_right_and_current_tick_array_accounts_from_pubkeys_via_rpc
    let tick_array_accounts_result: Result<Vec<(Account, Account, Account)>, String> 
        = left_right_and_current_tick_array_accounts_from_pubkeys_via_rpc(
            client, &tick_array_pubkeys_result.unwrap(), require_all_tick_arrays
        );
    // Error Catch
    if tick_array_accounts_result.is_err() {
        return Err(tick_array_accounts_result.err().unwrap());
    }
    let tick_array_accounts: Vec<(Account, Account, Account)> = tick_array_accounts_result.unwrap();

    // Phase 3: Construct Vec<TickArrays>

    let tick_arrays_result = deserialise_into_tick_arrays(
        tick_array_accounts, require_all_tick_arrays
    );
    if tick_arrays_result.is_err() {
        return Err(tick_arrays_result.err().unwrap());
    }
    let tick_arrays: Vec<TickArrays> = tick_arrays_result.unwrap();

    // TODO: ORACLES AND MINTS, THEN DONE :)

    // Phase 3: Construct Vec<Option<OracleFacade>>  

    // Phase 3.1: Fetch Vec<Option<Account>> via RPC client 
        // We use the private fn fetch_oracles_from_pubkeys_via_rpc
        // If the RPC call fails, use a fallback Vec of `None` with the same length 
        // so downstream logic remains safe.
    let oracle_accounts: Vec<Option<Account>> = match fetch_oracles_from_pubkeys_via_rpc(
        client, pool_pubkeys) {
        Ok(accounts) => accounts,
        Err(err) => {
            eprintln!("Failed to fetch oracle accounts: {}", err);
            vec![None; pool_pubkeys.len()]
        }
    };

    // Phase 3.2: Deserialise Vec<Option<Account>> into Vec<Option<OracleFacade>>
        // We use the private fn deserialise_into_oracle_facades
        // Again, if the deserialisation fails, use a fallback Vec of `None` with the 
        // same length so downstream logic remains safe.
    let oracle_facades: Vec<Option<OracleFacade>> = match deserialise_into_oracle_facades(oracle_accounts) {
        Ok(facades) => facades,
        Err(err) => {
            eprintln!("Failed to deserialize oracle accounts: {}", err);
            vec![None; pool_pubkeys.len()]
        }
    };

    // TODO: Mint data and final PoolState construction

    todo!()
}


/// Private Functions


/// Fetch Vec<Account> for pools via RPC client 
/// 
/// Parameters: 
///     - pool_pubkeys: A pointer to the Vec<Pubkey> struct containing pool pubkeys
///     - require_all_accounts: A bool dictating whether to flag an error if an account is empty
/// 
/// Returns:
///     - A Vec<Account> struct containing the pool accounts or a String type error code
fn fetch_pool_accounts_via_rpc(
    client: &RpcClient, pool_pubkeys: &Vec<Pubkey>, require_all_accounts: bool
) -> Result<Vec<Account>, String> {
    let pool_accounts_response: Result<Vec<Option<Account>>, _> = client.get_multiple_accounts(&pool_pubkeys);
    if pool_accounts_response.is_err() {
        return Err("Failed to fetch pool accounts".to_string());
    }
    let pool_account_options: Vec<Option<Account>> = pool_accounts_response.unwrap();

    // Phase 1.2: Deserialise Vec<Option<Account>> into Vec<Account> 
    let mut pool_accounts: Vec<Account> = Vec::new();
    for (i, pool_account ) in pool_account_options.iter().enumerate() {
        if pool_account.is_some() {
            pool_accounts.push(pool_account.clone().unwrap());
        }
        else {
            if require_all_accounts {
                return Err(format!(
                    "Failed to fetch pool account with address {}",
                    pool_pubkeys[i].to_string()
                ));
            }
            else {
                eprintln!("Failed to fetch pool account with address {}", 
                    pool_pubkeys[i].to_string());
                continue;
            }
        }
    }
    Ok(pool_accounts)
}


/// Deserialise Vec<Account> into Vec<WhirlpoolFacade>
/// 
/// Parameters:
///     - pool_accounts: a vector of Account structs, obtained from RPC client
///     - require_all_accounts: a boolean indicating whether to return an error if any pool account is None
/// 
/// Returns:
///     - A vector of WhirlpoolFacade structs
fn deserialise_into_whirlpool_facades(
    pool_accounts: Vec<Account>, require_all_accounts: bool
) -> Result<Vec<WhirlpoolFacade>, String> {

    let mut whirlpool_facades: Vec<WhirlpoolFacade> = Vec::new();
    for (i, account) in pool_accounts.iter().enumerate() {
        match Whirlpool::from_bytes(&account.data) {
            Ok(whirlpool) => whirlpool_facades.push(WhirlpoolFacade::from(whirlpool)),
            Err(err) => {
                if require_all_accounts {
                    return Err(format!(
                        "Failed to deserialize whirlpool account at index {}: {}",
                        i, err
                    ));
                } else {
                    println!(
                        "Failed to deserialize whirlpool account at index {}: {}",
                        i, err
                    );
                    continue;
                }
            }
        }
    }

    Ok(whirlpool_facades)
}


/// Derive left, right, and current tick array pubkeys from Vec<WhirlpoolFacade> and Vec<Pubkey>
/// 
/// Parameters:
///     - whirlpool_facades: a vector of WhirlpoolFacade structs, obtained from deserialisation of Vec<Account>
///     - pool_pubkeys: a vector of pool pubkeys
///     - require_all_tick_arrays: a boolean indicating whether to return an error if left/right tick array pubkeys are None
/// 
/// Returns:
///     - A vector of tuples, each containing the left, right, and current tick array pubkeys
fn left_right_and_current_tick_array_pubkeys_from_whirlpool_facades(
    whirlpool_facades: &Vec<WhirlpoolFacade>, pool_pubkeys: &Vec<Pubkey>, require_all_tick_arrays: bool
) -> Result<Vec<(Pubkey, Pubkey, Pubkey)>, String> {
     
    // Phase 1: Find start tick idxs 
    let mut start_tick_idxs: Vec<(i32, i32, i32)> = Vec::new();
    for whirlpool_facade in whirlpool_facades.iter() {
        let current_start_tick_idx: i32 = get_tick_array_start_tick_index(
            whirlpool_facade.tick_current_index, whirlpool_facade.tick_spacing
        );
        let i32_tick_spacing: i32 = whirlpool_facade.tick_spacing as i32;
        // Each tick array is 88 ticks wide
        let left_start_tick_idx: i32 = current_start_tick_idx - i32_tick_spacing*88;
        let right_start_tick_idx: i32 = current_start_tick_idx + i32_tick_spacing*88;
        start_tick_idxs.push((left_start_tick_idx, current_start_tick_idx, right_start_tick_idx));
    }

    // Phase 2: Construct Vec<(Pubkey, Pubkey, Pubkey)> of tick array pubkeys
    let mut tick_array_pubkeys: Vec<(Pubkey, Pubkey, Pubkey)> = Vec::new();
    for (i, (left_start_idx, current_start_idx, right_start_idx)) in start_tick_idxs.iter().enumerate() {
        // Derive left tick array pubkey
        let left_pubkey = match get_tick_array_address(&pool_pubkeys[i], *left_start_idx) {
            Ok((pk, _)) => pk,
            Err(err) => {
                if require_all_tick_arrays {
                    return Err(format!(
                        "Failed to derive left tick array address for {}: {}",
                        pool_pubkeys[i], err
                    ));
                } else {
                    println!(
                        "Failed to derive left tick array address for {}: {}",
                        pool_pubkeys[i], err
                    );
                    continue;
                }
            }
        };

        // Derive current tick array pubkey
            // We always return an error if current tick array pubkey is None 
            // as this is a critical error
        let current_pubkey = match get_tick_array_address(
            &pool_pubkeys[i], *current_start_idx
        ) {
            Ok((pk, _)) => pk,
            Err(err) => {
                return Err(format!(
                    "Failed to derive current tick array address for {}: {}",
                    pool_pubkeys[i], err
                ));
            }
        };

        // Derive right tick array pubkey
        let right_pubkey = match get_tick_array_address(&pool_pubkeys[i], *right_start_idx) {
            Ok((pk, _)) => pk,
            Err(err) => {
                if require_all_tick_arrays {
                    return Err(format!(
                        "Failed to derive right tick array address for {}: {}",
                        pool_pubkeys[i], err
                    ));
                } else {
                    println!(
                        "Failed to derive right tick array address for {}: {}",
                        pool_pubkeys[i], err
                    );
                    continue;
                }
            }
        };

        tick_array_pubkeys.push((left_pubkey, current_pubkey, right_pubkey));
    }

    Ok(tick_array_pubkeys)
}


fn left_right_and_current_tick_array_accounts_from_pubkeys_via_rpc(
    client: &RpcClient, tick_array_pubkeys: &Vec<(Pubkey, Pubkey, Pubkey)>, require_all_tick_arrays: bool
) -> Result<Vec<(Account, Account, Account)>, String> {

    // Phase 1: Flatten Vec<(Pubkey, Pubkey, Pubkey)> into Vec<Pubkey>
        // Structured as left_1, current_1, right_1, left_2, current_2, right_2, ...
    let mut flattened_pubkeys: Vec<Pubkey> = Vec::new();
    for (left_pubkey, current_pubkey, right_pubkey) in tick_array_pubkeys.iter() {
        flattened_pubkeys.push(left_pubkey.clone());
        flattened_pubkeys.push(current_pubkey.clone());
        flattened_pubkeys.push(right_pubkey.clone());
    }

    // Phase 2: Fetch Vec<Option<Account>> with RPC client 
        // get_multiple_accounts is order preserving
    let tick_array_accounts_response: Result<Vec<Option<Account>>, _> = client.get_multiple_accounts(&flattened_pubkeys);
    if tick_array_accounts_response.is_err() {
        return Err("Failed to fetch tick array accounts".to_string());
    }
    let tick_array_account_options: Vec<Option<Account>> = tick_array_accounts_response.unwrap();

    // Phase 3: Deserialise Vec<Option<Account>> into Vec<Account>
        // Structured as left_1, current_1, right_1, left_2, current_2, right_2, ...
    let mut tick_array_accounts: Vec<Account> = Vec::new();
    for (i, tick_array_account) in tick_array_account_options.iter().enumerate() {
        if tick_array_account.is_some() {
            tick_array_accounts.push(tick_array_account.clone().unwrap());
        }
        else {
            if require_all_tick_arrays {
                return Err(format!(
                    "Failed to fetch tick array account at index {}: missing account",
                    i
                ));
            }
            else {
                println!(
                    "Failed to fetch tick array account at index {}: missing account",
                    i
                );
                continue;
            }
        }
    }

    // Phase 4: regroup every 3 accounts back into (left, current, right)
    let mut tick_array_account_tuples = Vec::new();
    for chunk in tick_array_accounts.chunks(3) {
        if chunk.len() < 3 { break; } // shouldn't occur, but be safe
        tick_array_account_tuples.push((
            chunk[0].clone(),
            chunk[1].clone(),
            chunk[2].clone(),
        ));
    }

    Ok(tick_array_account_tuples)
}


fn deserialise_into_tick_arrays(
    tick_array_accounts: Vec<(Account, Account, Account)>,
    require_all_tick_arrays: bool,
) -> Result<Vec<TickArrays>, String> {
    let mut result: Vec<TickArrays> = Vec::new();

    for (i, (left_acc, current_acc, right_acc)) in tick_array_accounts.iter().enumerate() {
        // Helper macro to attempt deserialisation with unified error handling.
        macro_rules! try_deser_tick_array {
            ($acc:expr, $side:expr) => {
                match TickArray::from_bytes(&$acc.data) {
                    Ok(ta) => ta,
                    Err(err) => {
                        if require_all_tick_arrays {
                            return Err(format!(
                                "Failed to deserialize {} tick array at index {}: {}",
                                $side, i, err
                            ));
                        } else {
                            println!(
                                "Failed to deserialize {} tick array at index {}: {}",
                                $side, i, err
                            );
                            continue;
                        }
                    }
                }
            };
        }

        let left_ta = try_deser_tick_array!(left_acc, "left");
        let current_ta = try_deser_tick_array!(current_acc, "current");
        let right_ta = try_deser_tick_array!(right_acc, "right");

        // Convert to facades
        let left_facade = TickArrayFacade::from(left_ta);
        let current_facade = TickArrayFacade::from(current_ta);
        let right_facade = TickArrayFacade::from(right_ta);

        result.push(TickArrays::Three(left_facade, current_facade, right_facade));
    }

    Ok(result)
}


/// Derive Oracle addresses & fetch Vec<Option<Account>> via RPC client
/// 
/// Parameters:
///     - client: a pointer to an RPC client
///     - pool_pubkeys: a pointer to avector of pool pubkeys
/// 
/// Returns:
///     - A vector of Option<Account> structs
fn fetch_oracles_from_pubkeys_via_rpc(
    client: &RpcClient,
    pool_pubkeys: &Vec<Pubkey>,
) -> Result<Vec<Option<Account>>, String> {
    // Phase 1: derive oracle pubkeys (Vec<Option<Pubkey>> of same length)
    let mut oracle_pubkeys: Vec<Option<Pubkey>> = Vec::new();
    for (i, pool_pk) in pool_pubkeys.iter().enumerate() {
        match get_oracle_address(pool_pk) {
            Ok((pk, _)) => oracle_pubkeys.push(Some(pk)),
            Err(err) => {
                eprintln!(
                    "Failed to derive oracle pubkey for pool at index {}: {}",
                    i, err
                );
                oracle_pubkeys.push(None);
            }
        }
    }

    // Collect indices & pubkeys where we actually have a key to fetch
    let mut indices: Vec<usize> = Vec::new();
    let mut pubkeys_to_fetch: Vec<Pubkey> = Vec::new();
    for (idx, maybe_pk) in oracle_pubkeys.iter().enumerate() {
        if let Some(pk) = maybe_pk {
            indices.push(idx);
            pubkeys_to_fetch.push(pk.clone());
        }
    }

    // If there are no pubkeys to fetch just return vec of None, saves an RPC call
    if pubkeys_to_fetch.is_empty() {
        eprintln!("No oracle pubkeys to fetch");
        return Ok(vec![None; pool_pubkeys.len()]);
    }

    // Phase 2: fetch all present oracle accounts (order-preserving)
    let accounts_response: Result<Vec<Option<Account>>, _> = client.get_multiple_accounts(&pubkeys_to_fetch);
    if accounts_response.is_err() {
        eprintln!("Failed to fetch oracle accounts");
        return Ok(vec![None; pool_pubkeys.len()]);
    }
    let fetched_accounts = accounts_response.unwrap();

    // Phase 3: reconstruct Vec<Option<Account>> matching pool order
    let mut oracle_accounts: Vec<Option<Account>> = vec![None; pool_pubkeys.len()];
    for (pos, fetched_opt) in indices.iter().zip(fetched_accounts.into_iter()) {
        oracle_accounts[*pos] = fetched_opt;
    }

    Ok(oracle_accounts)
}


/// Deserialise Vec<Option<Account>> into Vec<Option<OracleFacade>>
/// 
/// Parameters:
///     - oracle_accounts: a vector of Option<Account> structs, obtained from RPC client
/// 
/// Returns:
///     - A vector of Option<OracleFacade> structs
fn deserialise_into_oracle_facades(
    oracle_accounts: Vec<Option<Account>>
) -> Result<Vec<Option<OracleFacade>>, String> {
    let mut oracle_facades: Vec<Option<OracleFacade>> = Vec::new();
    for (i, account_opt) in oracle_accounts.iter().enumerate() {
        match account_opt {
            Some(acc) => {
                match Oracle::from_bytes(&acc.data) {
                    Ok(oracle) => oracle_facades.push(Some(OracleFacade::from(oracle))),
                    Err(err) => {
                        eprintln!(
                            "Failed to deserialize oracle account at index {}: {}",
                            i, err
                        );
                        oracle_facades.push(None);
                    }
                }
            }
            None => oracle_facades.push(None),
        }
    }

    Ok(oracle_facades)
}