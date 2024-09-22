use krct_async::account::Account;
use krct_async::primitives::*;

mod common;

#[tokio::test]
async fn empty_resolve() {
    let data = "\
        type,client,tx,amount
        resolve,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(0, 0))
        .set_total(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn no_dispute_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        resolve,1,1    
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(11, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn single_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1
        resolve,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(11, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn double_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1
        resolve,1,1
        resolve,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(11, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn double_dispute_with_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1
        resolve,1,1
        dispute,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(0, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(11, 1));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_tx_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        deposit,1,2,2.2222 
        dispute,1,1 
        resolve,1,1"
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(33222, 4))
        .set_total(Coin::new(33222, 4))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_tx_multi_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        dispute,1,1 
        deposit,1,2,2.2222 
        resolve,1,1
        dispute,1,2 
        resolve,1,2
        "
    .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(Coin::new(33222, 4))
        .set_total(Coin::new(33222, 4));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}

#[tokio::test]
async fn multi_acc_resolve() {
    let data = "\
        type,client,tx,amount
        deposit,1,1,1.1
        deposit,2,2,2.2222 
        dispute,1,1 
        resolve,1,1"
        .to_owned();

    let accounts = common::run_tx(data.to_owned()).await;

    // acc 1
    let verify_account = Account::new(1)
        .set_available(Coin::new(11, 1))
        .set_total(Coin::new(11, 1))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));

    // acc 2
    let verify_account = Account::new(2)
        .set_available(Coin::new(22222, 4))
        .set_total(Coin::new(22222, 4))
        .set_held(Coin::new(0, 0));

    assert!(verify_account.check_amounts(accounts.get(&2).unwrap()));
}

#[tokio::test]
async fn single_withdrawl_resolve() {
    let data = "\
        type,client,tx,amount
        withdrawal,1,1,1.1
        dispute,1,1
        resolve,1,1
        ";

    let accounts = common::run_tx(data.to_owned()).await;

    let verify_account = Account::new(1)
        .set_available(-Coin::new(11, 1))
        .set_total(-Coin::new(11, 1));

    assert!(verify_account.check_amounts(accounts.get(&1).unwrap()));
}
