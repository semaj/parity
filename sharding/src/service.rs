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

//! Sharding devp2p network service

use std::sync::Arc;

use common;
use client::Client;
use net::{self, NetworkProtocolHandler, NetworkContext};
use ethsync::ManageNetwork;
use message::{Message, MessagePayload};

const SHARDING_PROTOCOL_ID: &net::ProtocolId = b"shr";

pub struct Service {
	client: Arc<Client>,
	net: Arc<ManageNetwork>
}

impl Service {
	pub fn new(client: Arc<Client>, net: Arc<ManageNetwork>) -> Self {
		Service { client: client, net: net }
	}

	pub fn send(&self, msg: Message) {
		let mut action = Some(into_action(msg));
		self.net.with_proto_context(
			*SHARDING_PROTOCOL_ID,
			&mut |ctx: &NetworkContext| {
				if let Some(action) = action.take() {
					match action {
						Action::Disconnect(peer_id) => { let _err = ctx.disconnect_peer(peer_id); },
						Action::Send(peer_id, packet_id, payload) => {
							if let Err(e) = ctx.send(peer_id, packet_id, payload) {
								trace!("Error sending packet to peer: {}", e)
							}
						}
					}
				}
			}
		);
	}
}

impl NetworkProtocolHandler for Service {
	fn read(&self, _ctx: &NetworkContext, peer: &net::PeerId, packet_id: u8, _data: &[u8]) {
		let message = match packet_id {
			packet::STATUS => Message {
				peer_id: *peer as common::PeerId,
				payload: MessagePayload::Status {
					// todo: deserialize
					protocol_version: 1,
					shard_id: 1,
					head_height: 1,
					head_hash: 0x0,
				}
			},
			_ => {
				// todo: drop stupid peer
				return;
			}
		};

		let mut sink = Vec::new();
		self.client.message(message, &mut sink)
			.expect("Vec as sink never fails; qed");

		for back_msg in sink.drain(..) {
			self.send(back_msg);
		}
	}

	fn connected(&self, _ctx: &NetworkContext, _peer: &net::PeerId) {
	}

	fn disconnected(&self, _io: &NetworkContext, peer: &net::PeerId) {
		self.client.remove_peer(*peer as common::PeerId);
	}
}

mod packet {
	pub const STATUS: u8 = 0x00;
}

enum Action {
	Send(net::PeerId, net::PacketId, Vec<u8>),
	Disconnect(net::PeerId),
}

fn into_action(msg: Message) -> Action {
	let peer_id = msg.peer_id;
	let payload = msg.payload;
	match payload {
		MessagePayload::Status { protocol_version: _, shard_id: _, head_height: _, head_hash: _ } =>
			Action::Send(peer_id as usize, packet::STATUS, Vec::new() /* todo: serialize */),

		MessagePayload::Disconnect => Action::Disconnect(peer_id as usize),
	}
}