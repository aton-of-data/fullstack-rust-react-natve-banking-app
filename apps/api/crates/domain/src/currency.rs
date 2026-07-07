use std::fmt;
use std::str::FromStr;

use crate::errors::DomainError;

/// Supported fiat currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Currency {
    Usd,
}

impl Currency {
    /// Returns the ISO-4217 code.
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
