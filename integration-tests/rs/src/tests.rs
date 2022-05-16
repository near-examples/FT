use serde_json::json;
use near_units::parse_near;
use workspaces::prelude::*; 
use workspaces::{network::Sandbox, Account, Contract, Worker};
use near_sdk::json_types::U128;

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

    // Initialize contracts
    ft_contract.call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
        .transact()
        .await?;
    defi_contract.call(&worker, "new")
        .args_json(serde_json::json!({
            "fungible_token_account_id": ft_contract.id()
        }))?
        .transact()
        .await?;
    defi_contract.as_account().call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": defi_contract.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // begin tests  
    test_total_supply(&owner, &ft_contract, &worker).await?;
    test_simple_transfer(&owner, &alice, &ft_contract, &worker).await?;
    test_can_close_empty_balance_account(&bob, &ft_contract, &worker).await?;
    Ok(())
}

async fn test_total_supply(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("1,000,000,000 N"));
    let res: U128 = owner
                .call(&worker, contract.id(), "ft_total_supply")
                .args_json(json!({}))?
                .transact()
                .await?
                .json()?;
    assert_eq!(res, initial_balance);
    println!("      Passed ✅ test_total_supply");
    Ok(())
}

async fn test_simple_transfer(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount = U128::from(parse_near!("1,000 N"));

    // register user 
    user.call(&worker, contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // transfer ft 
    owner.call(&worker, contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount
        }))?
        .deposit(1)
        .transact()
        .await?;

    let root_balance: U128 = owner.call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner.id()
        }))?
        .transact()
        .await?
        .json()?;
    
    let alice_balance: U128 = owner.call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(root_balance,  U128::from(parse_near!("999,999,000 N")));
    assert_eq!(alice_balance, transfer_amount);

    println!("      Passed ✅ test_simple_transfer");
    Ok(())
}

async fn test_can_close_empty_balance_account(
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // register user 
    user.call(&worker, contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    let result: bool = user.call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await?
        .json()?;

    assert_eq!(result, true);
    println!("      Passed ✅ can_close_empty_balance_account");
    Ok(())
}