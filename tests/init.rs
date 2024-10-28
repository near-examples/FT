use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::{Account, Contract, DevNetwork, Worker};

const INITIAL_BALANCE: NearToken = NearToken::from_near(30);
pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

pub async fn init_accounts(root: &Account) -> anyhow::Result<(Account, Account, Account, Account)> {
    // create accounts
    let alice = root
        .create_subaccount("alice")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let bob = root
        .create_subaccount("bob")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let charlie = root
        .create_subaccount("charlie")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let dave = root
        .create_subaccount("dave")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    return Ok((alice, bob, charlie, dave));
}

pub async fn init_contracts(
    worker: &Worker<impl DevNetwork>,
    initial_balance: U128,
    account: &Account,
) -> anyhow::Result<(Contract, Contract)> {
    let ft_wasm = near_workspaces::compile_project(".").await?;
    let ft_contract = worker.dev_deploy(&ft_wasm).await?;

    let res = ft_contract
        .call("new_default_meta")
        .args_json((ft_contract.id(), initial_balance))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let defi_wasm = near_workspaces::compile_project("./tests/contracts/defi").await?;
    let defi_contract = worker.dev_deploy(&defi_wasm).await?;

    let res = defi_contract
        .call("new")
        .args_json((ft_contract.id(),))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let res = ft_contract
        .call("storage_deposit")
        .args_json((account.id(), Option::<bool>::None))
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    return Ok((ft_contract, defi_contract));
}

pub async fn register_user(contract: &Contract, account_id: &AccountId) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((account_id, Option::<bool>::None))
        .max_gas()
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}
