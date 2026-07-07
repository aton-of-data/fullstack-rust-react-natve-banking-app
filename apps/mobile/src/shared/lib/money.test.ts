import { describe, expect, it } from 'vitest';

import { formatMoney, isValidAmountInput, majorToMinorUnits } from './money';

describe('majorToMinorUnits', () => {
  it('converts decimal dollars to cents', () => {
    expect(majorToMinorUnits('12.34')).toBe('1234');
  });

  it('pads single decimal digit', () => {
    expect(majorToMinorUnits('5.5')).toBe('550');
  });

  it('handles whole numbers', () => {
    expect(majorToMinorUnits('10')).toBe('1000');
  });
});

describe('formatMoney', () => {
  it('formats USD minor units for display', () => {
    expect(formatMoney('1234', 'USD')).toBe('$12.34');
  });
});

describe('isValidAmountInput', () => {
  it('accepts positive amounts', () => {
    expect(isValidAmountInput('1.00')).toBe(true);
  });

  it('rejects zero', () => {
    expect(isValidAmountInput('0')).toBe(false);
  });

  it('rejects invalid strings', () => {
    expect(isValidAmountInput('abc')).toBe(false);
  });
});
