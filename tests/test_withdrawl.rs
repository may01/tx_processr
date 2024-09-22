use krct_async::account::Account;
use krct_async::primitives::*;

mod common;

#[tokio::test]
async fn single_withdrawal() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(-Coin::new(11, 1))
        .set_total(-Coin::new(11, 1));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn succesfull_withdrawal_multi() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        withdrawal,1,2,2.2222  "
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(-Coin::new(33222, 4))
        .set_total(-Coin::new(33222, 4));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn duplicate_withdrawal() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        withdrawal,1,1,1.1  "
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(-Coin::new(11, 1))
        .set_total(-Coin::new(11, 1));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn succesfull_withdrawal_two_acc() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        withdrawal,2,2,2.2  "
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(-Coin::new(11, 1))
        .set_total(-Coin::new(11, 1));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));

    let verify_account = Account::new(2)
        .set_available(-Coin::new(22, 1))
        .set_total(-Coin::new(22, 1));

    assert!(verify_account.check_amounts(accounts.get(&2).unwrap()));
}
