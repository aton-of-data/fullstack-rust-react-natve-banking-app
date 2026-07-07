use std::fmt;
use std::str::FromStr;

use crate::currency::Currency;
use crate::errors::DomainError;

/// Integer minor-unit money value. Never uses floating point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money {
    amount_minor: i64,
    currency: Currency,
}

impl Money {
    /// Creates money from minor units with validation.
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

    /// Creates positive transfer amount.
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

    /// Parses a decimal string of minor units.
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

    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Subtracts another money value of the same currency.
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

    /// Returns zero money for a currency.
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
