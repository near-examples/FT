import { Workspace, BN, NearAccount, captureError, toYocto, NEAR, transfer, DEFAULT_FUNCTION_CALL_GAS, tGas } from 'near-workspaces-ava';

async function registerUser(ft: NearAccount, user: NearAccount) {
  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    { attachedDeposit: toYocto('0.00125') }
  );
}

async function ft_balance_of(ft: NearAccount, user: NearAccount): Promise<BN> {
  return new BN(await ft.view('ft_balance_of', {
    account_id: user,
  }));
}

async function init_ft(
  ft: NearAccount,
  owner: NearAccount,
  supply: BN | string,
  registeredAccount?: NearAccount,
) {
  await ft.call(
    ft,
    'new_default_meta',
    {
      owner_id: owner,
      total_supply: supply,
    }
  );if (registeredAccount) {
    await registerUser(ft, registeredAccount);
  }
}

const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createAccount('alice');

  // Create a subaccount of the root account, and also deploy a contract to it
  const ft = await root.createAndDeploy(
    'fungible-token',
    'res/fungible_token.wasm',
  );

  const defi = await root.createAndDeploy(
    'defi',
    'res/defi.wasm',
    {
      method: 'new',
      args: { fungible_token_account_id: ft.accountId },
    },
  );

  return { alice, ft, defi };
});

workspace.test('Total supply', async (test, { ft, root }) => {
  const initialBalance = '1000';
  await init_ft(ft, root, initialBalance);
  test.is(
    await ft.view('ft_total_supply'),
    initialBalance,
  );
});

workspace.test('Simple transfer', async (test, { root, alice, ft }) => {
  const transferAmount = new BN(100);
  const initialBalance = new BN(100_000);
  await init_ft(ft, root, initialBalance, alice)

  await root.call(
    ft,
    'ft_transfer',
    {
      receiver_id: alice,
      amount: transferAmount,
    },
    {
      attachedDeposit: '1',
    },
  )
  const rootBalance: string = await ft.view('ft_balance_of', { account_id: root });
  const aliceBalance: string = await ft.view('ft_balance_of', { account_id: alice });

  test.is(initialBalance.toNumber() - transferAmount.toNumber(), parseInt(rootBalance));
  test.is(transferAmount.toNumber(), parseInt(aliceBalance));
});

workspace.test('close account non empty balance', async (test, { alice, ft, root }) => {
  const initialBalance = new BN(100_000);
  await init_ft(ft, root, initialBalance, alice);
  const res: boolean = await alice.call(
    ft,
    'storage_unregister',
    {},
    { attachedDeposit: '1' },
  );
  test.true(res);
});


workspace.test('Close account force non empty balance', async (test, { ft, root }) => {
  await init_ft(ft, root, '100000');
  const errorString = await captureError(async () =>
    root.call(
      ft,
      'storage_unregister',
      {},
      { attachedDeposit: '1' }
    )
  );

  test.regex(errorString, /Can't unregister the account with the positive balance without force/);

  const result = await root.call_raw(
    ft,
    'storage_unregister',
    { force: true },
    { attachedDeposit: '1' },
  );

  test.is(result.logs[0],
    `Closed @${root.accountId} with 100000`,
  );
  test.is(result.succeeded, true);
  test.is(result.parseResult<boolean>(), true);

  const total_supply: string = await ft.view('ft_total_supply');

  test.is(total_supply, '0');
});


workspace.test('Transfer call with burned amount', async (test, { ft, root, defi }) => {
  const transferAmount = new BN(100);
  const initialBalance = new BN(10_000);
  const burnAmount = new BN(10);

  await init_ft(ft, root, initialBalance, defi);

  const result = await root
    .createTransaction(ft)
    .functionCall(
      'ft_transfer_call',
      {
        receiver_id: defi,
        amount: transferAmount,
        msg: burnAmount,
      },
      { attachedDeposit: '1', gas: tGas('150') },
    ).functionCall(
      'storage_unregister',
      { force: true },
      { attachedDeposit: '1', gas: tGas('150') },
    ).signAndSend();

  test.true(result.logs.includes(
    `Closed @${root.accountId} with ${(initialBalance.sub(transferAmount)).toString()}`,
  ));

  test.is(result.parseResult(), true);

  test.true(result.logs.includes(
    'The account of the sender was deleted',
  ));
  test.true(result.logs.includes(
    `Account @${root.accountId} burned ${burnAmount.toString()}`,
  ));

  const callbackOutcome = result.receipts_outcomes[5];

  test.is(callbackOutcome.parseResult(), transferAmount.toString());
  const expectedAmount = transferAmount.sub(burnAmount);

  const totalSupply: string = await ft.view('ft_total_supply');
  test.is(totalSupply, expectedAmount.toString());

  const defiBalance = await ft_balance_of(ft, defi);;
  test.deepEqual(defiBalance, expectedAmount);
});

workspace.test('Transfer call with immediate return and no refund', async (test, { ft, defi, root }) => {
  const initialAmount = new BN(10_000);
  const transferAmount = new BN(100);
  await init_ft(ft, root, initialAmount, defi);

  await root.call(
    ft,
    'ft_transfer_call',
    {
      receiver_id: defi,
      amount: transferAmount,
      msg: 'take-my-money',
    },
    { attachedDeposit: '1', gas: tGas('150') },
  );

  const rootBalance = await ft_balance_of(ft, root);
  const defiBalance = await ft_balance_of(ft, defi);

  test.deepEqual(rootBalance, initialAmount.sub(transferAmount));
  test.deepEqual(defiBalance, transferAmount);
});


workspace.test('Transfer call when called contract not registered with ft', async (test, { ft, defi, root }) => {
  let transferAmount = new BN(100);
  let initialAmount = new BN(1000);

  await init_ft(ft, root, initialAmount);
  const errorString = await captureError(async () =>
    await root.call(
      ft,
      'ft_transfer_call',
      {
        receiver_id: defi,
        amount: transferAmount,
        msg: 'take-my-money',
      },
      { attachedDeposit: '1', gas: tGas('150') },
    )
  );
  test.regex(errorString, new RegExp('The account ' + defi.accountId + ' is not registered'));

  const rootBalance = await ft_balance_of(ft, root);
  const defiBalance = await ft_balance_of(ft, defi);

  test.deepEqual(rootBalance, initialAmount);
  test.assert(defiBalance.isZero(), `Expected zero got ${defiBalance.toJSON()}`);
});


workspace.test('Transfer call with promise and refund', async (test, { ft, defi, root }) => {
  let transferAmount = new BN(100);
  let refundAmount = new BN(50);
  let initialAmount = new BN(1000);

  await init_ft(ft, root, initialAmount, defi);

  await root.call(
    ft,
    'ft_transfer_call',
    {
      receiver_id: defi,
      amount: transferAmount,
      msg: refundAmount,
    },
    { attachedDeposit: '1', gas: tGas('150') },
  );

  const rootBalance = await ft_balance_of(ft, root);
  const defiBalance = await ft_balance_of(ft, defi);

  test.deepEqual(rootBalance, initialAmount.sub(transferAmount).add(refundAmount));
  test.deepEqual(defiBalance, transferAmount.sub(refundAmount));
});

workspace.test('Transfer call promise panics for a full refund', async (test, {ft, defi, root}) => {
  const initialAmount = new BN(10_000);
  const transferAmount = new BN(100);
  await init_ft(ft, root, initialAmount, defi);

  const result = await root.call_raw(
    ft,
    'ft_transfer_call',
    {
      receiver_id: defi,
      amount: transferAmount,
      msg: 'no parsey as integer big panic oh no',
    },
    {attachedDeposit: '1', gas: tGas('150')},
  );
  test.regex(result.promiseErrorMessages[0], /ParseIntError/);

  const rootBalance = await ft_balance_of(ft, root);
  const defiBalance = await ft_balance_of(ft, defi);

  test.deepEqual(rootBalance, initialAmount);
  test.assert(defiBalance.isZero(), `Expected zero got ${defiBalance.toJSON()}`);
});