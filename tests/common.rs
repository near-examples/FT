use std::sync::{Arc, LazyLock};

use cargo_near_build::BuildOpts;
use near_api::{types::json::U128, Account, AccountId, Contract, NearToken, NetworkConfig, Signer};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sandbox::Sandbox;
use near_sdk::serde_json::json;

const INITIAL_BALANCE: NearToken = NearToken::from_near(30);
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

static FUNGIBLE_TOKEN_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let artifact = cargo_near_build::build_with_cli(BuildOpts {
        no_abi: true,
        no_embed_abi: true,
        no_locked: true,
        ..Default::default()
    })
    .expect("Could not compile Fungible Token contract for tests");

    let contract_wasm = std::fs::read(&artifact)
        .expect(format!("Could not read Fungible Token WASM file from {}", artifact).as_str());

    contract_wasm
});

static DEFI_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let artifact_path = cargo_near_build::build_with_cli(BuildOpts {
        manifest_path: Some("tests/contracts/defi/Cargo.toml".into()),
        no_abi: true,
        no_embed_abi: true,
        no_locked: true,
        ..Default::default()
    })
    .expect("Could not compile defi fixture");

    let contract_wasm = std::fs::read(&artifact_path)
        .expect(format!("Could not read DeFi WASM file from {}", artifact_path).as_str());

    contract_wasm
});

pub async fn init_sandbox() -> anyhow::Result<(Sandbox, NetworkConfig)> {
    // Initialize the sandbox
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    Ok((sandbox, sandbox_network))
}

pub async fn init_accounts(
    sandbox: &Sandbox,
) -> anyhow::Result<(Account, Account, Account, Account)> {
    // create accounts
    let alice = create_subaccount(&sandbox, "alice.sandbox").await.unwrap();
    let bob = create_subaccount(&sandbox, "bob.sandbox").await.unwrap();
    let charlie = create_subaccount(&sandbox, "charlie.sandbox")
        .await
        .unwrap();
    let dave = create_subaccount(&sandbox, "dave.sandbox").await.unwrap();

    return Ok((alice, bob, charlie, dave));
}

pub async fn init_contracts(
    sandbox: &Sandbox,
    sandbox_network: &NetworkConfig,
    initial_balance: &U128,
) -> anyhow::Result<(Contract, Contract, Arc<Signer>)> {
    let ft_contract = create_subaccount(&sandbox, "ft-contract.sandbox")
        .await
        .unwrap()
        .as_contract();
    let defi_contract = create_subaccount(&sandbox, "defi-contract.sandbox")
        .await
        .unwrap()
        .as_contract();

    // Initialize signer for the contract deployment
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    // Deploy the ft contract
    near_api::Contract::deploy(ft_contract.account_id().clone())
        .use_code(FUNGIBLE_TOKEN_CONTRACT_WASM.to_vec())
        .with_init_call(
            "new",
            json!({"owner_id": ft_contract.account_id().clone(), "total_supply": initial_balance, "metadata": FungibleTokenMetadata {
              spec: FT_METADATA_SPEC.to_string(),
              name: "Example NEAR fungible token".to_string(),
              symbol: "EXAMPLE".to_string(),
              icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
              reference: None,
              reference_hash: None,
              decimals: 24,
          },}),
        )?
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    near_api::Contract::deploy(defi_contract.account_id().clone())
        .use_code(DEFI_CONTRACT_WASM.to_vec())
        .with_init_call(
            "new",
            json!({"fungible_token_account_id": ft_contract.account_id().clone()}),
        )?
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    return Ok((ft_contract, defi_contract, signer));
}

pub async fn register_user(
    contract: &Contract,
    signer: Arc<Signer>,
    sandbox_network: &NetworkConfig,
    account_id: &AccountId,
) -> anyhow::Result<()> {
    contract
        .call_function(
            "storage_deposit",
            json!({"account_id": account_id, "registration_only": Option::<bool>::None}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(7000000000000000000000))
        .with_signer(contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

/// Wait for asynchronous gas/deposit refunds before measuring spendable balance.
pub async fn refunded_balance(
    account: &Account,
    network: &NetworkConfig,
    minimum: NearToken,
) -> anyhow::Result<NearToken> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let balance = account
            .tokens()
            .near_balance()
            .fetch_from(network)
            .await?
            .total;
        if balance >= minimum {
            return Ok(balance);
        }
        anyhow::ensure!(
            tokio::time::Instant::now() < deadline,
            "Balance {} stayed below {} after refunds",
            balance,
            minimum
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub async fn create_subaccount(
    sandbox: &near_sandbox::Sandbox,
    name: &str,
) -> testresult::TestResult<near_api::Account> {
    let account_id: AccountId = name.parse().unwrap();
    sandbox
        .create_account(account_id.clone())
        .initial_balance(INITIAL_BALANCE)
        .send()
        .await?;
    Ok(near_api::Account(account_id))
}
