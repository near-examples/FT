use serde_json::json;
use near_units::parse_near;
use workspaces::prelude::*; 
use workspaces::{network::Sandbox, Account, Contract, Worker};

const DEFI_WASM_FILEPATH: &str = "../../res/defi.wasm";
const FT_WASM_FILEPATH: &str = "../../res/fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt 
    let worker = workspaces::sandbox().await?;

    // deploy contracts 
    let defi_wasm = std::fs::read(DEFI_WASM_FILEPATH)?;
    let defi_contract = worker.dev_deploy(&defi_wasm).await?;
    let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
    let ft_contract = worker.dev_deploy(&ft_wasm).await?;

    // create accounts
    let owner = worker.root_account();
    let alice = owner
    .create_subaccount(&worker, "alice")
    .initial_balance(parse_near!("30 N"))
    .transact()
    .await?
    .into_result()?;
    let bob = owner
    .create_subaccount(&worker, "bob")
    .initial_balance(parse_near!("30 N"))
    .transact()
    .await?
    .into_result()?;

    // begin tests  
    test_a(&alice, &ft_contract, &worker).await?;
    // test_b(&alice, &owner, &contract, &worker).await?;
    Ok(())
}   

async fn test_a(
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    println!("      Passed ✅ test 1");
    Ok(())
}