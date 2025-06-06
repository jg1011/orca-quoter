# Orca Quoter

ALL CRATES ARE UNTESTED AND NOT INTENDED FOR PRODUCTION USE

## Usage 

CLI/GUI pending, for now just run a 

```
cargo build --release
```

and then a simple python wrapper to run the binary should suffice. One could also use the crates as a dependency, but consider my prior warning.

## Roadmap 

- Add decimal information for quote
- Release CLI prototype 
- Wrap quoter in an async for continuous quote streams
    - This logic is given elsewhere in a proprietary project, will release it here later because its easy
- Add LP quotes
- Add reward quotes 
- Incorporate blockchain data to quotes, e.g. token transfer fees & block height 
- Switch to websocket logic 
    - This logic is given elsewhere in a proprietary project, will be released here when no longer utilised. 