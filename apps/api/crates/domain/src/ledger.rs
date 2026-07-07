use uuid::Uuid;

use crate::currency::Currency;
use crate::errors::DomainError;
use crate::money::Money;

/// Direction of a ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerDirection {
    Debit,
    Credit,
}

/// Draft ledger entry before persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntryDraft {
    pub account_id: Uuid,
    pub transfer_id: Uuid,
    pub amount_minor: i64,
    pub direction: LedgerDirection,
    pub currency: Currency,
}

impl LedgerEntryDraft {
    /// Signed amount: debits negative, credits positive.
    pub fn signed_amount(&self) -> i64 {
        match self.direction {
            LedgerDirection::Debit => -self.amount_minor,
            LedgerDirection::Credit => self.amount_minor,
        }
    }
}

/// Builds balanced debit/credit pair for a transfer.
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

/// Validates that ledger entries sum to zero.
pub fn validate_balanced(entries: &[LedgerEntryDraft]) -> Result<(), DomainError> {
    let sum: i64 = entries.iter().map(|e| e.signed_amount()).sum();
    if sum != 0 {
        return Err(DomainError::UnbalancedLedger);
    }
    Ok(())
}

/// Reconstructs balance from ledger entries for an account.
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
