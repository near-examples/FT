use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
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
    let owner = worker.root_account().unwrap();
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
    let charlie = owner
        .create_subaccount(&worker, "charlie")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let dave = owner
        .create_subaccount(&worker, "dave")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    ft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
        .transact()
        .await?;
    defi_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "fungible_token_account_id": ft_contract.id()
        }))?
        .transact()
        .await?;
    defi_contract
        .as_account()
        .call(&worker, ft_contract.id(), "storage_deposit")
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
    test_close_account_non_empty_balance(&alice, &ft_contract, &worker).await?;
    test_close_account_force_non_empty_balance(&alice, &ft_contract, &worker).await?;
    test_transfer_call_with_burned_amount(&owner, &charlie, &ft_contract, &defi_contract, &worker)
        .await?;
    test_simulate_transfer_call_with_immediate_return_and_no_refund(
        &owner,
        &ft_contract,
        &defi_contract,
        &worker,
    )
    .await?;
    test_transfer_call_when_called_contract_not_registered_with_ft(
        &owner,
        &dave,
        &ft_contract,
        &worker,
    )
    .await?;
    test_transfer_call_promise_panics_for_a_full_refund(&owner, &alice, &ft_contract, &worker)
        .await?;
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
    owner
        .call(&worker, contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount
        }))?
        .deposit(1)
        .transact()
        .await?;

    let root_balance: U128 = owner
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner.id()
        }))?
        .transact()
        .await?
        .json()?;

    let alice_balance: U128 = owner
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(root_balance, U128::from(parse_near!("999,999,000 N")));
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

    let result: bool = user
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await?
        .json()?;

    assert_eq!(result, true);
    println!("      Passed ✅ test_can_close_empty_balance_account");
    Ok(())
}

async fn test_close_account_non_empty_balance(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    match user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await
    {
        Ok(_result) => {
            panic!("storage_unregister worked despite account being funded")
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string
                .contains("Can't unregister the account with the positive balance without force")
            {
                panic!("storage_unregister with balance displays unexpected error message")
            }
            println!("      Passed ✅ test_close_account_non_empty_balance");
        }
    }
    Ok(())
}

async fn test_close_account_force_non_empty_balance(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let result: CallExecutionDetails = user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    assert_eq!(true, result.is_success());
    assert_eq!(
        result.logs()[0],
        format!(
            "Closed @{} with {}",
            user_with_funds.id(),
            parse_near!("1,000 N") // alice balance from above transfer_amount
        )
    );
    println!("      Passed ✅ test_close_account_force_non_empty_balance");
    Ok(())
}

async fn test_transfer_call_with_burned_amount(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount_str = parse_near!("1,000,000 N").to_string();
    let ftc_amount_str = parse_near!("1,000 N").to_string();

    // register user
    owner
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // transfer ft
    owner
        .call(&worker, ft_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount_str
        }))?
        .deposit(1)
        .transact()
        .await?;

    user.call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": ftc_amount_str,
            "msg": "0",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let storage_result: CallExecutionDetails = user
        .call(&worker, ft_contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    // assert new state
    assert_eq!(
        storage_result.logs()[0],
        format!(
            "Closed @{} with {}",
            user.id(),
            parse_near!("999,000 N") // balance after defi ft transfer
        )
    );

    let total_supply: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(total_supply, U128::from(parse_near!("999,000,000 N")));

    let defi_balance: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(defi_balance, U128::from(parse_near!("999,000,000 N")));

    println!("      Passed ✅ test_transfer_call_with_burned_amount");
    Ok(())
}

async fn test_simulate_transfer_call_with_immediate_return_and_no_refund(
    owner: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount: u128 = parse_near!("100,000,000 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": amount_str,
            "msg": "take-my-money"
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let owner_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    assert_eq!(owner_before_balance.0 - amount, owner_after_balance.0);
    assert_eq!(defi_before_balance.0 + amount, defi_after_balance.0);
    println!("      Passed ✅ test_simulate_transfer_call_with_immediate_return_and_no_refund");
    Ok(())
}

async fn test_transfer_call_when_called_contract_not_registered_with_ft(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount = parse_near!("10 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id":  owner.id()}))?
        .transact()
        .await?
        .json()?;
    let user_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;

    match owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": amount_str,
            "msg": "take-my-money",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
    {
        Ok(res) => {
            panic!("Was able to transfer FT to an unregistered account");
        }
        Err(err) => {
            let owner_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id":  owner.id()}))?
                .transact()
                .await?
                .json()?;
            let user_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id": user.id()}))?
                .transact()
                .await?
                .json()?;
            assert_eq!(user_before_balance, user_after_balance);
            assert_eq!(owner_before_balance, owner_after_balance);
            println!(
                "      Passed ✅ test_transfer_call_when_called_contract_not_registered_with_ft"
            );
        }
    }
    Ok(())
}

async fn test_transfer_call_promise_panics_for_a_full_refund(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount = parse_near!("10 N");

    // register user
    owner
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id":  owner.id()}))?
        .transact()
        .await?
        .json()?;
    let user_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;

    match owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": amount,
            "msg": "no parsey as integer big panic oh no",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
    {
        Ok(res) => {
            panic!("Did not expect for trx to accept invalid paramenter data types")
        }
        Err(err) => {
            let owner_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id":  owner.id()}))?
                .transact()
                .await?
                .json()?;
            let user_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id": user.id()}))?
                .transact()
                .await?
                .json()?;
            assert_eq!(owner_before_balance, owner_after_balance);
            assert_eq!(user_before_balance, user_after_balance);
            println!("      Passed ✅ test_transfer_call_promise_panics_for_a_full_refund");
        }
    }
    Ok(())
}
