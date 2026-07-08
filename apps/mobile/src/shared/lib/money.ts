import { formatMinorUnits, parseMinorUnits, type FormatMinorUnitsOptions } from '@ficus/money';

/**
 * Converts a major-unit decimal string (e.g. "12.34") to minor units wire string.
 *
 * @param majorUnit Amount in major units as decimal string.
 * @param minorUnitDigits Number of decimal places (default 2).
 * @returns Canonical minor-unit string.
 */
export function majorToMinorUnits(majorUnit: string, minorUnitDigits = 2): string {
  const trimmed = majorUnit.trim();
  if (!trimmed) {
    return '0';
  }

  const [whole = '0', fraction = ''] = trimmed.split('.');
  const paddedFraction = fraction.padEnd(minorUnitDigits, '0').slice(0, minorUnitDigits);
  const combined = `${whole}${paddedFraction}`.replace(/^0+(?=\d)/, '');
  return parseMinorUnits(combined || '0');
}

/**
 * Formats minor units for display using @ficus/money.
 *
 * @param minor Minor units wire string.
 * @param currency ISO 4217 currency code.
 * @param locale BCP 47 locale.
 * @returns Formatted display string.
 */
export function formatMoney(minor: string, currency: string, locale?: string): string {
  const options: FormatMinorUnitsOptions = locale ? { currency, locale } : { currency };
  return formatMinorUnits(minor, options);
}

/**
 * Validates that an amount input is a positive major-unit decimal.
 * Uses integer minor-unit comparison — never floating-point arithmetic.
 *
 * @param input Amount input string.
 * @returns True when valid and greater than zero.
 */
export function isValidAmountInput(input: string): boolean {
  const trimmed = input.trim();
  if (!trimmed) {
    return false;
  }
  if (!/^\d+(\.\d{0,2})?$/.test(trimmed)) {
    return false;
  }
  try {
    const minor = majorToMinorUnits(trimmed);
    return BigInt(minor) > 0n;
  } catch {
    return false;
  }
}
