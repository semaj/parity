use ethcore;
use ethcore::client::BlockId;
use ethcore::executive::{Executive, TransactOptions};
use ethereum_types::Address;
use rlp;
use snapshot::SnapshotCommand;
use transaction::SignedTransaction;

pub fn execute(cmd: SnapshotCommand) -> Result<String, String> {
	let service = cmd.restore().unwrap();
	let client = service.client();
	let hex = "f86e8332dac6847735940082c350946c6ba235fb267945ceaf75a22a3a3584e283a35a880166fb336fcca0008025a0355d5fd94c555dab671364f619d8d2a3ef73ebeed665010f62db74e10cc0d355a02e6cb9527af71f11564c3c8cac452fa859e827a0fd7b3be1ceca3572e52f2035";
	let signed = rlp::decode(&::rustc_hex::FromHex::from_hex(hex).unwrap());
	let transaction = SignedTransaction::new(signed).unwrap();
	let mut state = client.latest_state();
	let options = TransactOptions::with_tracing()
		.dont_check_nonce()
		.save_output_from_contract();
	let env_info = client.env_info(BlockId::Latest).unwrap();
	let mut ret = Executive::new(&mut state, &env_info, client.engine().machine())
		.transact_virtual(&transaction, options)
		.unwrap();
	return Ok(String::new());
}
