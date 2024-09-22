use std::collections::HashMap;

use serde::Serialize;

use crate::transaction::{Transaction, TransactionType};
use crate::{
    primitives::{AccountID, Coin, TxID, PRECISION},
    transaction::AncestorState,
};

#[derive(Clone, Eq, PartialEq, Debug, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    id: AccountID,
    available: Coin,
    held: Coin,
    total: Coin,
    locked: bool,
    #[serde(skip)]
    txs: HashMap<TxID, Vec<Transaction>>, // DB for transactions stored by TxID, Vec represents successful transactions
    #[serde(skip)]
    failed: Vec<Transaction>, // DB for failed transactions
}

impl Account {
    pub fn new(id: AccountID) -> Self {
        Self {
            id,
            available: Coin::new(0, PRECISION),
            held: Coin::new(0, PRECISION),
            total: Coin::new(0, PRECISION),
            locked: false,
            txs: HashMap::new(),
            failed: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_available(self, available: Coin) -> Self {
        Self { available, ..self }
    }

    #[allow(dead_code)]
    pub fn set_held(self, held: Coin) -> Self {
        Self { held, ..self }
    }

    #[allow(dead_code)]
    pub fn set_total(self, total: Coin) -> Self {
        Self { total, ..self }
    }

    #[allow(dead_code)]
    pub fn set_locked(self, locked: bool) -> Self {
        Self { locked, ..self }
    }

    #[allow(dead_code)]
    pub fn check_amounts(&self, other: &Self) -> bool {
        self.id == other.id
            && self.available == other.available
            && self.held == other.held
            && self.total == other.total
            && self.locked == other.locked
    }

    /// Process transaction:
    ///
    /// if there are no previous transactions with this id -> insert valid
    ///
    /// if there are previous transactions with this id -> check latest transaction on account to see if it is valid ancestor
    ///
    /// if latest transaction on account is valid ancestor -> insert valid
    ///
    /// if latest transaction on account is not valid ancestor -> insert failed
    ///
    /// if there are no previous transactions with this id -> insert valid    
    pub async fn process(&mut self, tx: &Transaction) {
        let mut success = false;

        if !self.is_locked() {
            match self.txs.get_mut(&tx.id()) {
                Some(previous_txs) => {
                    // if there are previous transactions
                    // we need to check latest transaction on account to see if it is valid ancestor
                    if let Some(last) = previous_txs.last() {
                        if tx.valid_ancestor(last) == AncestorState::Valid {
                            // first transaction should be deposit or withdrawal and we can take amount from it
                            let first = previous_txs[0].clone(); //XXX: as we checked last(), there should be first
                            previous_txs.push(tx.clone());
                            self.calc_transaction(first.amount(), &tx.tx_type(), &first.tx_type());
                            success = true;
                        }
                    }
                }
                None => {
                    // if there is no previous transactions with this id -> insert valid
                    if tx.tx_type() == TransactionType::Deposit
                        || tx.tx_type() == TransactionType::Withdrawal
                    {
                        self.txs.insert(tx.id(), vec![tx.clone()]); // provides guarantee that first transaction is deposit or withdrawal
                        self.calc_transaction(tx.amount(), &tx.tx_type(), &tx.tx_type());
                        success = true;
                    }
                }
            };
        }

        if !success {
            self.failed.push(tx.clone());
        }
    }

    /// calculate account state after transaction
    ///
    fn calc_transaction(
        &mut self,
        amount: Coin,
        tx_type: &TransactionType,
        parent_type: &TransactionType,
    ) {
        // we take strait amount from parent transaction if it is deposit
        // we take negative amount from parent transaction if it is withdrawal
        // then dispute, resolve and chargeback will perform correctly with this amount
        let amount = match parent_type {
            TransactionType::Deposit => amount,
            TransactionType::Withdrawal => -amount, // negate amount for dependent transactions
            _ => Coin::new(0, PRECISION),
        };

        match tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal => self.deposit(amount),
            TransactionType::Dispute => self.dispute(amount),
            TransactionType::Resolve => self.resolve(amount),
            TransactionType::Chargeback => self.chargeback(amount),
        }
    }

    /// deposit Coins onto account
    ///
    /// withdraw works the same way with negative values
    fn deposit(&mut self, amount: Coin) {
        self.available += amount;
        self.total += amount;
    }

    /// dispute Coins from account
    fn dispute(&mut self, amount: Coin) {
        self.available -= amount;
        self.held += amount;
    }

    /// resolve Coins to account
    fn resolve(&mut self, amount: Coin) {
        self.available += amount;
        self.held -= amount;
    }

    /// chargeback Coins from account
    fn chargeback(&mut self, amount: Coin) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }

    /// check if account is locked
    fn is_locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        account.deposit(Coin::new(11, 1));
        assert_eq!(account.available, Coin::new(11, 1));
        assert_eq!(account.held, Coin::new(0, 4));
        assert_eq!(account.total, Coin::new(11, 1));
        assert_eq!(account.locked, false);
    }

    //   #[test]
    //   fn test_withdraw() {
    //     let mut account = Account::new(1);
    //     account.withdraw(Coin::new(11,1));
    //     assert_eq!(account.available, -Coin::new(11,1));
    //     assert_eq!(account.held, Coin::new(0,4));
    //     assert_eq!(account.total, -Coin::new(11,1));
    //     assert_eq!(account.locked, false);
    //   }

    #[test]
    fn test_dispute() {
        let mut account = Account::new(1);
        account.dispute(Coin::new(11, 1));
        assert_eq!(account.available, -Coin::new(11, 1));
        assert_eq!(account.held, Coin::new(11, 1));
        assert_eq!(account.total, Coin::new(0, 4));
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::new(1);
        account.resolve(Coin::new(11, 1));
        assert_eq!(account.available, Coin::new(11, 1));
        assert_eq!(account.held, -Coin::new(11, 1));
        assert_eq!(account.total, Coin::new(0, 4));
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account::new(1);
        account.chargeback(Coin::new(11, 1));
        assert_eq!(account.available, Coin::new(0, 4));
        assert_eq!(account.held, -Coin::new(11, 1));
        assert_eq!(account.total, -Coin::new(11, 1));
        assert_eq!(account.locked, true);
    }
}
