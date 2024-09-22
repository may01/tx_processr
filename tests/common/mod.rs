use krct_async::account::Account;
use krct_async::primitives::*;
use krct_async::service::Service;
use krct_async::transaction::{InputTransaction, Transaction};
use std::collections::HashMap;

pub async fn run_tx(data: String) -> HashMap<AccountID, Account> {
    let (tx_sender, rx) = tokio::sync::mpsc::channel(CHANNEL_BUUFER_SIZE);

    let data_handle: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        //let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());
        for result in rdr.deserialize::<InputTransaction>() {
            if let Ok(record) = result {
                if let Ok(tx) = Transaction::try_from(record) {
                    tx_sender.send(Message::Tx(tx)).await?;
                }
            }
        }
        tx_sender.send(Message::Stop).await?;
        Ok(())
    });

    let service_handle = tokio::spawn(async move {
        let mut service = Service::new(rx);
        service.run().await;
        service.get_accounts().await
    });

    let (_, accounts) = tokio::join!(data_handle, service_handle);

    return accounts.unwrap();
}
