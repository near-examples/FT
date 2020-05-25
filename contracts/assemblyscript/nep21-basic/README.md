# Minimal NEP#21 Implementation

This repository includes FT implementations in Rust and AssemblyScript for [NEP#21 - Fungible Token](https://github.com/nearprotocol/NEPs/blob/master/specs/Standards/Tokens/FungibleToken.md)

# Reference-level explanation

The full implementation in Rust can be found there: https://github.com/nearprotocol/near-sdk-rs/blob/master/examples/fungible-token/src/lib.rs

**NOTES**

- All amounts, balances and allowance are limited by `u128` (max value `2**128 - 1`).
- Token standard uses JSON for serialization of arguments and results.
- Amounts in arguments and results have are serialized as Base-10 strings, e.g. `"100"`. This is done to avoid JSON limitation of max integer value of `2**53`.

# Drawbacks

- Current interface doesn't have minting, precision (decimals), naming. But it should be done as extensions, e.g. a Precision extension.
- It's not possible to exchange tokens without transferring them to escrow first.
- It's not possible to transfer tokens to a contract with a single transaction without setting the allowance first. It should be possible if we introduce `transfer_with` function that transfers tokens and calls escrow contract. It needs to handle result of the execution and contracts have to be aware of this API.

# Future possibilities

- Support for multiple token types
- Minting and burning
- Precision, naming and short token name.

# Notable limitations of this implementation

- Anyone can mint tokens (!!)
- No functions to return metadata such as the name or symbol of this FT

# Notable additions that go beyond the specification of NEP#4

`mint_to`: the spec gives no guidance or requirements on how tokens are minted/created/assigned. If this implementation of `mint_to` is close to matching your needs, feel free to ship your FT with only minor modifications (such as caller verification).
