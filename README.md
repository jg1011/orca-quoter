# Orca Quoter

ALL CRATES ARE UNTESTED AND NOT INTENDED FOR PRODUCTION USE

## Usage 

CLI/GUI pending, for now just run a quick (not quick at all, takes like 5-10 mins) 

```
cargo run
```

after implementing your logic in ```crates/cli/src/main.rs``` to test functions. 

## TO-DO

- Finish ```populate_pool_states_via_RPC``` refactor
    - Fetch MintData from ```Vec<WhirlpoolFacade>```, or if mint information missing from Facades use ```Vec<Whirlpool>```
    - Construct timestamp hasmap
- Refactor SwapQuote logic for ```Vec<PoolState>``` struct
- Implement decrease/increase liquidity logic for ```Vec<PoolState>``` struct
- Create CLI prototype
- Implement ```update_pool_states_via_websocket``` logic for live listening
