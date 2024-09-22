use crate::{
    account::Account,
    transaction::{InputTransaction, Transaction},
};
use csv_async::AsyncReaderBuilder;
use rust_decimal::Decimal;
use std::{ffi::OsString, io};
use tokio::{fs::File, sync::mpsc};
use tokio_stream::StreamExt;

pub const CHANNEL_BUUFER_SIZE: usize = 100;
pub const PRECISION: u32 = 4;
pub type AccountID = u16;
pub type TxID = u32;
pub type Coin = Decimal;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message {
    Tx(Transaction),
    Stop,
}

pub fn write_results(v: Vec<Account>) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for i in v {
        wtr.serialize(i)?;
    }
    wtr.flush()?;
    Ok(())
}

pub async fn run_reader(file_path: OsString, sender: mpsc::Sender<Message>) -> anyhow::Result<()> {
    let file = File::open(file_path).await?;
    let mut rdr = AsyncReaderBuilder::new()
        .flexible(true)
        .create_deserializer(file);

    let mut records = rdr.deserialize::<InputTransaction>();
    while let Some(Ok(record)) = records.next().await {
        if let Ok(tx) = Transaction::try_from(record) {
            sender.send(Message::Tx(tx)).await.expect("service stopped");
        }
        // TODO: handle parsing errors
    }
    sender
        .send(Message::Stop)
        .await
        .expect("failed to gracefully stop");
    Ok(())
}
