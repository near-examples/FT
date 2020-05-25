import { u128 } from 'near-sdk-as'

export type AccountId = string
export type AllowanceKey = string // format is Owner<AccountId>:Escrow<AccountId>
export type Amount = u128
