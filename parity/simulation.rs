use ethcore::client::BlockId;
use ethcore::executive::{Executive, TransactOptions};
use rlp;
use snapshot::SnapshotCommand;
use std::io::{self, BufRead};
use std::sync::Arc;
use transaction::SignedTransaction;

pub fn execute(cmd: SnapshotCommand) -> Result<String, String> {
	let service = cmd.restore().unwrap();
	let client = service.client();
	let mut state = client.latest_state();
	let env_info = client.env_info(BlockId::Latest).unwrap();
	let stdin = io::stdin();
	for line in stdin.lock().lines() {
		let hex = line.unwrap();
		let signed = rlp::decode(&::rustc_hex::FromHex::from_hex(&hex[..]).unwrap());
		let transaction = SignedTransaction::new(signed).unwrap();
		let options = TransactOptions::with_tracing()
			.dont_check_nonce()
			.save_output_from_contract();
		let mut executive = Executive::new(&mut state, &env_info, client.engine().machine());
		executive.transact_virtual(&transaction, options).unwrap();
	}
	return Ok(String::new());
}
