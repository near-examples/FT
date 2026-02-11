pub mod common;

use near_api::{types::json::U128, NearToken};
use near_sdk::serde_json::json;

#[tokio::test]
async fn storage_deposit_not_enough_deposit() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let new_account = common::create_subaccount(&sandbox, "new-account.sandbox")
        .await
        .unwrap();

    let new_account_balance_before_deposit = new_account
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;
    let contract_balance_before_deposit = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);

    // New account deposits storage
    ft_contract
        .call_function("storage_deposit", json!({"account_id": new_account.account_id(), "registration_only": Option::<bool>::None}))
        .transaction()
        .deposit(minimal_deposit.saturating_sub(NearToken::from_yoctonear(1)))
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_failure();

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        new_account
            .tokens()
            .near_balance()
            .fetch_from(&sandbox_network)
            .await?
            .total,
    );
    // new_account is charged the transaction fee, so it should loose some NEAR
    assert!(new_account_balance_diff > NearToken::from_near(0));
    assert!(new_account_balance_diff < NearToken::from_millinear(1));

    let contract_balance_diff = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total
        .saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so it should gain some NEAR
    assert!(contract_balance_diff > NearToken::from_near(0));
    assert!(contract_balance_diff < NearToken::from_yoctonear(30_000_000_000_000_000_000));

    Ok(())
}

#[tokio::test]
async fn storage_deposit_minimal_deposit() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let new_account = common::create_subaccount(&sandbox, "new-account.sandbox")
        .await
        .unwrap();

    let new_account_balance_before_deposit = new_account
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;
    let contract_balance_before_deposit = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);
    ft_contract
        .call_function("storage_deposit", json!({"account_id": new_account.account_id(), "registration_only": Option::<bool>::None}))
        .transaction()
        .deposit(minimal_deposit)
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        new_account
            .tokens()
            .near_balance()
            .fetch_from(&sandbox_network)
            .await?
            .total,
    );
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(1))
    );

    let contract_balance_diff = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total
        .saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so the difference should be slightly more than minimal_deposit
    assert!(contract_balance_diff > minimal_deposit);
    // adjust the upper limit of the assertion to be more flexible for small variations in the gas reward received
    assert!(
        contract_balance_diff
            < minimal_deposit.saturating_add(NearToken::from_yoctonear(50_000_000_000_000_000_000))
    );

    Ok(())
}

#[tokio::test]
async fn storage_deposit_refunds_excessive_deposit() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);

    // Check the storage balance bounds to make sure we have the right minimal deposit
    #[derive(near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceBounds {
        min: U128,
        max: U128,
    }
    let storage_balance_bounds: StorageBalanceBounds = ft_contract
        .call_function("storage_balance_bounds", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        storage_balance_bounds.min,
        minimal_deposit.as_yoctonear().into()
    );
    assert_eq!(
        storage_balance_bounds.max,
        minimal_deposit.as_yoctonear().into()
    );

    // Check that a non-registered account does not have storage balance
    #[derive(near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceOf {
        total: U128,
        available: U128,
    }
    let storage_balance_of: Option<StorageBalanceOf> = ft_contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": "non-registered-account.sandbox"}),
        )
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert!(storage_balance_of.is_none());

    // Create a new account and deposit some NEAR to cover the storage
    let new_account = common::create_subaccount(&sandbox, "new-account.sandbox")
        .await
        .unwrap();

    let new_account_balance_before_deposit = new_account
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;
    let contract_balance_before_deposit = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total;

    ft_contract
        .call_function("storage_deposit", json!({"account_id": new_account.account_id(), "registration_only": Option::<bool>::None}))
        .transaction()
        .deposit(NearToken::from_near(5))
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // The expected storage balance should be the minimal deposit,
    // the balance of the account should be reduced by the deposit,
    // and the contract should gain the deposit.
    let storage_balance: StorageBalanceOf = ft_contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": new_account.account_id()}),
        )
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(storage_balance.total, minimal_deposit.as_yoctonear().into());
    assert_eq!(storage_balance.available, U128::from(0));

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        new_account
            .tokens()
            .near_balance()
            .fetch_from(&sandbox_network)
            .await?
            .total,
    );
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(1))
    );

    let contract_balance_diff = ft_contract
        .as_account()
        .tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await?
        .total
        .saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so the difference should be slightly more than minimal_deposit
    assert!(contract_balance_diff > minimal_deposit);
    assert!(
        contract_balance_diff
            < minimal_deposit.saturating_add(NearToken::from_yoctonear(50_000_000_000_000_000_000))
    );

    Ok(())
}

#[tokio::test]
async fn close_account_empty_balance() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    // Alice unregisters with empty balance
    ft_contract
        .call_function("storage_unregister", json!({"force": Option::<bool>::None}))
        .transaction()
        .deposit(common::ONE_YOCTO)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

#[tokio::test]
async fn close_account_non_empty_balance() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register ft_contract account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    // ft_contract tries to unregister with non-empty balance without force
    let res = ft_contract
        .call_function("storage_unregister", json!({"force": Option::<bool>::None}))
        .transaction()
        .deposit(common::ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    let error_msg = format!("{:?}", res);
    res.assert_failure();
    assert!(
        error_msg.contains("Can't unregister the account with the positive balance without force")
    );

    // ft_contract tries to unregister with non-empty balance with force=false
    let res = ft_contract
        .call_function("storage_unregister", json!({"force": Some(false)}))
        .transaction()
        .deposit(common::ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    let error_msg = format!("{:?}", res);
    res.assert_failure();
    assert!(
        error_msg.contains("Can't unregister the account with the positive balance without force")
    );

    Ok(())
}

#[tokio::test]
async fn close_account_force_non_empty_balance() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register ft_contract account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    // ft_contract unregisters with force=true, burning all tokens
    ft_contract
        .call_function("storage_unregister", json!({"force": Some(true)}))
        .transaction()
        .deposit(common::ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Check that total supply is now 0
    let total_supply: U128 = ft_contract
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(total_supply.0, 0);

    Ok(())
}
