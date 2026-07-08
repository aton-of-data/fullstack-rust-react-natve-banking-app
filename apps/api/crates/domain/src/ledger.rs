//! Double-entry ledger drafts and balancing rules.
//!
//! This module belongs to the domain layer. It owns in-memory ledger entry
//! drafts, debit/credit signed-amount semantics, and balance validation for a
//! transfer. It does **not** open database transactions, lock accounts, or
//! update the `account_balances` projection — those belong to the persistence
//! adapter.
//!
//! # Financial invariants
//!
//! - A completed P2P transfer produces exactly one debit (sender) and one
//!   credit (recipient) of equal magnitude.
//! - Debits contribute a negative signed amount; credits contribute positive.
//! - The signed sum of entries for a transfer must be zero
//!   ([`validate_balanced`]).
//! - Balance projection rows must remain reconcilable via
//!   [`reconstruct_balance`] over append-only ledger entries.

use uuid::Uuid;

use crate::currency::Currency;
use crate::errors::DomainError;
use crate::money::Money;

/// Direction of a ledger entry in the double-entry journal.
///
/// Debits reduce the account's reconstructed balance; credits increase it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerDirection {
    /// Outflow from the accounted account (sender side of a P2P transfer).
    Debit,
    /// Inflow to the accounted account (recipient side of a P2P transfer).
    Credit,
}

/// Draft ledger entry before persistence.
///
/// `amount_minor` is always a non-negative magnitude; the sign is derived from
/// [`LedgerDirection`] via [`LedgerEntryDraft::signed_amount`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntryDraft {
    /// Account this entry posts against.
    pub account_id: Uuid,
    /// Transfer that produced this entry.
    pub transfer_id: Uuid,
    /// Absolute amount in minor units (never negative).
    pub amount_minor: i64,
    /// Debit or credit.
    pub direction: LedgerDirection,
    /// Currency of the posting (must match the transfer).
    pub currency: Currency,
}

impl LedgerEntryDraft {
    /// Signed amount for balancing: debits negative, credits positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use ficus_domain::currency::Currency;
    /// use ficus_domain::ledger::{LedgerDirection, LedgerEntryDraft};
    /// use uuid::Uuid;
    ///
    /// let debit = LedgerEntryDraft {
    ///     account_id: Uuid::nil(),
    ///     transfer_id: Uuid::nil(),
    ///     amount_minor: 100,
    ///     direction: LedgerDirection::Debit,
    ///     currency: Currency::Usd,
    /// };
    /// assert_eq!(debit.signed_amount(), -100);
    /// ```
    pub fn signed_amount(&self) -> i64 {
        match self.direction {
            LedgerDirection::Debit => -self.amount_minor,
            LedgerDirection::Credit => self.amount_minor,
        }
    }
}

/// Builds a balanced debit/credit pair for a P2P transfer.
///
/// The sender receives a debit and the recipient a credit of equal magnitude.
/// The pair is validated with [`validate_balanced`] before return.
///
/// # Errors
///
/// Returns [`DomainError::UnbalancedLedger`] if the constructed pair somehow
/// fails balancing (defensive; should not occur for equal-magnitude pairs).
///
/// # Examples
///
/// ```
/// use ficus_domain::currency::Currency;
/// use ficus_domain::ledger::{build_transfer_entries, validate_balanced, LedgerDirection};
/// use ficus_domain::money::Money;
/// use uuid::Uuid;
///
/// let amount = Money::transfer_amount(250, Currency::Usd).unwrap();
/// let entries = build_transfer_entries(
///     Uuid::nil(),
///     Uuid::from_u128(1),
///     Uuid::from_u128(2),
///     &amount,
/// )
/// .unwrap();
/// assert_eq!(entries[0].direction, LedgerDirection::Debit);
/// assert_eq!(entries[1].direction, LedgerDirection::Credit);
/// validate_balanced(&entries).unwrap();
/// ```
pub fn build_transfer_entries(
    transfer_id: Uuid,
    sender_account_id: Uuid,
    recipient_account_id: Uuid,
    amount: &Money,
) -> Result<[LedgerEntryDraft; 2], DomainError> {
    let minor = amount.amount_minor();
    let currency = amount.currency();
    let debit = LedgerEntryDraft {
        account_id: sender_account_id,
        transfer_id,
        amount_minor: minor,
        direction: LedgerDirection::Debit,
        currency,
    };
    let credit = LedgerEntryDraft {
        account_id: recipient_account_id,
        transfer_id,
        amount_minor: minor,
        direction: LedgerDirection::Credit,
        currency,
    };
    validate_balanced(&[debit.clone(), credit.clone()])?;
    Ok([debit, credit])
}

/// Validates that ledger entries sum to zero under signed-amount semantics.
///
/// # Errors
///
/// Returns [`DomainError::UnbalancedLedger`] when the signed sum is not zero.
///
/// # Examples
///
/// ```
/// use ficus_domain::currency::Currency;
/// use ficus_domain::ledger::{validate_balanced, LedgerDirection, LedgerEntryDraft};
/// use uuid::Uuid;
///
/// let id = Uuid::nil();
/// let entries = [
///     LedgerEntryDraft {
///         account_id: id,
///         transfer_id: id,
///         amount_minor: 10,
///         direction: LedgerDirection::Debit,
///         currency: Currency::Usd,
///     },
///     LedgerEntryDraft {
///         account_id: id,
///         transfer_id: id,
///         amount_minor: 10,
///         direction: LedgerDirection::Credit,
///         currency: Currency::Usd,
///     },
/// ];
/// assert!(validate_balanced(&entries).is_ok());
/// ```
pub fn validate_balanced(entries: &[LedgerEntryDraft]) -> Result<(), DomainError> {
    let sum: i64 = entries.iter().map(|e| e.signed_amount()).sum();
    if sum != 0 {
        return Err(DomainError::UnbalancedLedger);
    }
    Ok(())
}

/// Reconstructs an account balance by summing signed ledger amounts.
///
/// Used by reconciliation helpers and tests. Production reads preferably use
/// the `account_balances` projection, which must match this sum for each
/// account after every committed transfer.
pub fn reconstruct_balance(account_id: Uuid, entries: &[LedgerEntryDraft]) -> i64 {
    entries
        .iter()
        .filter(|e| e.account_id == account_id)
        .map(|e| e.signed_amount())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn transfer_entries_balance() {
        let amount = Money::transfer_amount(100, Currency::Usd).unwrap();
        let tid = Uuid::new_v4();
        let sender = Uuid::new_v4();
        let recipient = Uuid::new_v4();
        let entries = build_transfer_entries(tid, sender, recipient, &amount).unwrap();
        assert_eq!(entries[0].direction, LedgerDirection::Debit);
        assert_eq!(entries[1].direction, LedgerDirection::Credit);
        validate_balanced(&entries).unwrap();
    }

    proptest! {
        #[test]
        fn valid_entries_always_balance(amount in 1i64..10_000_000) {
            let money = Money::transfer_amount(amount, Currency::Usd).unwrap();
            let entries = build_transfer_entries(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                &money,
            ).unwrap();
            prop_assert!(validate_balanced(&entries).is_ok());
        }
    }
}
