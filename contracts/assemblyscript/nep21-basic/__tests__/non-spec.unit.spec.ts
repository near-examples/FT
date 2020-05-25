import * as contract from '../main'
import * as nonSpec from '../non-spec'

//////////////////////////////
// non-spec interface tests //
//////////////////////////////

const alice = 'alice'
const bob = 'bob'
const carol = 'carol'

it('should mint tokens up to MAX_SUPPLY', () => {
  expect(() => {
    let limit = contract.get_total_supply()
    nonSpec.mint(alice)
    expect(contract.get_balance(alice)).toBe(limit)
  }).not.toThrow()

  // minting can only happen once
  expect(() => {
    nonSpec.mint(bob)
  }).toThrow()
})
