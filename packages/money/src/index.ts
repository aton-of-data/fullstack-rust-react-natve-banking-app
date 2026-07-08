/**
 * Canonical wire-format representation of a signed integer minor-unit amount.
 */
export type MinorUnits = string;

/**
 * Error thrown when minor-unit parsing or arithmetic fails.
 */
export class MoneyError extends Error {
  /**
   * Creates a money validation or arithmetic error.
   * @param message Human-readable failure reason.
   */
  constructor(message: string) {
    super(message);
    this.name = 'MoneyError';
  }
}

/**
 * Options for {@link formatMinorUnits}.
 */
export interface FormatMinorUnitsOptions {
  /** ISO 4217 currency code used for symbol selection. */
  currency: string;
  /** BCP 47 locale for grouping and separators. Defaults to `en-US`. */
  locale?: string;
  /** Number of decimal places implied by the minor unit (e.g. 2 for USD cents). Defaults to `2`. */
  minorUnitDigits?: number;
}

const DEFAULT_MINOR_UNIT_DIGITS = 2;
const DEFAULT_LOCALE = 'en-US';

/**
 * Normalizes a minor-unit value to a canonical decimal string without fractional minor units.
 *
 * @param value Minor units as string or bigint.
 * @returns Canonical non-negative integer string.
 * @throws {MoneyError} When the value is empty, fractional, or out of range.
 */
export function normalizeMinorUnits(value: string | bigint): MinorUnits {
  if (typeof value === 'bigint') {
    if (value < 0n) {
      throw new MoneyError('minor units cannot be negative');
    }
    return value.toString(10);
  }

  const trimmed = value.trim();
  if (trimmed.length === 0) {
    throw new MoneyError('minor units cannot be empty');
  }
  if (trimmed.startsWith('-')) {
    throw new MoneyError('minor units cannot be negative');
  }
  if (trimmed.includes('.')) {
    throw new MoneyError('fractional minor units are not allowed');
  }
  if (!/^\d+$/.test(trimmed)) {
    throw new MoneyError(`invalid minor units: ${value}`);
  }

  try {
    return BigInt(trimmed).toString(10);
  } catch {
    throw new MoneyError(`invalid minor units: ${value}`);
  }
}

/**
 * Parses user or API input into canonical minor-unit string form.
 *
 * @param input Raw minor-unit string (integer only).
 * @returns Canonical minor-unit string.
 * @throws {MoneyError} When input is not a valid non-negative integer string.
 */
export function parseMinorUnits(input: string): MinorUnits {
  return normalizeMinorUnits(input);
}

/**
 * Formats minor units for display using locale-aware grouping.
 * Uses bigint division only — no floating-point arithmetic on amounts.
 *
 * @param minor Minor units as string or bigint.
 * @param options Formatting options including currency code.
 * @returns Locale-formatted major-unit display string (e.g. `$12.34`).
 * @throws {MoneyError} When minor units are invalid.
 */
export function formatMinorUnits(minor: string | bigint, options: FormatMinorUnitsOptions): string {
  const normalized = normalizeMinorUnits(minor);
  const minorBigInt = BigInt(normalized);
  const minorUnitDigits = options.minorUnitDigits ?? DEFAULT_MINOR_UNIT_DIGITS;
  const locale = options.locale ?? DEFAULT_LOCALE;
  const divisor = 10n ** BigInt(minorUnitDigits);
  const major = minorBigInt / divisor;
  const remainder = minorBigInt % divisor;
  const fraction = remainder.toString(10).padStart(minorUnitDigits, '0');
  const majorFormatted = formatIntegerWithGrouping(major, locale);
  const decimalSeparator = getDecimalSeparator(locale);
  const formattedNumber = `${majorFormatted}${decimalSeparator}${fraction}`;

  const symbol = getCurrencySymbol(options.currency, locale);
  return symbol ? `${symbol}${formattedNumber}` : `${formattedNumber} ${options.currency}`;
}

/**
 * Whether the current runtime exposes `Intl.NumberFormat.prototype.formatToParts`.
 * Hermes / some React Native Intl polyfills omit it.
 *
 * @returns True when formatToParts is available.
 */
function supportsFormatToParts(): boolean {
  return typeof Intl.NumberFormat.prototype.formatToParts === 'function';
}

/**
 * Formats an integer major-unit component with locale grouping, without floats.
 *
 * @param value Integer major units.
 * @param locale BCP 47 locale.
 * @returns Grouped integer string.
 */
function formatIntegerWithGrouping(value: bigint, locale: string): string {
  if (value <= BigInt(Number.MAX_SAFE_INTEGER)) {
    return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(Number(value));
  }

  const digits = value.toString(10);
  let groupingSeparator = ',';
  if (supportsFormatToParts()) {
    groupingSeparator =
      new Intl.NumberFormat(locale).formatToParts(1000).find((part) => part.type === 'group')
        ?.value ?? ',';
  }

  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, groupingSeparator);
}

/**
 * Resolves the locale decimal separator without floating-point formatting.
 *
 * @param locale BCP 47 locale.
 * @returns Decimal separator character.
 */
function getDecimalSeparator(locale: string): string {
  if (!supportsFormatToParts()) {
    return '.';
  }
  const parts = new Intl.NumberFormat(locale).formatToParts(1.1);
  return parts.find((part) => part.type === 'decimal')?.value ?? '.';
}

/**
 * Adds two minor-unit values using bigint arithmetic.
 *
 * @param left First operand.
 * @param right Second operand.
 * @returns Sum as canonical minor-unit string.
 * @throws {MoneyError} When operands are invalid or overflow occurs.
 */
export function addMinor(left: string | bigint, right: string | bigint): MinorUnits {
  const a = BigInt(normalizeMinorUnits(left));
  const b = BigInt(normalizeMinorUnits(right));
  const sum = a + b;
  if (sum < a) {
    throw new MoneyError('minor-unit addition overflow');
  }
  return sum.toString(10);
}

/**
 * Subtracts the right operand from the left using bigint arithmetic.
 *
 * @param left Minuend.
 * @param right Subtrahend.
 * @returns Difference as canonical minor-unit string.
 * @throws {MoneyError} When operands are invalid or the result would be negative.
 */
export function subtractMinor(left: string | bigint, right: string | bigint): MinorUnits {
  const a = BigInt(normalizeMinorUnits(left));
  const b = BigInt(normalizeMinorUnits(right));
  if (b > a) {
    throw new MoneyError('minor-unit subtraction would be negative');
  }
  return (a - b).toString(10);
}

/**
 * Resolves a display symbol for a currency code when available.
 *
 * @param currency ISO 4217 currency code.
 * @param locale BCP 47 locale.
 * @returns Currency symbol or empty string when unavailable.
 */
function getCurrencySymbol(currency: string, locale: string): string {
  if (!supportsFormatToParts()) {
    if (currency === 'USD') {
      return '$';
    }
    return '';
  }
  const parts = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
    currencyDisplay: 'narrowSymbol',
  }).formatToParts(0);
  return parts.find((part) => part.type === 'currency')?.value ?? '';
}
