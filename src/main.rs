mod account;
mod primitives;
mod service;
mod transaction;

use primitives::run_reader;
use std::{env, ffi::OsString};
use tokio::sync::mpsc;

use crate::primitives::{write_results, CHANNEL_BUUFER_SIZE};
use crate::service::Service;

fn get_first_arg() -> anyhow::Result<OsString> {
    match env::args_os().nth(1) {
        None => Err(anyhow::anyhow!("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::channel(CHANNEL_BUUFER_SIZE);

    let file_path = get_first_arg()?;
    let data_handle = tokio::spawn(run_reader(file_path, sender));

    let service_handle = tokio::spawn(async {
        let mut service = Service::new(receiver);
        service.run().await;
        service.get_accounts().await
    });

    let (read_res, accounts) = tokio::join!(data_handle, service_handle);

    read_res??;
    let accounts = accounts?;

    write_results(accounts.values().cloned().collect::<Vec<_>>())?;

    Ok(())
}
