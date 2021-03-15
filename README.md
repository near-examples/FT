Fungible Token (FT)
===================

Example implementation of a [Fungible Token] contract which uses [near-contract-standards] and [simulation] tests. This is a contract-only example.

  [Fungible Token]: https://nomicon.io/Standards/Tokens/FungibleTokenCore.html
  [near-contract-standards]: https://github.com/near/near-sdk-rs/tree/master/near-contract-standards
  [simulation]: https://github.com/near/near-sdk-rs/tree/master/near-sdk-sim

## Building

Follow instructions for installing Rust here https://docs.near.org/docs/tutorials/contracts/intro-to-rust#3-step-rust-installation (if you're using Gitpod, you can skip this step).

To build run:
```bash
./build.sh
```

## Testing

As with many Rust libraries and contracts, there are tests in the main fungible token implementation at `ft/src/lib.rs`.

Additionally, this project has [simulation] tests in `tests/sim`. Simulation tests allow testing cross-contract calls, which is crucial to ensuring that the `ft_transfer_call` function works properly. These simulation tests are the reason this project has the file structure it does. Note that the root project has a `Cargo.toml` which sets it up as a workspace. `ft` and `test-contract-defi` are both small & focused contract projects, the latter only existing for simulation tests. The root project imports `near-sdk-sim` and tests interaction between these contracts.

You can run all these tests with one command:

```bash
cargo test
```

If you want to run only simulation tests, you can use `cargo test simulation`, since all the simulation tests include "simulation" in their names.


## Notes

 - The maximum balance value is limited by U128 (`2**128 - 1`).
 - JSON calls should pass U128 as a base-10 string. E.g. "100".
 - This does not include escrow functionality, as `ft_transfer_call` provides a superior approach. An escrow system can, of course, be added as a separate contract or additional functionality within this contract.

## No AssemblyScript?

[near-contract-standards] is currently Rust-only. We strongly suggest using this library to create your own Fungible Token contract to ensure it works as expected.

Someday NEAR core or community contributors may provide a similar library for AssemblyScript, at which point this example will be updated to include both a Rust and AssemblyScript version.

## Contributing

When making changes to the files in `ft` or `test-contract-defi`, remember to use `./build.sh` to compile all contracts and copy the output to the `res` folder. If you forget this, **the simulation tests will not use the latest versions**.
