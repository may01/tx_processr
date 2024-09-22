use krct_async::account::Account;
use krct_async::primitives::*;

mod common;

#[tokio::test]
async fn empty_chargeback() {
    let data = "\
        type,client,tx,amount
        chargeback,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(0, 0))
        .set_total(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn no_dispute_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        chargeback,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(11, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn single_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1        
        dispute,1,1
        chargeback,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1).set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn double_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1        
        dispute,1,1
        chargeback,1,1
        chargeback,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    // acc 1
    let verify_account = Account::new(1)
        .set_available(Coin::new(0, 0))
        .set_total(Coin::new(0, 0))
        .set_held(Coin::new(0, 0))
        .set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn double_dispute_with_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1
        chargeback,1,1
        dispute,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1).set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_tx_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        deposit,1,2,2.2222 
        dispute,1,1 
        chargeback,1,1"
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(22222, 4))
        .set_total(Coin::new(22222, 4))
        .set_held(Coin::new(0, 0))
        .set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_tx_multi_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1 
        deposit,1,2,2.2222 
        chargeback,1,1
        dispute,1,2 
        chargeback,1,2
        "
    .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(22222, 4))
        .set_total(Coin::new(22222, 4))
        .set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_acc_chargeback() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        deposit,2,2,2.2222 
        dispute,1,1 
        chargeback,1,1"
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    // acc 1
    let verify_account = Account::new(1).set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));

    // acc 2
    let verify_account = Account::new(2)
        .set_available(Coin::new(22222, 4))
        .set_total(Coin::new(22222, 4))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&2).unwrap()));
}

#[tokio::test]
async fn check_chargeback_lock() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1 
        chargeback,1,1
        deposit,1,2,2.2222         
        "
    .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(0, 4))
        .set_total(Coin::new(0, 4))
        .set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn single_withdrawl_chargeback() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        dispute,1,1
        chargeback,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1).set_locked(true);

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}
