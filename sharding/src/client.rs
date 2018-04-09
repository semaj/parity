// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Transport-agnostic sharding client implementation.

use parking_lot::RwLock;
use futures::Sink;

use {common, message, ShardId, PeerId, Head, Message, MessagePayload, Database, disconnect_message};

pub struct Client {
	shard_id: ShardId,
	peers: RwLock<Vec<PeerId>>,
	head: RwLock<Head>,
	db: Box<Database>
}

impl Client {

	pub fn new(shard_id: ShardId, db: Box<Database>) -> Self {
		Client {
			shard_id: shard_id,
			head: RwLock::default(),
			peers: RwLock::default(),
			db: db,
		}
	}

	pub fn message<S, E>(&self, message: Message, sink: &mut S)
		-> Result<(), E>
		where S: Sink<SinkItem=Message, SinkError=E>
	{
		use MessagePayload::*;

		let peer_id = message.peer_id;
		let payload = message.payload;

		match payload {
			Status { protocol_version, shard_id, head_hash: _, head_height: _ } => {
				trace!("Status message from peer #{}: shard_id = {}", peer_id, shard_id);
				if protocol_version != common::PROTOCOL_VERSION {
					trace!("Sending disconnect to peer #{}: Invalid protocol version ({})",
						peer_id, protocol_version);

					sink.start_send(disconnect_message(peer_id))?;
				} else if shard_id != self.shard_id {
					trace!("Sending disconnect to peer #{}: useless shard ({})",
						peer_id, shard_id);

					sink.start_send(disconnect_message(peer_id))?;
				} else {
					self.peers.write().push(peer_id);
					sink.start_send(self.status_message(peer_id))?;
				}
			},
			_ => {
				trace!("Unhandled message type from peer #{}", peer_id)
			}
		}

		Ok(())
	}

	pub fn maintaince<S, E>(&self, _sink: &mut S)
		where S: Sink<SinkItem=Message, SinkError=E>
	{

	}

	fn status_message(&self, peer_id: PeerId) -> Message {
		let head = self.head.read();
		Message {
			peer_id: peer_id,
			payload: MessagePayload::Status {
				protocol_version: common::PROTOCOL_VERSION,
				shard_id: self.shard_id,
				head_hash: *head.hash(),
				head_height: head.height(),
			}
		}
	}

	pub fn active_peers(&self) -> usize {
		self.peers.read().len()
	}

	pub fn remove_peer(&self, peer: PeerId) {
		self.peers.write().retain(|p| *p != peer);
	}
}