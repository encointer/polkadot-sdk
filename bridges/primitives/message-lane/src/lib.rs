// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Primitives of message lane module.

#![cfg_attr(not(feature = "std"), no_std)]
// RuntimeApi generated functions
#![allow(clippy::too_many_arguments)]
// Generated by `DecodeLimit::decode_with_depth_limit`
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use sp_api::decl_runtime_apis;
use sp_std::{collections::vec_deque::VecDeque, prelude::*};

pub mod source_chain;
pub mod target_chain;

/// Lane identifier.
pub type LaneId = [u8; 4];

/// Message nonce. Valid messages will never have 0 nonce.
pub type MessageNonce = u64;

/// Message id as a tuple.
pub type MessageId = (LaneId, MessageNonce);

/// Opaque message payload. We only decode this payload when it is dispatched.
pub type MessagePayload = Vec<u8>;

/// Message key (unique message identifier) as it is stored in the storage.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct MessageKey {
	/// ID of the message lane.
	pub lane_id: LaneId,
	/// Message nonce.
	pub nonce: MessageNonce,
}

/// Message data as it is stored in the storage.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct MessageData<Fee> {
	/// Message payload.
	pub payload: MessagePayload,
	/// Message delivery and dispatch fee, paid by the submitter.
	pub fee: Fee,
}

/// Message as it is stored in the storage.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Message<Fee> {
	/// Message key.
	pub key: MessageKey,
	/// Message data.
	pub data: MessageData<Fee>,
}

/// Inbound lane data.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct InboundLaneData<RelayerId> {
	/// Identifiers of relayers and messages that they have delivered (ordered by message nonce).
	/// It is guaranteed to have at most N entries, where N is configured at module level. If
	/// there are N entries in this vec, then:
	/// 1) all incoming messages are rejected if they're missing corresponding `proof-of(outbound-lane.state)`;
	/// 2) all incoming messages are rejected if `proof-of(outbound-lane.state).latest_received_nonce` is
	///    equal to `this.latest_confirmed_nonce`.
	/// Given what is said above, all nonces in this queue are in range (latest_confirmed_nonce; latest_received_nonce].
	///
	/// When a relayer sends a single message, both of MessageNonces are the same.
	/// When relayer sends messages in a batch, the first arg is the lowest nonce, second arg the highest nonce.
	/// Multiple dispatches from the same relayer one are allowed.
	pub relayers: VecDeque<(MessageNonce, MessageNonce, RelayerId)>,
	/// Nonce of latest message that we have received from bridged chain.
	pub latest_received_nonce: MessageNonce,
	/// Nonce of latest message that has been confirmed to the bridged chain.
	pub latest_confirmed_nonce: MessageNonce,
}

impl<RelayerId> Default for InboundLaneData<RelayerId> {
	fn default() -> Self {
		InboundLaneData {
			relayers: VecDeque::new(),
			latest_received_nonce: 0,
			latest_confirmed_nonce: 0,
		}
	}
}

/// Outbound lane data.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct OutboundLaneData {
	/// Nonce of oldest message that we haven't yet pruned. May point to not-yet-generated message if
	/// all sent messages are already pruned.
	pub oldest_unpruned_nonce: MessageNonce,
	/// Nonce of latest message, received by bridged chain.
	pub latest_received_nonce: MessageNonce,
	/// Nonce of latest message, generated by us.
	pub latest_generated_nonce: MessageNonce,
}

impl Default for OutboundLaneData {
	fn default() -> Self {
		OutboundLaneData {
			// it is 1 because we're pruning everything in [oldest_unpruned_nonce; latest_received_nonce]
			oldest_unpruned_nonce: 1,
			latest_received_nonce: 0,
			latest_generated_nonce: 0,
		}
	}
}

decl_runtime_apis! {
	/// Outbound message lane API.
	pub trait OutboundLaneApi {
		/// Returns nonce of the latest message, received by bridged chain.
		fn latest_received_nonce(lane: LaneId) -> MessageNonce;
		/// Returns nonce of the latest message, generated by given lane.
		fn latest_generated_nonce(lane: LaneId) -> MessageNonce;
	}

	/// Inbound message lane API.
	pub trait InboundLaneApi {
		/// Returns nonce of the latest message, received by given lane.
		fn latest_received_nonce(lane: LaneId) -> MessageNonce;
		/// Nonce of latest message that has been confirmed to the bridged chain.
		fn latest_confirmed_nonce(lane: LaneId) -> MessageNonce;
	}
}
