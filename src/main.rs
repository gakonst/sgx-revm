use alloy_primitives::{Address, U256};
use revm::{
    primitives::{AccountInfo, TxEnv},
    InMemoryDB, EVM,
};

use std::{io::Read, net::TcpListener};

mod server;
use server::{get_key_and_cert, serve};

// This payload should be generalized to include all the pre-state for each
// simulation.
#[derive(serde::Deserialize)]
struct Payload {
    sender: Address,
    amount: U256,
}

fn main() -> eyre::Result<()> {
    let (mut key, mut cert) = get_key_and_cert();
    // dbg!(&cert);
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buf = vec![];
        let _num_bytes = stream.read_to_end(&mut buf)?;
        let data: Payload = serde_json::from_slice(&buf)?;
        simulate(data)?;

        // TODO: Re-enable this,
        // let _ = serve(stream, &mut key, &mut cert).unwrap();
    }

    Ok(())
}

fn simulate(payload: Payload) -> eyre::Result<()> {
    let mut db = InMemoryDB::default();
    let receiver = payload.sender;
    let value = payload.amount;

    let balance = U256::from(111);
    // this is a random address
    let address = "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".parse()?;
    let info = AccountInfo {
        balance,
        ..Default::default()
    };

    // Populate the DB pre-state,
    // TODO: Make this data witnessed via merkle patricia proofs.
    db.insert_account_info(address, info);
    // For storage insertions:
    // db.insert_account_storage(address, slot, value)

    // Setup the EVM with the configured DB
    // The EVM will ONLY be able to access the witnessed state, and
    // any simulation that tries to use state outside of the provided data
    // will fail.
    let mut evm = EVM::new();
    evm.database(db);

    evm.env.tx = TxEnv {
        caller: address,
        transact_to: revm::primitives::TransactTo::Call(receiver),
        value,
        ..Default::default()
    };

    let result = evm.transact_ref()?;

    assert_eq!(
        result.state.get(&address).unwrap().info.balance,
        U256::from(69)
    );

    dbg!(&result);

    Ok(())
}
