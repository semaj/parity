// TODO: if connected to all configured nodes => consensus unreachable, else => temporary unreachable

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

use std::fmt;
use std::io::Error as IoError;

use serde_json;

use {ethkey, ethcrypto, kvdb, bytes, ethereum_types, key_server_cluster};

/// Secret store error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
	/// Invalid node address has been passed.
	InvalidNodeAddress,
	/// Invalid node id has been passed.
	InvalidNodeId,
	/// Session with the given id already exists.
	DuplicateSessionId,
	/// No active session with given id.
	NoActiveSessionWithId,
	/// Invalid threshold value has been passed.
	/// Threshold value must be in [0; n - 1], where n is a number of nodes participating in the encryption.
	NotEnoughNodesForThreshold,
	/// Current state of encryption/decryption session does not allow to proceed request.
	/// Reschedule this request for later processing.
	TooEarlyForRequest,
	/// Current state of encryption/decryption session does not allow to proceed request.
	/// This means that either there is some comm-failure or node is misbehaving/cheating.
	InvalidStateForRequest,
	/// Request cannot be sent/received from this node.
	InvalidNodeForRequest,
	/// Message or some data in the message was recognized as invalid.
	/// This means that node is misbehaving/cheating.
	InvalidMessage,
	/// Message version is not supported.
	InvalidMessageVersion,
	/// Message is invalid because of replay-attack protection.
	ReplayProtection,
	/// Connection to node, required for this session is not established.
	NodeDisconnected,
	/// Cryptographic error.
	EthKey(String),
	/// I/O error has occured.
	Io(String),
	/// Deserialization error has occured.
	Serde(String),
	/// Consensus is temporary unreachable.
	ConsensusTemporaryUnreachable,
	/// Consensus is unreachable.
	ConsensusUnreachable,
	/// Acl storage error.
	AccessDenied,
	/// Can't start session, because exclusive session is active.
	ExclusiveSessionActive,
	/// Can't start exclusive session, because there are other active sessions.
	HasActiveSessions,

	/// Insufficient requester data
	InsufficientRequesterData(String),
	/// Hyper error
	Hyper(String),
	/// Database-related error
	Database(String),
	/// Internal error
	Internal(String),

	/// Server key with this ID is already generated.
	ServerKeyAlreadyGenerated,
	/// Server key with this ID is not yet generated.
	ServerKeyIsNotFound,
	/// Server key version with this ID is not found.
	ServerKeyVersionIsNotFound,
	/// Document key with this ID is already stored.
	DocumentKeyAlreadyStored,
	/// Document key with this ID is not yet stored.
	DocumentKeyIsNotFound,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::InvalidNodeAddress => write!(f, "invalid node address has been passed"),
			Error::InvalidNodeId => write!(f, "invalid node id has been passed"),
			Error::DuplicateSessionId => write!(f, "session with the same id is already registered"),
			Error::NoActiveSessionWithId => write!(f, "no active session with given id"),
			Error::NotEnoughNodesForThreshold => write!(f, "not enough nodes for passed threshold"),
			Error::TooEarlyForRequest => write!(f, "session is not yet ready to process this request"),
			Error::InvalidStateForRequest => write!(f, "session is in invalid state for processing this request"),
			Error::InvalidNodeForRequest => write!(f, "invalid node for this request"),
			Error::InvalidMessage => write!(f, "invalid message is received"),
			Error::InvalidMessageVersion => write!(f, "unsupported message is received"),
			Error::ReplayProtection => write!(f, "replay message is received"),
			Error::NodeDisconnected => write!(f, "node required for this operation is currently disconnected"),
			Error::EthKey(ref e) => write!(f, "cryptographic error {}", e),
			Error::ConsensusUnreachable => write!(f, "Consensus unreachable"),
			Error::ConsensusTemporaryUnreachable => write!(f, "Consensus temporary unreachable"),
			Error::ExclusiveSessionActive => write!(f, "Exclusive session active"),
			Error::HasActiveSessions => write!(f, "Unable to start exclusive session"),

			Error::InsufficientRequesterData(ref e) => write!(f, "Insufficient requester data: {}", e),
			Error::AccessDenied => write!(f, "Access dened"),
			Error::Hyper(ref msg) => write!(f, "Hyper error: {}", msg),
			Error::Serde(ref msg) => write!(f, "Serialization error: {}", msg),
			Error::Database(ref msg) => write!(f, "Database error: {}", msg),
			Error::Internal(ref msg) => write!(f, "Internal error: {}", msg),
			Error::Io(ref msg) => write!(f, "IO error: {}", msg),

			Error::ServerKeyAlreadyGenerated => write!(f, "Server key with this ID is already generated"),
			Error::ServerKeyIsNotFound => write!(f, "Server key with this ID is not found"),
			Error::ServerKeyVersionIsNotFound => write!(f, "Server key version with this ID is not found"),
			Error::DocumentKeyAlreadyStored => write!(f, "Document key with this ID is already stored"),
			Error::DocumentKeyIsNotFound => write!(f, "Document key with this ID is not found"),
		}
	}
}

impl From<ethkey::Error> for Error {
	fn from(err: ethkey::Error) -> Self {
		Error::EthKey(err.into())
	}
}

impl From<kvdb::Error> for Error {
	fn from(err: kvdb::Error) -> Self {
		Error::Database(err.to_string())
	}
}

impl From<ethcrypto::Error> for Error {
	fn from(err: ethcrypto::Error) -> Self {
		Error::EthKey(err.into())
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err.to_string())
	}
}

/*impl From<key_server_cluster::Error> for Error {
	fn from(err: key_server_cluster::Error) -> Self {
		match err {
			key_server_cluster::Error::InsufficientRequesterData(err)
				=> Error::InsufficientRequesterData(err),
			key_server_cluster::Error::ConsensusUnreachable
				| key_server_cluster::Error::AccessDenied => Error::AccessDenied,
			key_server_cluster::Error::MissingKeyShare => Error::DocumentNotFound,
			_ => Error::Internal(err.into()),
		}
	}
}*/

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}


impl Error {
	/// Is this an internal error? Internal error means that it is SS who's responsible for it, like: connectivity, db failure, ...
	/// External error is caused by SS misuse, like: trying to generate duplicated key, access denied, ...
	/// When internal error occurs, it is possible that the same request will succeed after retry.
	/// When external error occurs, we reject request.
	fn is_internal_error(&self) -> bool {
		match *self {
			Error::InvalidNodeAddress => false,
			Error::InvalidNodeId => false,
			Error::DuplicateSessionId => true,
			Error::NoActiveSessionWithId => true,
			Error::NotEnoughNodesForThreshold => false,
			Error::TooEarlyForRequest => true,
			Error::InvalidStateForRequest => true,
			Error::InvalidNodeForRequest => true,
			Error::InvalidMessage => true,
			Error::InvalidMessageVersion => true,
			Error::ReplayProtection => true,
			Error::NodeDisconnected => true,
			Error::ConsensusUnreachable => false,
			Error::ConsensusTemporaryUnreachable => true,
			Error::AccessDenied => false,
			Error::ExclusiveSessionActive => true,
			Error::HasActiveSessions => true,
			Error::InsufficientRequesterData(_) => false,

			Error::EthKey(_) => false,
			Error::Serde(_) => false,
			Error::Hyper(_) => false,
			Error::Database(_) => false,
			Error::Internal(_) => false,
			Error::Io(_) => false,

			Error::ServerKeyAlreadyGenerated => false,
			Error::ServerKeyIsNotFound => false,
			Error::ServerKeyVersionIsNotFound => true,
			Error::DocumentKeyAlreadyStored => false,
			Error::DocumentKeyIsNotFound => false,
		}

		// TODO [Reliability]: implement me after proper is passed through network
		//false
	}
}