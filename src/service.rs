use crate::account::Account;
use crate::primitives::{AccountID, Message, CHANNEL_BUUFER_SIZE};
use crate::transaction::Transaction;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct Service {
    input: mpsc::Receiver<Message>,

    // TODO: can be combined with accounts_channels in separate structure with option chanel
    accounts: HashMap<AccountID, Arc<Mutex<Account>>>,
    accounts_channels: HashMap<AccountID, mpsc::Sender<Message>>,
}

impl Service {
    pub fn new(receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            accounts: HashMap::new(),
            input: receiver,
            accounts_channels: HashMap::new(),
        }
    }

    /// Wait for messages from reader, parse them and process transaction
    pub async fn run(&mut self) {
        while let Some(input) = self.input.recv().await {
            match input {
                Message::Tx(tx) => {
                    let _ = self.process_tx(tx).await; // TODO: handle error
                }
                Message::Stop => {
                    for acc_sender in self.accounts_channels.values() {
                        let _ = acc_sender.send(Message::Stop).await;
                        acc_sender.closed().await;
                    }
                    break;
                }
            }
        }
    }

    /// Process transaction:
    ///
    /// if there are no workers for account, open new connection,
    ///
    /// if there are workers for account, send transaction,
    ///
    /// if connection is closed, remove worker,
    pub async fn process_tx(&mut self, tx: Transaction) -> anyhow::Result<()> {
        let acc_id = tx.account();
        //if there are task for account
        if let Some(acc_sender) = self.accounts_channels.get_mut(&acc_id) {
            // send transaction to the task
            if acc_sender.send(Message::Tx(tx.clone())).await.is_err() {
                self.accounts_channels.remove(&acc_id);
                // TODO: make proper connection reopen
            }
        } else {
            // create new task for account

            // get or create account 'acc_id'
            let account = self
                .accounts
                .entry(acc_id)
                .or_insert(Arc::new(Mutex::new(Account::new(acc_id))));
            let account = Arc::clone(account);

            // open new channel
            let (acc_sender, mut acc_receiver) = mpsc::channel(CHANNEL_BUUFER_SIZE);
            acc_sender.send(Message::Tx(tx)).await?;
            self.accounts_channels.insert(acc_id, acc_sender);

            // spawn new task for account processing
            tokio::spawn(async move {
                while let Some(tx_msg) = acc_receiver.recv().await {
                    let mut lock = account.lock().await;
                    match tx_msg {
                        Message::Tx(tx) => {
                            lock.process(&tx).await;
                        }
                        Message::Stop => {
                            break;
                        }
                    }
                    // TODO: close connection by timeout: not relevant in 'read file' case
                }
            });
        }
        Ok(())
    }

    /// Get all accounts  
    ///
    /// clones all accounts states and returns them without mutexes
    pub async fn get_accounts(&mut self) -> HashMap<AccountID, Account> {
        let mut res = HashMap::with_capacity(self.accounts.len());
        for (&id, m) in &self.accounts {
            let val = m.lock().await;
            res.insert(id, val.clone());
        }
        res
    }
}
