pub mod common;

use near_api::{
    types::{json::U128, transaction::result::ValueOrReceiptId},
    types::{transaction::actions::FunctionCallAction, Action, NearGas},
    NearToken, Transaction,
};
use near_sdk::serde_json::json;

use common::ONE_YOCTO;

#[tokio::test]
async fn simple_transfer() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register the ft contract account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let transfer_amount = NearToken::from_near(10);

    // Transfer tokens to alice
    ft_contract
        .call_function(
            "ft_transfer",
            (
                alice.account_id().clone(),
                transfer_amount,
                Option::<bool>::None,
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Get ft_contract balance
    let ft_contract_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    // Get alice balance
    let alice_balance: U128 = ft_contract
        .call_function("ft_balance_of", (alice.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    assert_eq!(
        NearToken::from_yoctonear(initial_balance.0)
            .saturating_sub(transfer_amount)
            .as_yoctonear(),
        ft_contract_balance.0
    );
    assert_eq!(transfer_amount.as_yoctonear(), alice_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_burned_amount() -> anyhow::Result<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    // Unregister in the same receipt as the transfer, before the receiver and
    // refund callback execute. Sequential transactions would miss this case.
    let res = Transaction::construct(
        ft_contract.account_id().clone(),
        ft_contract.account_id().clone(),
    )
    .add_action(Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "ft_transfer_call".into(),
        args: near_sdk::serde_json::to_vec(&json!({
            "receiver_id": defi_contract.account_id(),
            "amount": transfer_amount,
            "memo": Option::<String>::None,
            "msg": "10",
        }))?,
        gas: NearGas::from_tgas(150),
        deposit: ONE_YOCTO,
    })))
    .add_action(Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "storage_unregister".into(),
        args: near_sdk::serde_json::to_vec(&json!({"force": true}))?,
        gas: NearGas::from_tgas(150),
        deposit: ONE_YOCTO,
    })))
    .with_signer(signer.clone())
    .send_to(&sandbox_network)
    .await?
    .into_full()
    .expect("Final execution must include receipt outcomes");
    assert!(res.is_success());
    assert!(res.receipt_failures().is_empty());

    let logs = res.logs();
    assert!(logs.contains(&"The account of the sender was deleted"));
    let burned = format!("Account @{} burned {}", ft_contract.account_id(), 10);
    assert!(logs.contains(&burned.as_str()));

    // Locate the resolver's result by executor and value, not receipt order.
    assert!(
        res.receipt_outcomes().iter().any(|outcome| {
            outcome.executor_id == *ft_contract.account_id()
                && matches!(outcome.clone().into_result(),
                Ok(ValueOrReceiptId::Value(value))
                    if value.json::<U128>().is_ok_and(|used| used.0 == transfer_amount.0))
        }),
        "The resolver must report the entire transfer as used when its sender is gone"
    );
    assert!(res.into_result()?.json::<bool>()?);

    let supply: U128 = ft_contract
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(supply.0, transfer_amount.0 - 10);

    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(defi_balance.0, transfer_amount.0 - 10);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_immediate_return_and_no_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let transfer_amount = NearToken::from_near(10);

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    // root invests in defi by calling `ft_transfer_call`
    ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount,
                Option::<String>::None,
                "take-my-money",
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        NearToken::from_yoctonear(initial_balance.0)
            .saturating_sub(transfer_amount)
            .as_yoctonear(),
        root_balance.0
    );
    assert_eq!(transfer_amount.as_yoctonear(), defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_when_called_contract_not_registered_with_ft() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (_, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    let transfer_amount = NearToken::from_near(10);

    // call fails because DEFI contract is not registered as FT user
    let res = ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount,
                Option::<String>::None,
                "take-my-money",
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(res.is_failure());

    // balances remain unchanged
    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(initial_balance.0, root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_promise_and_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    let refund_amount = NearToken::from_near(5);
    let transfer_amount = NearToken::from_near(10);

    ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount,
                Option::<String>::None,
                refund_amount.as_yoctonear().to_string(),
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(
        NearToken::from_yoctonear(initial_balance.0)
            .saturating_sub(transfer_amount)
            .saturating_add(refund_amount)
            .as_yoctonear(),
        root_balance.0
    );
    assert_eq!(
        transfer_amount.saturating_sub(refund_amount).as_yoctonear(),
        defi_balance.0
    );

    Ok(())
}

#[tokio::test]
async fn transfer_call_promise_panics_for_a_full_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount,
                Option::<String>::None,
                "no parsey as integer big panic oh no".to_string(),
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .into_full()
        .expect("Final execution must include receipt outcomes");
    assert!(res.is_success());

    let promise_failures = res.receipt_failures();
    assert_eq!(promise_failures.len(), 1);
    let failure = promise_failures[0].clone().into_result();
    if let Err(err) = failure {
        assert!(format!("{:?}", err).contains("ParseIntError"));
    } else {
        unreachable!();
    }

    // balances remain unchanged
    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(initial_balance.0, root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}
