# Shredstream Parser

> **Please ensure that the source of the shred stream is trusted.**

Skips the replay stage and directly parses transactions in the shred stream.

Although the transaction execution results cannot be known and there are some [uncertain risks](#uncertain-risks), downstream programs can receive transactions faster.

## Uncertain risks

- The transaction may fail
- We can only get the amount including slippage, and cannot know the specific transaction results
- Blocks may be rolled back

## Supported protocol

- Pump fun

## TODO

- Pump fun AMM
- Raydium launch lab
