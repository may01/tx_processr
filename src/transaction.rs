use crate::primitives::{AccountID, Coin, TxID, PRECISION};
use anyhow::{anyhow, Error as AnyhowError, Result as AnyhowResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputTransaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub client: String,
    #[serde(rename = "tx")]
    pub id: String,
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<String> for TransactionType {
    type Error = AnyhowError;

    fn try_from(input: String) -> AnyhowResult<Self> {
        match input.to_lowercase().trim() {
            "deposit" => Ok(TransactionType::Deposit),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "chargeback" => Ok(TransactionType::Chargeback),
            _ => Err(anyhow!("Unknown TransactionType: {}", input)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Transaction {
    tx_type: TransactionType,
    account: AccountID,
    id: TxID,
    amount: Option<Coin>,
}

impl TryFrom<InputTransaction> for Transaction {
    type Error = AnyhowError;

    fn try_from(input: InputTransaction) -> AnyhowResult<Self> {
        let tx_type = input.tx_type.try_into()?;
        let amount = match tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                let val: Coin = input
                    .amount
                    .ok_or(anyhow!("Wrong amount"))?
                    .trim()
                    .parse()?;
                if val < Coin::new(0, 0) {
                    return Err(anyhow!("Negative amount"));
                }
                Some(val.round_dp(PRECISION))
            }
            _ => None,
        };

        Ok(Self {
            tx_type,
            account: input.client.trim().parse()?,
            id: input.id.trim().parse()?,
            amount,
        })
    }
}

#[derive(PartialEq)]
pub enum AncestorState {
    Valid,   // transaction can be parent for current
    Invalid, // transaction doesn't allow current transaction to occur
}

impl Transaction {
    pub fn account(&self) -> AccountID {
        self.account
    }

    pub fn amount(&self) -> Coin {
        self.amount.unwrap_or_default()
    }

    pub fn tx_type(&self) -> TransactionType {
        self.tx_type
    }

    pub fn id(&self) -> TxID {
        self.id
    }

    pub fn valid_ancestor(&self, ancestor: &Self) -> AncestorState {
        if self.account != ancestor.account {
            return AncestorState::Invalid;
        }
        if self.id != ancestor.id {
            return AncestorState::Invalid;
        }

        match self.tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                AncestorState::Invalid // we don't allow deposit and withdrawal if we have tx with same id + client
            }
            TransactionType::Dispute => {
                match ancestor.tx_type {
                    TransactionType::Deposit | TransactionType::Withdrawal => AncestorState::Valid, // we allow dispute for new tx
                    TransactionType::Dispute => AncestorState::Invalid, // we don't allow second dispute after dispute
                    TransactionType::Resolve | TransactionType::Chargeback => AncestorState::Valid, // we allow second dispute for finalized dispute
                }
            }
            TransactionType::Resolve | TransactionType::Chargeback => {
                match ancestor.tx_type {
                    TransactionType::Deposit | TransactionType::Withdrawal => {
                        AncestorState::Invalid
                    } // we don't allow finalized dispute without dispute
                    TransactionType::Dispute => AncestorState::Valid, // we allow to finalize dispute
                    TransactionType::Resolve | TransactionType::Chargeback => {
                        AncestorState::Invalid
                    } // we don't allow second dispute finalization
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_into_transaction_deposit() {
        let input = InputTransaction {
            tx_type: "".to_owned(),
            client: "".to_owned(),
            id: "".to_owned(),
            amount: Some("".to_owned()),
        };

        assert!(Transaction::try_from(input).is_err());
    }

    #[test]
    fn test_into_transaction_dispute() {
        let input = InputTransaction {
            tx_type: "dispute".to_owned(),
            client: "1".to_owned(),
            id: "2".to_owned(),
            amount: Some("3.0".to_owned()),
        };

        let output = Transaction {
            tx_type: TransactionType::Dispute,
            account: 1,
            id: 2,
            amount: None,
        };

        assert_eq!(Transaction::try_from(input).unwrap(), output);
    }

    #[test]
    fn test_into_transaction_deposits() {
        let input = InputTransaction {
            tx_type: "deposit".to_owned(),
            client: "1".to_owned(),
            id: "2".to_owned(),
            amount: Some("3.0".to_owned()),
        };

        let output = Transaction {
            tx_type: TransactionType::Deposit,
            account: 1,
            id: 2,
            amount: Some(Coin::new(3, 0)),
        };

        assert_eq!(Transaction::try_from(input).unwrap(), output);
    }

    #[test]
    fn test_into_transaction_withdrawal_with_spaces() {
        let input = InputTransaction {
            tx_type: " withdrawal".to_owned(),
            client: "1 ".to_owned(),
            id: "2    ".to_owned(),
            amount: Some("    3.0".to_owned()),
        };

        let output = Transaction {
            tx_type: TransactionType::Withdrawal,
            account: 1,
            id: 2,
            amount: Some(Coin::new(3, 0)),
        };

        assert_eq!(Transaction::try_from(input).unwrap(), output);
    }

    #[test]
    fn test_into_transaction_deposits_no_amount() {
        let input = InputTransaction {
            tx_type: " deposit".to_owned(),
            client: "1 ".to_owned(),
            id: "2    ".to_owned(),
            amount: Some("    ".to_owned()),
        };

        assert!(Transaction::try_from(input).is_err());
    }

    #[test]
    fn test_into_transaction_deposits_negative_amount() {
        let input = InputTransaction {
            tx_type: " deposit".to_owned(),
            client: "1 ".to_owned(),
            id: "2    ".to_owned(),
            amount: Some("-2.3".to_owned()),
        };

        assert!(Transaction::try_from(input).is_err());
    }

    #[test]
    fn test_for_valid_ancestor() {
        // TODO: add valid_ancestor tests
    }
}
