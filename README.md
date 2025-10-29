# Fungible Token (FT)

[![](https://img.shields.io/badge/⋈%20Examples-Basics-green)](https://docs.near.org/tutorials/welcome)
[![](https://img.shields.io/badge/Contract-Rust-red)](contract-rs)

This repository contains an example implementation of a [fungible token](https://docs.near.org/primitives/ft/standard) contract in Rust which uses [near-contract-standards] and [sandobox testing](https://docs.near.org/smart-contracts/testing/integration-test).

[near-contract-standards]: https://github.com/near/near-sdk-rs/tree/master/near-contract-standards

<br />

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
# Create a new account
cargo near create-account <contract-account-id> --useFaucet

# Deploy the contract on it
cargo near deploy <contract-account-id>

# Initialize the contract
near call <contract-account-id> new '{"owner_id": "<contract-account-id>", "total_supply": "1000000000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}' --accountId <contract-account-id>
```

## Basic methods
```bash
# View metadata
near view <contract-account-id> ft_metadata

# Make a storage deposit
near call <contract-account-id> storage_deposit '' --accountId <account-id> --amount 0.00125

# View balance
near view <contract-account-id> ft_balance_of '{"account_id": "<account-id>"}'

# Transfer tokens
near call <contract-account-id> ft_transfer '{"receiver_id": "<account-id>", "amount": "19"}' --accountId <contract-account-id> --amount 0.000000000000000000000001
```

## Notes

 - The maximum balance value is limited by U128 (`2**128 - 1`).
 - JSON calls should pass U128 as a base-10 string. E.g. "100".
 - This does not include escrow functionality, as `ft_transfer_call` provides a superior approach. An escrow system can, of course, be added as a separate contract or additional functionality within this contract.

## Useful Links

- [NEAR Documentation](https://docs.near.org)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)

- [Smart Contracts Docs](https://docs.near.org/smart-contracts/anatomy)
- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Iteract with NEAR blockchain from command line
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
