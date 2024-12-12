pub mod common;

use near_sdk::{json_types::U128, NearToken};
use near_workspaces::{operations::Function, result::ValueOrReceiptId};

use common::{init_accounts, init_contracts, register_user, ONE_YOCTO};

#[tokio::test]
async fn simple_transfer() -> anyhow::Result<()> {
    // Create balance variables
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance, &alice).await?;

    let res = ft_contract
        .call("ft_transfer")
        .args_json((alice.id(), transfer_amount, Option::<bool>::None))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let ft_contract_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let alice_balance = ft_contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, ft_contract_balance.0);
    assert_eq!(transfer_amount.0, alice_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_burned_amount() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance, &alice).await?;

    // defi contract must be registered as a FT account
    register_user(&ft_contract, defi_contract.id()).await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = ft_contract
        .batch()
        .call(
            Function::new("ft_transfer_call")
                .args_json((
                    defi_contract.id(),
                    transfer_amount,
                    Option::<String>::None,
                    "10",
                ))
                .deposit(ONE_YOCTO)
                .gas(near_sdk::Gas::from_tgas(150)),
        )
        .call(
            Function::new("storage_unregister")
                .args_json((Some(true),))
                .deposit(ONE_YOCTO)
                .gas(near_sdk::Gas::from_tgas(150)),
        )
        .transact()
        .await?;
    assert!(res.is_success());

    let logs = res.logs();
    let expected = format!("Account @{} burned {}", ft_contract.id(), 10);
    assert!(logs.len() >= 2);
    assert!(logs.contains(&"The account of the sender was deleted"));
    assert!(logs.contains(&(expected.as_str())));

    match res.receipt_outcomes()[5].clone().into_result()? {
        ValueOrReceiptId::Value(val) => {
            let used_amount = val.json::<U128>()?;
            assert_eq!(used_amount, transfer_amount);
        }
        _ => panic!("Unexpected receipt id"),
    }
    assert!(res.json::<bool>()?);

    let res = ft_contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?.0, transfer_amount.0 - 10);
    let defi_balance = ft_contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(defi_balance.0, transfer_amount.0 - 10);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_immediate_return_and_no_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance, &alice).await?;

    // defi contract must be registered as a FT account
    register_user(&ft_contract, defi_contract.id()).await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = ft_contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "take-my-money",
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = ft_contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, root_balance.0);
    assert_eq!(transfer_amount.0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_when_called_contract_not_registered_with_ft() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance, &alice).await?;

    // call fails because DEFI contract is not registered as FT user
    let res = ft_contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "take-my-money",
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_failure());

    // balances remain unchanged
    let root_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = ft_contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0, root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_promise_and_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let refund_amount = U128::from(NearToken::from_near(50).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance, &alice).await?;

    // defi contract must be registered as a FT account
    register_user(&ft_contract, defi_contract.id()).await?;

    let res = ft_contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            refund_amount.0.to_string(),
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = ft_contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        initial_balance.0 - transfer_amount.0 + refund_amount.0,
        root_balance.0
    );
    assert_eq!(transfer_amount.0 - refund_amount.0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_promise_panics_for_a_full_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance, &alice).await?;

    // defi contract must be registered as a FT account
    register_user(&ft_contract, defi_contract.id()).await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = ft_contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "no parsey as integer big panic oh no".to_string(),
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
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
    let root_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = ft_contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance, root_balance);
    assert_eq!(0, defi_balance.0);

    Ok(())
}
