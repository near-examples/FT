pub mod common;

use near_sdk::{json_types::U128, NearToken};

use common::{init_accounts, init_contracts, ONE_YOCTO};

#[tokio::test]
async fn storage_deposit_not_enough_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let new_account = ft_contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = ft_contract.view_account().await?.balance;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);
    let res = new_account
        .call(ft_contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(minimal_deposit.saturating_sub(NearToken::from_yoctonear(1)))
        .transact()
        .await?;
    assert!(res.is_failure());

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            new_account_balance_before_deposit.saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    // new_account is charged the transaction fee, so it should loose some NEAR
    assert!(new_account_balance_diff > NearToken::from_near(0));
    assert!(new_account_balance_diff < NearToken::from_millinear(5));

    let contract_balance_diff = ft_contract
        .view_account()
        .await?
        .balance
        .saturating_sub(contract_balance_before_deposit);
    // Any gas reward must stay within the transaction gas budget.
    assert!(contract_balance_diff < NearToken::from_millinear(5));

    Ok(())
}

#[tokio::test]
async fn storage_deposit_minimal_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let new_account = ft_contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = ft_contract.view_account().await?.balance;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);
    new_account
        .call(ft_contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(minimal_deposit)
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            new_account_balance_before_deposit
                .saturating_sub(minimal_deposit)
                .saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)),
        "balance difference: {} yocto, deposit: {} yocto; before: {}, after: {}",
        new_account_balance_diff.as_yoctonear(),
        minimal_deposit.as_yoctonear(),
        new_account_balance_before_deposit.as_yoctonear(),
        new_account.view_account().await?.balance.as_yoctonear()
    );

    let contract_balance_diff = ft_contract
        .view_account()
        .await?
        .balance
        .saturating_sub(contract_balance_before_deposit);
    // Registration retains the minimum deposit; any gas reward is protocol-dependent.
    assert!(contract_balance_diff >= minimal_deposit);
    // Allow for the transaction gas budget rather than a fixed gas-price estimate.
    assert!(contract_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)));

    Ok(())
}

#[tokio::test]
async fn storage_deposit_refunds_excessive_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);

    // Check the storage balance bounds to make sure we have the right minimal deposit
    //
    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceBounds {
        min: U128,
        max: U128,
    }
    let storage_balance_bounds: StorageBalanceBounds = ft_contract
        .call("storage_balance_bounds")
        .view()
        .await?
        .json()?;
    assert_eq!(
        storage_balance_bounds.min,
        minimal_deposit.as_yoctonear().into()
    );
    assert_eq!(
        storage_balance_bounds.max,
        minimal_deposit.as_yoctonear().into()
    );

    // Check that a non-registered account does not have storage balance
    //
    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceOf {
        total: U128,
        available: U128,
    }
    let storage_balance_bounds: Option<StorageBalanceOf> = ft_contract
        .call("storage_balance_of")
        .args_json(near_sdk::serde_json::json!({"account_id": "non-registered-account"}))
        .view()
        .await?
        .json()?;
    assert!(storage_balance_bounds.is_none());

    // Create a new account and deposit some NEAR to cover the storage
    //
    let new_account = ft_contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = ft_contract.view_account().await?.balance;

    new_account
        .call(ft_contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;

    // The expected storage balance should be the minimal deposit,
    // the balance of the account should be reduced by the deposit,
    // and the contract should gain the deposit.
    //
    let storage_balance_bounds: StorageBalanceOf = ft_contract
        .call("storage_balance_of")
        .args_json(near_sdk::serde_json::json!({"account_id": new_account.id()}))
        .view()
        .await?
        .json()?;
    assert_eq!(
        storage_balance_bounds.total,
        minimal_deposit.as_yoctonear().into()
    );
    assert_eq!(storage_balance_bounds.available, 0.into());

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            new_account_balance_before_deposit
                .saturating_sub(minimal_deposit)
                .saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)),
        "balance difference: {} yocto, deposit: {} yocto; before: {}, after: {}",
        new_account_balance_diff.as_yoctonear(),
        minimal_deposit.as_yoctonear(),
        new_account_balance_before_deposit.as_yoctonear(),
        new_account.view_account().await?.balance.as_yoctonear()
    );

    let contract_balance_diff = ft_contract
        .view_account()
        .await?
        .balance
        .saturating_sub(contract_balance_before_deposit);
    // Registration retains the minimum deposit; any gas reward is protocol-dependent.
    assert!(contract_balance_diff >= minimal_deposit);
    assert!(contract_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)));

    Ok(())
}

#[tokio::test]
async fn close_account_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let res = alice
        .call(ft_contract.id(), "storage_unregister")
        .args_json((Option::<bool>::None,))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.json::<bool>()?);

    Ok(())
}

#[tokio::test]
async fn close_account_non_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let res = ft_contract
        .call("storage_unregister")
        .args_json((Option::<bool>::None,))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(format!("{:?}", res)
        .contains("Can't unregister the account with the positive balance without force"));

    let res = ft_contract
        .call("storage_unregister")
        .args_json((Some(false),))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(format!("{:?}", res)
        .contains("Can't unregister the account with the positive balance without force"));

    Ok(())
}

#[tokio::test]
async fn close_account_force_non_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let res = ft_contract
        .call("storage_unregister")
        .args_json((Some(true),))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = ft_contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?.0, 0);

    Ok(())
}
