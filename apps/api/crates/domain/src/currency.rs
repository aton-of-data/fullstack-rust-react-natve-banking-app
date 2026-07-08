//! Supported currencies for money movement.
//!
//! This module belongs to the domain layer. It owns the closed set of currencies
//! the system will book. Expanding currencies requires coordinated ledger,
//! balance, and HTTP contract changes — do not treat currency as a free-form
//! string at the domain boundary.
//!
//! Currently only USD is supported. Amounts are always integer minor units for
//! the currency (USD cents).

use std::fmt;
use std::str::FromStr;

use crate::errors::DomainError;

/// Supported fiat currencies.
///
/// # Examples
///
/// ```
/// use ficus_domain::currency::Currency;
/// use std::str::FromStr;
///
/// assert_eq!(Currency::from_str("usd").unwrap(), Currency::Usd);
/// assert_eq!(Currency::Usd.code(), "USD");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Currency {
    /// United States dollar (ISO-4217 `USD`), minor unit = cent.
    Usd,
}

impl Currency {
    /// Returns the ISO-4217 alphabetic code.
    pub fn code(&self) -> &'static str {
        match self {
            Currency::Usd => "USD",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = DomainError;

    /// Parses a currency code (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::UnsupportedCurrency`] for any code other than
    /// `USD`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::Usd),
            other => Err(DomainError::UnsupportedCurrency(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usd() {
        assert_eq!(Currency::from_str("usd").unwrap(), Currency::Usd);
    }
}
