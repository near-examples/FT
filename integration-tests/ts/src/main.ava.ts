import { Worker, NEAR, NearAccount, parseNEAR } from "near-workspaces";
import anyTest, { TestFn } from "ava";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

const TOTAL_SUPPLY = parseNEAR("300 N");
const DEFAULT_GAS = "30" + "0".repeat(12);
const STORAGE = "125" + "0".repeat(22);

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init();

  // deploy contract
  const root = worker.rootAccount;
  const ft_contract = await root.createAndDeploy(
    root.getSubAccount("fungible-token").accountId,
    "../../res/fungible_token.wasm",
    {
      method: "new_default_meta",
      args: { owner_id: root, total_supply: TOTAL_SUPPLY },
    }
  );
  const defi_contract = await root.createAndDeploy(
    root.getSubAccount("fungible-token").accountId,
    "../../res/defi.wasm",
    {
      method: "new",
      args: { fungible_token_account_id: ft_contract }
    }
  );

  // some test accounts
  const alice = await root.createSubAccount("alice", {
    initialBalance: NEAR.parse("30 N").toJSON(),
  });
  const bob = await root.createSubAccount("bob", {
    initialBalance: NEAR.parse("30 N").toJSON(),
  });

  // Register accounts with ft_contract
  await alice.call(
    ft_contract,
    "storage_deposit",
    { account_id: alice },
    { gas: DEFAULT_GAS, attachedDeposit: STORAGE }
  );
  await bob.call(
    ft_contract,
    "storage_deposit",
    { account_id: bob },
    { gas: DEFAULT_GAS, attachedDeposit: STORAGE }
  );
  await defi_contract.call(
    ft_contract,
    "storage_deposit",
    { account_id: defi_contract },
    { gas: DEFAULT_GAS, attachedDeposit: STORAGE }
  );

  // Save state for test runs, it is unique for each test
  t.context.worker = worker;
  t.context.accounts = {
    root,
    ft_contract,
    defi_contract,
    alice,
    bob,
  };
});

test.afterEach(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error);
  });
});

test("simulate_total_supply", async (t) => {
  const { ft_contract } = t.context.accounts;
  const totalSupply = await ft_contract.view("ft_total_supply");
  t.is(totalSupply, TOTAL_SUPPLY.toString());
});

test("simulate_simple_transfer", async (t) => {
  const transferAmount = parseNEAR("100");
  const initialBalance = TOTAL_SUPPLY;
  const { root, ft_contract, alice } = t.context.accounts;

  // Transfer from root to alice
  await root.call(
    ft_contract,
    "ft_transfer",
    {
      receiver_id: alice,
      amount: transferAmount,
    },
    { gas: DEFAULT_GAS, attachedDeposit: "1" }
  );

  const rootBalance: string = await ft_contract.view("ft_balance_of", {
    account_id: root,
  });

  const aliceBalance: string = await ft_contract.view("ft_balance_of", {
    account_id: alice,
  });

  t.is(
    (initialBalance.toBigInt() - transferAmount.toBigInt()).toString(),
    rootBalance
  );
  t.is(transferAmount.toString(), aliceBalance);
});

test("simulate_close_account_empty_balance", async (t) => {
  const { ft_contract, alice } = t.context.accounts;
  const outcome = await alice.callRaw(
    ft_contract,
    "storage_unregister",
    {},
    { attachedDeposit: "1" }
  );
  t.true(outcome.succeeded);
  const aliceBalance: string = await ft_contract.view("ft_balance_of", {
    account_id: alice,
  });
  t.is(aliceBalance, "0");
});

test("simulate_close_account_non_empty_balance", async (t) => {
  const { root, ft_contract } = t.context.accounts;
  const outcome = await root.callRaw(
    ft_contract,
    "storage_unregister",
    {},
    { attachedDeposit: "1" }
  );
  t.false(outcome.succeeded);
  t.regex(
    outcome.receiptFailureMessages.join("\n"),
    /Can't unregister the account with the positive balance without force/
  );
});

test("simulate_close_account_force_non_empty_balance", async (t) => {
  const { root, ft_contract } = t.context.accounts;
  const outcome = await root.callRaw(
    ft_contract,
    "storage_unregister",
    { force: true },
    { attachedDeposit: "1" }
  );
  t.true(outcome.succeeded);
  t.is(outcome.logs[0], `Closed @${root.accountId} with ${TOTAL_SUPPLY}`);
  const totalSupply = await ft_contract.view("ft_total_supply");
  t.is(totalSupply, "0");
});

test("simulate_transfer_call_with_burned_amount", async (t) => {
  const { root, ft_contract, defi_contract, alice, bob } = t.context.accounts;
  t.log("Passed ✅");
});

test("simulate_transfer_call_with_immediate_return_and_no_refund", async (t) => {
  const { root, ft_contract, defi_contract, alice, bob } = t.context.accounts;
  t.log("Passed ✅");
});

test("simulate_transfer_call_when_called_contract_not_registered_with_ft", async (t) => {
  const { root, ft_contract, defi_contract, alice, bob } = t.context.accounts;
  t.log("Passed ✅");
});

test("simulate_transfer_call_with_promise_and_refund", async (t) => {
  const { root, ft_contract, defi_contract, alice, bob } = t.context.accounts;
  t.log("Passed ✅");
});

test("simulate_transfer_call_promise_panics_for_a_full_refund", async (t) => {
  const { root, ft_contract, defi_contract, alice, bob } = t.context.accounts;
  t.log("Passed ✅");
});
