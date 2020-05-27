import * as model from '../models'

// these are all implicitly tested in the main tests; I don't think it adds much clarity to include these tests
describe('interface', () => {
  it('TOTAL_SUPPLY should exist', () => {
    expect(isDefined(model.TOTAL_SUPPLY)).toBeTruthy()
  })

  it('allowanceRegistry should exist', () => {
    expect(isDefined(model.allowanceRegistry)).toBeTruthy()
  })
  it('balanceRegistry should exist', () => {
    expect(isDefined(model.balanceRegistry)).toBeTruthy()
  })
  it('getAllowanceKey should exist', () => {
    expect(isDefined(model.getAllowanceKey)).toBeTruthy()
  })
})

// everything else in the model file seems like an implementation detail
