use std::sync::LazyLock;

use cargo_near_build::BuildOpts;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::{Account, Contract, DevNetwork, Worker};

const INITIAL_BALANCE: NearToken = NearToken::from_near(30);
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

static FUNGIBLE_TOKEN_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let artifact = cargo_near_build::build(BuildOpts {
        no_abi: true,
        no_embed_abi: true,
        ..Default::default()
    })
    .expect("Could not compile Fungible Token contract for tests");

    let contract_wasm = std::fs::read(&artifact.path).expect(
        format!(
            "Could not read Fungible Token WASM file from {}",
            artifact.path
        )
        .as_str(),
    );

    contract_wasm
});

static DEFI_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let artifact_path = "tests/contracts/defi/res/defi.wasm";

    let contract_wasm = std::fs::read(artifact_path)
        .expect(format!("Could not read DeFi WASM file from {}", artifact_path).as_str());

    contract_wasm
});

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
    let ft_contract = worker.dev_deploy(&FUNGIBLE_TOKEN_CONTRACT_WASM).await?;

    let res = ft_contract
        .call("new")
        .args_json((
            ft_contract.id(),
            initial_balance,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        ))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let defi_contract = worker.dev_deploy(&DEFI_CONTRACT_WASM).await?;

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
