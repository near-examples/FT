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
- It's not possible to transfer tokens to a contract with a single transaction without setting the allowance first. It should be possible if we introduce transfer_with function that transfers tokens and calls escrow contract. It needs to handle result of the execution and contracts have to be aware of this API.

# Future possibilities

- Support for multiple token types
- Minting and burning
- Precision, naming and short token name.

# Notable limitations of this implementation

- Anyone can mint tokens (!!) until the supply is maxed out
- You cannot give another account escrow access to a limited set of your tokens; an escrow must be trusted with all of your tokens or none at all
- You cannot name more than one account as an escrow
- No functions to return the maximum or current supply of tokens
- No functions to return metadata such as the name or symbol of this NFT
- No functions (or storage primitives) to find all tokens belonging to a given account
- Usability issues: some functions (`revoke_access`, `transfer`, `get_token_owner`) do not verify that they were given sensible inputs; if given non-existent keys, the errors they throw will not be very useful

Still, if you track some of this information in an off-chain database, these limitations may be acceptable for your needs. In that case, this implementation may help reduce gas and storage costs.

# Notable additions that go beyond the specification of NEP#4

`mint_to`: the spec gives no guidance or requirements on how tokens are minted/created/assigned. If this implementation of `mint_to` is close to matching your needs, feel free to ship your NFT with only minor modifications (such as caller verification). If you'd rather go with a strategy such as minting the whole supply of tokens upon deploy of the contract, or something else entirely, you may want to drastically change this behavior.
