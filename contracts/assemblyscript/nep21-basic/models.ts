import { u128, PersistentMap } from 'near-sdk-as'
import { AccountId, AllowanceKey, Amount } from './types'

/**************************/
/* DATA TYPES AND STORAGE */
/**************************/

export const TOTAL_SUPPLY = u128.from(10)

export const allowanceRegistry = new PersistentMap<AllowanceKey, Amount>('a')
export const balanceRegistry = new PersistentMap<AccountId, Amount>('b')

/**
 * Generate a consistent key format for looking up which `owner_id` has given
 * an `escrow_id` some `allowance` to transfer on their behalf
 * @param owner_id
 * @param escrow_id
 */
export function getAllowanceKey(owner_id: AccountId, escrow_id: AccountId): string {
  return owner_id + ":" + escrow_id
}
