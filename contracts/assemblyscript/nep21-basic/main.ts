import { context, storage, u128 } from 'near-sdk-as'
import { AccountId, Amount } from './types'
import {
  allowanceRegistry,
  balanceRegistry,
  getAllowanceKey as keyFrom,
  TOTAL_SUPPLY,
} from './models'

/******************/
/* ERROR MESSAGES */
/******************/

export const ERR_INVALID_AMOUNT = 'Allowance must be greater than zero'
export const ERR_INVALID_ACCOUNT = 'Account not found in registry'
export const ERR_INVALID_ESCROW_ACCOUNT = 'Escrow account not found in registry'
export const ERR_INSUFFICIENT_BALANCE = 'Account does not have enough balance for this transaction'
export const ERR_INSUFFICIENT_ESCROW_BALANCE = 'Escrow account does not have enough allowance for this transaction'
export const ERR_TOKEN_ALREADY_MINTED = 'Token has previously been minted'

/********************/
/* NON-SPEC METHODS */
/********************/

const HAS_BEEN_MINTED = 'm'

export function init(): void {
  // check if previously minted
  assert(!storage.contains(HAS_BEEN_MINTED), ERR_TOKEN_ALREADY_MINTED)

  // assign ownership to 
  balanceRegistry.set(context.sender, TOTAL_SUPPLY)

  // record that minting is complete
  storage.set(HAS_BEEN_MINTED, true)
}

/******************/
/* CHANGE METHODS */
/******************/

/**
 * Sets the `allowance` for `escrow_account_id` on the account of the caller of this contract
 * (`predecessor_id`) who is the balance owner.
 *
 * @param escrow_account_id
 * @param allowance
 */
export function set_allowance(escrow_account_id: AccountId, allowance: Amount): void {
  assert(allowance > u128.Zero, ERR_INVALID_AMOUNT)

  const owner_id = context.predecessor
  allowanceRegistry.set(keyFrom(owner_id, escrow_account_id), u128.from(allowance))
}

/**
 * Transfers the `amount` of tokens from `owner_id` to the `new_owner_id`.
 * Requirements:
 * - `amount` should be a positive integer.
 * - `owner_id` should have balance on the account greater or equal than the transfer `amount`.
 * - If this function is called by an escrow account (`owner_id != predecessor_id`),
 *   then the allowance of the caller of the function (`predecessor_id`) on
 *   the account of `owner_id` should be greater or equal than the transfer `amount`.
 * @param owner_id
 * @param new_owner_id
 * @param amount
 */
export function transfer_from(owner_id: AccountId, new_owner_id: AccountId,  amount: Amount): void {
  assert(amount > u128.Zero, ERR_INVALID_AMOUNT)
  assert(balanceRegistry.contains(owner_id), ERR_INVALID_ACCOUNT)
  assert(balanceRegistry.getSome(owner_id) >= amount, ERR_INSUFFICIENT_BALANCE)

  if(owner_id != context.predecessor) {
    const key = keyFrom(owner_id, context.predecessor)
    assert(allowanceRegistry.contains(key), ERR_INVALID_ESCROW_ACCOUNT)

    const allowance = allowanceRegistry.getSome(key)
    assert(allowance >= amount, ERR_INSUFFICIENT_ESCROW_BALANCE)

    allowanceRegistry.set(key, u128.sub(allowance, amount))
  }

  const balanceOfOwner = balanceRegistry.getSome(owner_id)
  const balanceOfNewOwner = balanceRegistry.get(new_owner_id, u128.Zero)!

  balanceRegistry.set(owner_id, u128.sub(balanceOfOwner, amount))
  balanceRegistry.set(new_owner_id, u128.add(balanceOfNewOwner, amount))
}

/**
 * Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to
 * `new_owner_id`.
 * Note: This call behaves as if `transfer_from` with `owner_id` equal to the caller
 * of the contract (`predecessor_id`).
 * @param new_owner_id
 * @param amount
 */
// it bugs me that we have both of these when we decided we didn't need both for NFT
// but i guess that's part of the spec
export function transfer(new_owner_id: AccountId, amount: Amount): void {
  const owner_id = context.predecessor
  transfer_from(owner_id, new_owner_id, amount)
}

/****************/
/* VIEW METHODS */
/****************/

/**
 * Returns total supply of tokens.
 */
export function get_total_supply(): u128 {
  return TOTAL_SUPPLY
}

/**
 * Returns balance of the `owner_id` account.
 * @param owner_id
 */
// do we need a warning similar to the one for get_allowance?
export function get_balance(owner_id: AccountId): u128 {
  assert(balanceRegistry.contains(owner_id), ERR_INVALID_ACCOUNT)
  return balanceRegistry.getSome(owner_id)
}
/**
 * Returns current allowance of `escrow_account_id` for the account of `owner_id`.
 *
 * NOTE: Other contracts should not rely on this information, because by the moment a contract
 * receives this information, the allowance may already be changed by the owner.
 * So this method should only be used on the front-end to see the current allowance.
 */
export function get_allowance(owner_id: AccountId, escrow_account_id: AccountId): u128 {
  const key = keyFrom(owner_id, escrow_account_id)
  return allowanceRegistry.get(key, u128.Zero)!
}
