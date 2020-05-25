import { u128, VM, Context, storage } from 'near-sdk-as'
import * as contract from '../main'
import * as model from '../models'
import * as nonSpec from '../non-spec'

const alice = 'alice'
const bob = 'bob'
const carol = 'carol'

beforeEach(() => {
  // increase storage size to avoid InconsistentStateError(IntegerOverflow)
  Context.setStorage_usage(200)

  // given tokens are minted to alice
  nonSpec.mint(alice)
})

afterEach(() => {
  // cleanup storage between tests
  storage.delete('minted')

  model.balanceRegistry.delete(alice)
  model.balanceRegistry.delete(bob)
  model.balanceRegistry.delete(carol)

  model.allowanceRegistry.delete(model.getAllowanceKey(alice, bob))
})

/******************/
/* CHANGE METHODS */
/******************/

// export function set_allowance(escrow_account_id: AccountId, allowance: Amount): void {
describe('set_allowance', () => {
  it('should change the allowance of an escrow account on owner owner', () => {
    // when alice records bob as her escrow up to `allowance`
    Context.setPredecessor_account_id(alice)
    const allowance = u128.from(100)
    contract.set_allowance(bob, allowance)

    // then bob will appear to be authorized to transfer up to `allowance` of alice's tokens
    expect(contract.get_allowance(alice, bob)).toBe(allowance)
  })
})

describe('transfer_from', () => {
  it('should reflect updates to account balances when called by token owner', () => {
    // given alice holds all tokens
    const aliceBalanceBeforeXfer = contract.get_balance(alice)

    // when we transfer tokens to bob
    Context.setPredecessor_account_id(alice)
    const amount = u128.from(2)
    contract.transfer_from(alice, bob, amount)

    // then we should expect to see bob and alice's balances reflect the change
    const aliceBalanceAfterXfer = contract.get_balance(alice)
    const bobBalanceAfterXfer = contract.get_balance(bob)
    expect(aliceBalanceAfterXfer).toBe(u128.sub(aliceBalanceBeforeXfer, amount))
    expect(bobBalanceAfterXfer).toBe(amount)
  })

  it('should reflect updates to account balances when called by escrow', () => {
    // given alice holds all tokens
    const aliceBalanceBeforeXfer = contract.get_balance(alice)

    // when we set bob as escrow
    Context.setPredecessor_account_id(alice)
    const amount = u128.from(2)
    contract.set_allowance(bob, amount)

    // and when bob transfers alice's tokens to carol
    Context.setPredecessor_account_id(bob)
    contract.transfer_from(alice, carol, amount)

    // then we should expect to see bob and alice's balances reflect the change
    const aliceBalanceAfterXfer = contract.get_balance(alice)
    const carolBalanceAfterXfer = contract.get_balance(carol)
    expect(aliceBalanceAfterXfer).toBe(u128.sub(aliceBalanceBeforeXfer, amount))
    expect(carolBalanceAfterXfer).toBe(amount)
  })
})

describe('transfer', () => {
  it('should reflect updates to account balances', () => {
    // given alice holds all tokens
    const aliceBalanceBeforeXfer = contract.get_balance(alice)

    // when we transfer tokens to bob
    Context.setPredecessor_account_id(alice)
    const amount = u128.from(2)
    contract.transfer(bob, amount)

    // then we should expect to see bob and alice's balances reflect the change
    const aliceBalanceAfterXfer = contract.get_balance(alice)
    const bobBalanceAfterXfer = contract.get_balance(bob)
    expect(aliceBalanceAfterXfer).toBe(u128.sub(aliceBalanceBeforeXfer, amount))
    expect(bobBalanceAfterXfer).toBe(amount)
  })
})

/****************/
/* VIEW METHODS */
/****************/

describe('get_total_supply', () => {
  it('should match the TOTAL_SUPPLY', () => {
    expect(contract.get_total_supply()).toBe(model.TOTAL_SUPPLY)
  })
})

describe('get_balance', () => {
  it('should provide the balance of an account', () => {
    // when we ask for alice's balance
    const balance = contract.get_balance(alice)

    // then initial owner after minting should have all the tokens
    expect(balance).toBe(model.TOTAL_SUPPLY)

    // any other account should throw
    expect(() => {
      contract.get_balance(bob)
    }).toThrow(nonSpec.ERR_INVALID_ACCOUNT)
  })

  /* this is basically just a test of transfer()

  it('should reflect updates after transfer()', () => {
    const aliceBalanceBeforeXfer = contract.get_balance(alice)

    // when we transfer tokens to bob
    Context.setPredecessor_account_id(alice)
    const amount = u128.from(1)
    contract.transfer(bob, amount)

    // then we should expect to see bob and alice's balances reflect the change
    const aliceBalanceAfterXfer = contract.get_balance(alice)
    const bobBalanceAfterXfer = contract.get_balance(bob)
    expect(aliceBalanceAfterXfer).toBe(u128.sub(aliceBalanceBeforeXfer, amount))
    expect(bobBalanceAfterXfer).toBe(amount)
  })
  */

  /* seems likely that this will also just be a test of transfer_from

  xit('should reflect updates after transfer_from()', () => {
    // given alice authorizes bob as escrow up to amount
    // when bob transfers from alice to carol
    // then alice accounts should reflect a debit and carol's account a credit
  })
  */
})

describe('get_allowance', () => {
  /* not sure how to test get_allowance without testing set_allowance

  it("should provide the allowance an escrow has to spend of an owner's tokens", () => {
    // when alice records bob as her escrow up to `allowance`
    Context.setPredecessor_account_id(alice)
    const allowance = u128.from(100)
    contract.set_allowance(bob, allowance)

    // then bob will appear to be authorized to transfer up to `allowance` of alice's tokens
    expect(contract.get_allowance(alice, bob)).toBe(allowance)
  })
  */

  xit('should reflect updates after transfer_from()', () => {
    // given an existing allowance between escrow and owner
    // when tokens are transfered by escrow
    // then existing allowance should decrease accordingly
  })
})
