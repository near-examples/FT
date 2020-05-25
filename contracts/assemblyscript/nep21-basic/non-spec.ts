import { AccountId } from './types'
import { TOTAL_SUPPLY, balanceRegistry } from './models'
import { storage } from 'near-sdk-as'

/******************/
/* ERROR MESSAGES */
/******************/

export const ERR_INVALID_AMOUNT = 'Allowance must be greater than zero'
export const ERR_INVALID_ACCOUNT = 'Account not found in registry'
export const ERR_INVALID_ESCROW_ACCOUNT = 'Escrow account not found in registry'
export const ERR_INSUFFICIENT_BALANCE = 'Account does not have enough balance for this transaction' // prettier-ignore
export const ERR_INSUFFICIENT_ESCROW_BALANCE = 'Escrow account does not have enough allowance for this transaction' // prettier-ignore
export const ERR_TOKEN_ALREADY_MINTED = 'Token has previously been minted'

/********************/
/* NON-SPEC METHODS */
/********************/

export function mint(owner_id: AccountId): void {
  // check if previously minted
  assert(!storage.contains('minted'), ERR_TOKEN_ALREADY_MINTED)

  // assign ownership
  balanceRegistry.set(owner_id, TOTAL_SUPPLY)

  // record that minting is complete
  storage.set('minted', true)
}
