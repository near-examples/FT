import * as model from '../models'

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
