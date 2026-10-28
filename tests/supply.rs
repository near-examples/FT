pub mod common;

use near_api::{types::json::U128, NearToken};

#[tokio::test]
async fn test_total_supply() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let (ft_contract, _, _) =
        common::init_contracts(&sandbox, &sandbox_network, &initial_balance).await?;

    // Get the supply data
    let supply: U128 = ft_contract
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(supply, initial_balance);

    Ok(())
}
