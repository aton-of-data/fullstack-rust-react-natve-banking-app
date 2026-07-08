//! Integer minor-unit money value objects.
//!
//! This module belongs to the domain layer. It owns construction, parsing, and
//! checked arithmetic for [`Money`]. It does not own currency catalogs beyond
//! accepting a [`Currency`], ledger posting, or persistence of balances.
//!
//! # Financial invariants
//!
//! - Amounts are integer **minor units** (for USD, cents).
//! - Floating-point types must never represent money in this codebase.
//! - General balances may be zero; transfer amounts must be strictly positive.
//! - Arithmetic uses checked operations and fails on overflow or currency
//!   mismatch rather than wrapping silently.

use std::fmt;
use std::str::FromStr;

use crate::currency::Currency;
use crate::errors::DomainError;

/// Integer minor-unit money value with an associated currency.
///
/// `Money` never uses floating point. Callers in the application and
/// persistence layers pass minor-unit integers (or decimal strings of those
/// integers) and keep ledger postings and balance projections consistent with
/// this representation.
///
/// # Examples
///
/// ```
/// use ficus_domain::currency::Currency;
/// use ficus_domain::money::Money;
///
/// let amount = Money::transfer_amount(1250, Currency::Usd).unwrap();
/// assert_eq!(amount.amount_minor(), 1250);
/// assert_eq!(amount.currency().code(), "USD");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money {
    amount_minor: i64,
    currency: Currency,
}

impl Money {
    /// Creates money from minor units.
    ///
    /// Rejects negative amounts. Zero is allowed for balance projections and
    /// arithmetic intermediates; use [`Money::transfer_amount`] when the value
    /// must be a payable transfer magnitude.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidMoney`] when `amount_minor` is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::money::Money;
    ///
    /// assert!(Money::from_minor(0, Currency::Usd).is_ok());
    /// assert!(Money::from_minor(-1, Currency::Usd).is_err());
    /// ```
    pub fn from_minor(amount_minor: i64, currency: Currency) -> Result<Self, DomainError> {
        if amount_minor < 0 {
            return Err(DomainError::InvalidMoney(
                "amount cannot be negative".into(),
            ));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Creates a strictly positive transfer amount in minor units.
    ///
    /// Transfers must move a positive quantity; zero and negative values are
    /// rejected before any ledger or balance mutation occurs upstream.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidMoney`] when `amount_minor` is zero or
    /// negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::money::Money;
    ///
    /// assert!(Money::transfer_amount(1, Currency::Usd).is_ok());
    /// assert!(Money::transfer_amount(0, Currency::Usd).is_err());
    /// ```
    pub fn transfer_amount(amount_minor: i64, currency: Currency) -> Result<Self, DomainError> {
        if amount_minor <= 0 {
            return Err(DomainError::InvalidMoney(
                "transfer amount must be positive".into(),
            ));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Parses a decimal string of **integer** minor units.
    ///
    /// The string must not contain a decimal point, leading minus, or empty
    /// content. Fractional minor units are rejected to avoid accidental
    /// floating-point style inputs from HTTP clients.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidMoney`] for empty, negative, fractional,
    /// or otherwise unparsable input.
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::money::Money;
    ///
    /// let amount = Money::parse_minor("1250", Currency::Usd).unwrap();
    /// assert_eq!(amount.amount_minor(), 1250);
    /// assert!(Money::parse_minor("12.50", Currency::Usd).is_err());
    /// ```
    pub fn parse_minor(s: &str, currency: Currency) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidMoney("empty amount".into()));
        }
        if trimmed.starts_with('-') {
            return Err(DomainError::InvalidMoney("negative amount".into()));
        }
        if trimmed.contains('.') {
            return Err(DomainError::InvalidMoney(
                "fractional minor units not allowed".into(),
            ));
        }
        let amount_minor: i64 = trimmed
            .parse()
            .map_err(|_| DomainError::InvalidMoney(format!("invalid amount: {trimmed}")))?;
        Self::from_minor(amount_minor, currency)
    }

    /// Returns the amount in integer minor units.
    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    /// Returns the associated currency.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Subtracts another money value of the same currency.
    ///
    /// # Errors
    ///
    /// - [`DomainError::UnsupportedCurrency`] on currency mismatch
    /// - [`DomainError::InvalidMoney`] on overflow/underflow of `i64`
    /// - [`DomainError::InvalidMoney`] when the result would be negative
    ///   (via [`Money::from_minor`])
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::money::Money;
    ///
    /// let a = Money::from_minor(500, Currency::Usd).unwrap();
    /// let b = Money::from_minor(200, Currency::Usd).unwrap();
    /// assert_eq!(a.checked_sub(&b).unwrap().amount_minor(), 300);
    /// ```
    pub fn checked_sub(&self, other: &Money) -> Result<Money, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::UnsupportedCurrency(format!(
                "currency mismatch: {} vs {}",
                self.currency, other.currency
            )));
        }
        let result = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or_else(|| DomainError::InvalidMoney("overflow".into()))?;
        Self::from_minor(result, self.currency)
    }

    /// Adds another money value of the same currency.
    ///
    /// # Errors
    ///
    /// - [`DomainError::UnsupportedCurrency`] on currency mismatch
    /// - [`DomainError::InvalidMoney`] on `i64` overflow
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::money::Money;
    ///
    /// let a = Money::from_minor(100, Currency::Usd).unwrap();
    /// let b = Money::from_minor(50, Currency::Usd).unwrap();
    /// assert_eq!(a.checked_add(&b).unwrap().amount_minor(), 150);
    /// ```
    pub fn checked_add(&self, other: &Money) -> Result<Money, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::UnsupportedCurrency(format!(
                "currency mismatch: {} vs {}",
                self.currency, other.currency
            )));
        }
        let result = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or_else(|| DomainError::InvalidMoney("overflow".into()))?;
        Self::from_minor(result, self.currency)
    }

    /// Returns zero money for a currency (valid balance projection seed).
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount_minor, self.currency)
    }
}

impl FromStr for Money {
    type Err = DomainError;

    /// Parses minor units as USD via [`Money::parse_minor`].
    ///
    /// Prefer the currency-aware [`Money::parse_minor`] when the request
    /// currency is known explicitly.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_minor(s, Currency::Usd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn add_sub_roundtrip(a in 0i64..1_000_000, b in 0i64..1_000_000) {
            let m1 = Money::from_minor(a, Currency::Usd).unwrap();
            let m2 = Money::from_minor(b, Currency::Usd).unwrap();
            let sum = m1.checked_add(&m2).unwrap();
            let back = sum.checked_sub(&m2).unwrap();
            prop_assert_eq!(back.amount_minor(), a);
        }

        #[test]
        fn rejects_negative(s in 1i64..1_000_000) {
            prop_assert!(Money::from_minor(-s, Currency::Usd).is_err());
        }

        #[test]
        fn rejects_malformed(input in r"-?[0-9]*\.[0-9]+") {
            prop_assert!(Money::parse_minor(&input, Currency::Usd).is_err());
        }
    }

    #[test]
    fn transfer_amount_rejects_zero() {
        assert!(Money::transfer_amount(0, Currency::Usd).is_err());
    }
}
