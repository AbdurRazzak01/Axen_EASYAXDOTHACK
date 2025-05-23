// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

/// Notifications protocol.
///
/// The Substrate notifications protocol consists in the following:
///
/// - Node A opens a substream to node B and sends a message which contains some
///   protocol-specific higher-level logic. This message is prefixed with a variable-length
///   integer message length. This message can be empty, in which case `0` is sent.
/// - If node B accepts the substream, it sends back a message with the same properties.
/// - If instead B refuses the connection (which typically happens because no empty slot is
///   available), then it immediately closes the substream without sending back anything.
/// - Node A can then send notifications to B, prefixed with a variable-length integer
///   indicating the length of the message.
/// - Either node A or node B can signal that it doesn't want this notifications substream
///   anymore by closing its writing side. The other party should respond by also closing their
///   own writing side soon after.
///
/// Notification substreams are unidirectional. If A opens a substream with B, then B is
/// encouraged but not required to open a substream to A as well.
use crate::types::ProtocolName;

use asynchronous_codec::Framed;
use bytes::BytesMut;
use futures::prelude::*;
use libp2p::{
	core::{InboundUpgrade, OutboundUpgrade, UpgradeInfo},
	PeerId,
};
use log::{debug, error, warn};
use unsigned_varint::codec::UviBytes;

use std::{
	fmt, io, mem,
	pin::Pin,
	task::{Context, Poll},
	vec,
};

/// Logging target for the file.
const LOG_TARGET: &str = "sub-libp2p::notification::upgrade";

/// Maximum allowed size of the two handshake messages, in bytes.
const MAX_HANDSHAKE_SIZE: usize = 1024;

/// Upgrade that accepts a substream, sends back a status message, then becomes a unidirectional
/// stream of messages.
#[derive(Debug, Clone)]
pub struct NotificationsIn {
	/// Protocol name to use when negotiating the substream.
	/// The first one is the main name, while the other ones are fall backs.
	protocol_names: Vec<ProtocolName>,
	/// Maximum allowed size for a single notification.
	max_notification_size: u64,
}

/// Upgrade that opens a substream, waits for the remote to accept by sending back a status
/// message, then becomes a unidirectional sink of data.
#[derive(Debug, Clone)]
pub struct NotificationsOut {
	/// Protocol name to use when negotiating the substream.
	/// The first one is the main name, while the other ones are fall backs.
	protocol_names: Vec<ProtocolName>,
	/// Message to send when we start the handshake.
	initial_message: Vec<u8>,
	/// Maximum allowed size for a single notification.
	max_notification_size: u64,
	/// The peerID of the remote.
	peer_id: PeerId,
}

/// A substream for incoming notification messages.
///
/// When creating, this struct starts in a state in which we must first send back a handshake
/// message to the remote. No message will come before this has been done.
#[pin_project::pin_project]
pub struct NotificationsInSubstream<TSubstream> {
	#[pin]
	socket: Framed<TSubstream, UviBytes<io::Cursor<Vec<u8>>>>,
	handshake: NotificationsInSubstreamHandshake,
}

/// State of the handshake sending back process.
#[derive(Debug)]
pub enum NotificationsInSubstreamHandshake {
	/// Waiting for the user to give us the handshake message.
	NotSent,
	/// User gave us the handshake message. Trying to push it in the socket.
	PendingSend(Vec<u8>),
	/// Handshake message was pushed in the socket. Still need to flush.
	Flush,
	/// Handshake message successfully sent and flushed.
	Sent,
	/// Remote has closed their writing side. We close our own writing side in return.
	ClosingInResponseToRemote,
	/// Both our side and the remote have closed their writing side.
	BothSidesClosed,
}

/// A substream for outgoing notification messages.
#[pin_project::pin_project]
pub struct NotificationsOutSubstream<TSubstream> {
	/// Substream where to send messages.
	#[pin]
	socket: Framed<TSubstream, UviBytes<io::Cursor<Vec<u8>>>>,

	/// The remote peer.
	peer_id: PeerId,
}

#[cfg(test)]
impl<TSubstream> NotificationsOutSubstream<TSubstream> {
	pub fn new(socket: Framed<TSubstream, UviBytes<io::Cursor<Vec<u8>>>>) -> Self {
		Self { socket, peer_id: PeerId::random() }
	}
}

impl NotificationsIn {
	/// Builds a new potential upgrade.
	pub fn new(
		main_protocol_name: impl Into<ProtocolName>,
		fallback_names: Vec<ProtocolName>,
		max_notification_size: u64,
	) -> Self {
		let mut protocol_names = fallback_names;
		protocol_names.insert(0, main_protocol_name.into());

		Self { protocol_names, max_notification_size }
	}
}

impl UpgradeInfo for NotificationsIn {
	type Info = ProtocolName;
	type InfoIter = vec::IntoIter<Self::Info>;

	fn protocol_info(&self) -> Self::InfoIter {
		self.protocol_names.clone().into_iter()
	}
}

impl<TSubstream> InboundUpgrade<TSubstream> for NotificationsIn
where
	TSubstream: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
	type Output = NotificationsInOpen<TSubstream>;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;
	type Error = NotificationsHandshakeError;

	fn upgrade_inbound(self, mut socket: TSubstream, _negotiated_name: Self::Info) -> Self::Future {
		Box::pin(async move {
			let handshake_len = unsigned_varint::aio::read_usize(&mut socket).await?;
			if handshake_len > MAX_HANDSHAKE_SIZE {
				return Err(NotificationsHandshakeError::TooLarge {
					requested: handshake_len,
					max: MAX_HANDSHAKE_SIZE,
				})
			}

			let mut handshake = vec![0u8; handshake_len];
			if !handshake.is_empty() {
				socket.read_exact(&mut handshake).await?;
			}

			let mut codec = UviBytes::default();
			codec.set_max_len(usize::try_from(self.max_notification_size).unwrap_or(usize::MAX));

			let substream = NotificationsInSubstream {
				socket: Framed::new(socket, codec),
				handshake: NotificationsInSubstreamHandshake::NotSent,
			};

			Ok(NotificationsInOpen { handshake, substream })
		})
	}
}

/// Yielded by the [`NotificationsIn`] after a successfully upgrade.
pub struct NotificationsInOpen<TSubstream> {
	/// Handshake sent by the remote.
	pub handshake: Vec<u8>,
	/// Implementation of `Stream` that allows receives messages from the substream.
	pub substream: NotificationsInSubstream<TSubstream>,
}

impl<TSubstream> fmt::Debug for NotificationsInOpen<TSubstream> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NotificationsInOpen")
			.field("handshake", &self.handshake)
			.finish_non_exhaustive()
	}
}

impl<TSubstream> NotificationsInSubstream<TSubstream>
where
	TSubstream: AsyncRead + AsyncWrite + Unpin,
{
	#[cfg(test)]
	pub fn new(
		socket: Framed<TSubstream, UviBytes<io::Cursor<Vec<u8>>>>,
		handshake: NotificationsInSubstreamHandshake,
	) -> Self {
		Self { socket, handshake }
	}

	/// Sends the handshake in order to inform the remote that we accept the substream.
	pub fn send_handshake(&mut self, message: impl Into<Vec<u8>>) {
		if !matches!(self.handshake, NotificationsInSubstreamHandshake::NotSent) {
			error!(target: LOG_TARGET, "Tried to send handshake twice");
			return
		}

		self.handshake = NotificationsInSubstreamHandshake::PendingSend(message.into());
	}

	/// Equivalent to `Stream::poll_next`, except that it only drives the handshake and is
	/// guaranteed to not generate any notification.
	pub fn poll_process(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
		let mut this = self.project();

		loop {
			match mem::replace(this.handshake, NotificationsInSubstreamHandshake::Sent) {
				NotificationsInSubstreamHandshake::PendingSend(msg) => {
					match Sink::poll_ready(this.socket.as_mut(), cx) {
						Poll::Ready(_) => {
							*this.handshake = NotificationsInSubstreamHandshake::Flush;
							match Sink::start_send(this.socket.as_mut(), io::Cursor::new(msg)) {
								Ok(()) => {},
								Err(err) => return Poll::Ready(Err(err)),
							}
						},
						Poll::Pending => {
							*this.handshake = NotificationsInSubstreamHandshake::PendingSend(msg);
							return Poll::Pending
						},
					}
				},
				NotificationsInSubstreamHandshake::Flush => {
					match Sink::poll_flush(this.socket.as_mut(), cx)? {
						Poll::Ready(()) => {
							*this.handshake = NotificationsInSubstreamHandshake::Sent;
							return Poll::Ready(Ok(()));
						},
						Poll::Pending => {
							*this.handshake = NotificationsInSubstreamHandshake::Flush;
							return Poll::Pending
						},
					}
				},

				st @ NotificationsInSubstreamHandshake::NotSent |
				st @ NotificationsInSubstreamHandshake::Sent |
				st @ NotificationsInSubstreamHandshake::ClosingInResponseToRemote |
				st @ NotificationsInSubstreamHandshake::BothSidesClosed => {
					*this.handshake = st;
					return Poll::Ready(Ok(()));
				},
			}
		}
	}
}

impl<TSubstream> Stream for NotificationsInSubstream<TSubstream>
where
	TSubstream: AsyncRead + AsyncWrite + Unpin,
{
	type Item = Result<BytesMut, io::Error>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
		let mut this = self.project();

		// This `Stream` implementation first tries to send back the handshake if necessary.
		loop {
			match mem::replace(this.handshake, NotificationsInSubstreamHandshake::Sent) {
				NotificationsInSubstreamHandshake::NotSent => {
					*this.handshake = NotificationsInSubstreamHandshake::NotSent;
					return Poll::Pending
				},
				NotificationsInSubstreamHandshake::PendingSend(msg) => {
					match Sink::poll_ready(this.socket.as_mut(), cx) {
						Poll::Ready(_) => {
							*this.handshake = NotificationsInSubstreamHandshake::Flush;
							match Sink::start_send(this.socket.as_mut(), io::Cursor::new(msg)) {
								Ok(()) => {},
								Err(err) => return Poll::Ready(Some(Err(err))),
							}
						},
						Poll::Pending => {
							*this.handshake = NotificationsInSubstreamHandshake::PendingSend(msg);
							return Poll::Pending
						},
					}
				},
				NotificationsInSubstreamHandshake::Flush => {
					match Sink::poll_flush(this.socket.as_mut(), cx)? {
						Poll::Ready(()) =>
							*this.handshake = NotificationsInSubstreamHandshake::Sent,
						Poll::Pending => {
							*this.handshake = NotificationsInSubstreamHandshake::Flush;
							return Poll::Pending
						},
					}
				},

				NotificationsInSubstreamHandshake::Sent => {
					match Stream::poll_next(this.socket.as_mut(), cx) {
						Poll::Ready(None) =>
							*this.handshake =
								NotificationsInSubstreamHandshake::ClosingInResponseToRemote,
						Poll::Ready(Some(msg)) => {
							*this.handshake = NotificationsInSubstreamHandshake::Sent;
							return Poll::Ready(Some(msg))
						},
						Poll::Pending => {
							*this.handshake = NotificationsInSubstreamHandshake::Sent;
							return Poll::Pending
						},
					}
				},

				NotificationsInSubstreamHandshake::ClosingInResponseToRemote =>
					match Sink::poll_close(this.socket.as_mut(), cx)? {
						Poll::Ready(()) =>
							*this.handshake = NotificationsInSubstreamHandshake::BothSidesClosed,
						Poll::Pending => {
							*this.handshake =
								NotificationsInSubstreamHandshake::ClosingInResponseToRemote;
							return Poll::Pending
						},
					},

				NotificationsInSubstreamHandshake::BothSidesClosed => return Poll::Ready(None),
			}
		}
	}
}

impl NotificationsOut {
	/// Builds a new potential upgrade.
	pub fn new(
		main_protocol_name: impl Into<ProtocolName>,
		fallback_names: Vec<ProtocolName>,
		initial_message: impl Into<Vec<u8>>,
		max_notification_size: u64,
		peer_id: PeerId,
	) -> Self {
		let initial_message = initial_message.into();
		if initial_message.len() > MAX_HANDSHAKE_SIZE {
			error!(target: LOG_TARGET, "Outbound networking handshake is above allowed protocol limit");
		}

		let mut protocol_names = fallback_names;
		protocol_names.insert(0, main_protocol_name.into());

		Self { protocol_names, initial_message, max_notification_size, peer_id }
	}
}

impl UpgradeInfo for NotificationsOut {
	type Info = ProtocolName;
	type InfoIter = vec::IntoIter<Self::Info>;

	fn protocol_info(&self) -> Self::InfoIter {
		self.protocol_names.clone().into_iter()
	}
}

impl<TSubstream> OutboundUpgrade<TSubstream> for NotificationsOut
where
	TSubstream: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
	type Output = NotificationsOutOpen<TSubstream>;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;
	type Error = NotificationsHandshakeError;

	fn upgrade_outbound(self, mut socket: TSubstream, negotiated_name: Self::Info) -> Self::Future {
		Box::pin(async move {
			{
				let mut len_data = unsigned_varint::encode::usize_buffer();
				let encoded_len =
					unsigned_varint::encode::usize(self.initial_message.len(), &mut len_data).len();
				socket.write_all(&len_data[..encoded_len]).await?;
			}
			socket.write_all(&self.initial_message).await?;
			socket.flush().await?;

			// Reading handshake.
			let handshake_len = unsigned_varint::aio::read_usize(&mut socket).await?;
			if handshake_len > MAX_HANDSHAKE_SIZE {
				return Err(NotificationsHandshakeError::TooLarge {
					requested: handshake_len,
					max: MAX_HANDSHAKE_SIZE,
				})
			}

			let mut handshake = vec![0u8; handshake_len];
			if !handshake.is_empty() {
				socket.read_exact(&mut handshake).await?;
			}

			let mut codec = UviBytes::default();
			codec.set_max_len(usize::try_from(self.max_notification_size).unwrap_or(usize::MAX));

			Ok(NotificationsOutOpen {
				handshake,
				negotiated_fallback: if negotiated_name == self.protocol_names[0] {
					None
				} else {
					Some(negotiated_name)
				},
				substream: NotificationsOutSubstream {
					socket: Framed::new(socket, codec),
					peer_id: self.peer_id,
				},
			})
		})
	}
}

/// Yielded by the [`NotificationsOut`] after a successfully upgrade.
pub struct NotificationsOutOpen<TSubstream> {
	/// Handshake returned by the remote.
	pub handshake: Vec<u8>,
	/// If the negotiated name is not the "main" protocol name but a fallback, contains the
	/// name of the negotiated fallback.
	pub negotiated_fallback: Option<ProtocolName>,
	/// Implementation of `Sink` that allows sending messages on the substream.
	pub substream: NotificationsOutSubstream<TSubstream>,
}

impl<TSubstream> fmt::Debug for NotificationsOutOpen<TSubstream> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NotificationsOutOpen")
			.field("handshake", &self.handshake)
			.field("negotiated_fallback", &self.negotiated_fallback)
			.finish_non_exhaustive()
	}
}

impl<TSubstream> Sink<Vec<u8>> for NotificationsOutSubstream<TSubstream>
where
	TSubstream: AsyncRead + AsyncWrite + Unpin,
{
	type Error = NotificationsOutError;

	fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
		let mut this = self.project();
		Sink::poll_ready(this.socket.as_mut(), cx).map_err(NotificationsOutError::Io)
	}

	fn start_send(self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
		let mut this = self.project();
		Sink::start_send(this.socket.as_mut(), io::Cursor::new(item))
			.map_err(NotificationsOutError::Io)
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
		let mut this = self.project();

		// `Sink::poll_flush` does not expose stream closed error until we write something into
		// the stream, so the code below makes sure we detect that the substream was closed
		// even if we don't write anything into it.
		match Stream::poll_next(this.socket.as_mut(), cx) {
			Poll::Pending => {},
			Poll::Ready(Some(result)) => match result {
				Ok(_) => {
					debug!(
						target: "sub-libp2p",
						"Unexpected incoming data in `NotificationsOutSubstream` peer={:?}",
						this.peer_id
					);

					return Poll::Ready(Err(NotificationsOutError::UnexpectedData));
				},
				Err(error) => {
					debug!(
						target: "sub-libp2p",
						"Error while reading from `NotificationsOutSubstream` peer={:?} error={error:?}",
						this.peer_id
					);

					// The expectation is that the remote has closed the substream.
					return Poll::Ready(Err(NotificationsOutError::Closed));
				},
			},
			Poll::Ready(None) => return Poll::Ready(Err(NotificationsOutError::Closed)),
		}

		Sink::poll_flush(this.socket.as_mut(), cx).map_err(NotificationsOutError::Io)
	}

	fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
		let mut this = self.project();
		Sink::poll_close(this.socket.as_mut(), cx).map_err(NotificationsOutError::Io)
	}
}

/// Error generated by sending on a notifications out substream.
#[derive(Debug, thiserror::Error)]
pub enum NotificationsHandshakeError {
	/// I/O error on the substream.
	#[error(transparent)]
	Io(#[from] io::Error),

	/// Initial message or handshake was too large.
	#[error("Initial message or handshake was too large: {requested}")]
	TooLarge {
		/// Size requested by the remote.
		requested: usize,
		/// Maximum allowed,
		max: usize,
	},

	/// Error while decoding the variable-length integer.
	#[error(transparent)]
	VarintDecode(#[from] unsigned_varint::decode::Error),
}

impl From<unsigned_varint::io::ReadError> for NotificationsHandshakeError {
	fn from(err: unsigned_varint::io::ReadError) -> Self {
		match err {
			unsigned_varint::io::ReadError::Io(err) => Self::Io(err),
			unsigned_varint::io::ReadError::Decode(err) => Self::VarintDecode(err),
			_ => {
				warn!("Unrecognized varint decoding error");
				Self::Io(From::from(io::ErrorKind::InvalidData))
			},
		}
	}
}

/// Error generated by sending on a notifications out substream.
#[derive(Debug, thiserror::Error)]
pub enum NotificationsOutError {
	/// I/O error on the substream.
	#[error(transparent)]
	Io(#[from] io::Error),

	/// The substream was closed.
	#[error("substream was closed/reset")]
	Closed,

	/// The remote peer did not comply with the notification spec.
	///
	/// This is a terminal error and the peer should be banned immediately.
	#[error("unexpected data received from the remote peer")]
	UnexpectedData,
}

#[cfg(test)]
mod tests {
	use crate::ProtocolName;

	use super::{
		NotificationsHandshakeError, NotificationsIn, NotificationsInOpen,
		NotificationsInSubstream, NotificationsOut, NotificationsOutError, NotificationsOutOpen,
		NotificationsOutSubstream,
	};
	use futures::{channel::oneshot, future, prelude::*, SinkExt, StreamExt};
	use libp2p::{
		core::{upgrade, InboundUpgrade, OutboundUpgrade, UpgradeInfo},
		PeerId,
	};
	use std::{pin::Pin, task::Poll};
	use tokio::net::{TcpListener, TcpStream};
	use tokio_util::compat::TokioAsyncReadCompatExt;

	/// Opens a substream to the given address, negotiates the protocol, and returns the substream
	/// along with the handshake message.
	async fn dial(
		addr: std::net::SocketAddr,
		handshake: impl Into<Vec<u8>>,
	) -> Result<
		(
			Vec<u8>,
			NotificationsOutSubstream<
				multistream_select::Negotiated<tokio_util::compat::Compat<TcpStream>>,
			>,
		),
		NotificationsHandshakeError,
	> {
		let socket = TcpStream::connect(addr).await.unwrap();
		let notifs_out = NotificationsOut::new(
			"/test/proto/1",
			Vec::new(),
			handshake,
			1024 * 1024,
			PeerId::random(),
		);
		let (_, substream) = multistream_select::dialer_select_proto(
			socket.compat(),
			notifs_out.protocol_info(),
			upgrade::Version::V1,
		)
		.await
		.unwrap();
		let NotificationsOutOpen { handshake, substream, .. } =
			<NotificationsOut as OutboundUpgrade<_>>::upgrade_outbound(
				notifs_out,
				substream,
				"/test/proto/1".into(),
			)
			.await?;
		Ok((handshake, substream))
	}

	/// Listens on a localhost, negotiates the protocol, and returns the substream along with the
	/// handshake message.
	///
	/// Also sends the listener address through the given channel.
	async fn listen_on_localhost(
		listener_addr_tx: oneshot::Sender<std::net::SocketAddr>,
	) -> Result<
		(
			Vec<u8>,
			NotificationsInSubstream<
				multistream_select::Negotiated<tokio_util::compat::Compat<TcpStream>>,
			>,
		),
		NotificationsHandshakeError,
	> {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		listener_addr_tx.send(listener.local_addr().unwrap()).unwrap();

		let (socket, _) = listener.accept().await.unwrap();
		let notifs_in = NotificationsIn::new("/test/proto/1", Vec::new(), 1024 * 1024);
		let (_, substream) =
			multistream_select::listener_select_proto(socket.compat(), notifs_in.protocol_info())
				.await
				.unwrap();
		let NotificationsInOpen { handshake, substream, .. } =
			<NotificationsIn as InboundUpgrade<_>>::upgrade_inbound(
				notifs_in,
				substream,
				"/test/proto/1".into(),
			)
			.await?;
		Ok((handshake, substream))
	}

	#[tokio::test]
	async fn basic_works() {
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let (handshake, mut substream) =
				dial(listener_addr_rx.await.unwrap(), &b"initial message"[..]).await.unwrap();

			assert_eq!(handshake, b"hello world");
			substream.send(b"test message".to_vec()).await.unwrap();
		});

		let (handshake, mut substream) = listen_on_localhost(listener_addr_tx).await.unwrap();

		assert_eq!(handshake, b"initial message");
		substream.send_handshake(&b"hello world"[..]);

		let msg = substream.next().await.unwrap().unwrap();
		assert_eq!(msg.as_ref(), b"test message");

		client.await.unwrap();
	}

	#[tokio::test]
	async fn empty_handshake() {
		// Check that everything still works when the handshake messages are empty.

		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let (handshake, mut substream) =
				dial(listener_addr_rx.await.unwrap(), vec![]).await.unwrap();

			assert!(handshake.is_empty());
			substream.send(Default::default()).await.unwrap();
		});

		let (handshake, mut substream) = listen_on_localhost(listener_addr_tx).await.unwrap();

		assert!(handshake.is_empty());
		substream.send_handshake(vec![]);

		let msg = substream.next().await.unwrap().unwrap();
		assert!(msg.as_ref().is_empty());

		client.await.unwrap();
	}

	#[tokio::test]
	async fn refused() {
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let outcome = dial(listener_addr_rx.await.unwrap(), &b"hello"[..]).await;

			// Despite the protocol negotiation being successfully conducted on the listener
			// side, we have to receive an error here because the listener didn't send the
			// handshake.
			assert!(outcome.is_err());
		});

		let (handshake, substream) = listen_on_localhost(listener_addr_tx).await.unwrap();
		assert_eq!(handshake, b"hello");

		// We successfully upgrade to the protocol, but then close the substream.
		drop(substream);

		client.await.unwrap();
	}

	#[tokio::test]
	async fn large_initial_message_refused() {
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let ret =
				dial(listener_addr_rx.await.unwrap(), (0..32768).map(|_| 0).collect::<Vec<_>>())
					.await;
			assert!(ret.is_err());
		});

		let _ret = listen_on_localhost(listener_addr_tx).await;
		client.await.unwrap();
	}

	#[tokio::test]
	async fn large_handshake_refused() {
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let ret = dial(listener_addr_rx.await.unwrap(), &b"initial message"[..]).await;
			assert!(ret.is_err());
		});

		let (handshake, mut substream) = listen_on_localhost(listener_addr_tx).await.unwrap();
		assert_eq!(handshake, b"initial message");

		// We check that a handshake that is too large gets refused.
		substream.send_handshake((0..32768).map(|_| 0).collect::<Vec<_>>());
		let _ = substream.next().await;

		client.await.unwrap();
	}

	#[tokio::test]
	async fn send_handshake_without_polling_for_incoming_data() {
		const PROTO_NAME: &str = "/test/proto/1";
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let socket = TcpStream::connect(listener_addr_rx.await.unwrap()).await.unwrap();
			let NotificationsOutOpen { handshake, .. } = OutboundUpgrade::upgrade_outbound(
				NotificationsOut::new(
					PROTO_NAME,
					Vec::new(),
					&b"initial message"[..],
					1024 * 1024,
					PeerId::random(),
				),
				socket.compat(),
				ProtocolName::Static(PROTO_NAME),
			)
			.await
			.unwrap();

			assert_eq!(handshake, b"hello world");
		});

		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		listener_addr_tx.send(listener.local_addr().unwrap()).unwrap();

		let (socket, _) = listener.accept().await.unwrap();
		let NotificationsInOpen { handshake, mut substream, .. } = InboundUpgrade::upgrade_inbound(
			NotificationsIn::new(PROTO_NAME, Vec::new(), 1024 * 1024),
			socket.compat(),
			ProtocolName::Static(PROTO_NAME),
		)
		.await
		.unwrap();

		assert_eq!(handshake, b"initial message");
		substream.send_handshake(&b"hello world"[..]);

		// Actually send the handshake.
		future::poll_fn(|cx| Pin::new(&mut substream).poll_process(cx)).await.unwrap();

		client.await.unwrap();
	}

	#[tokio::test]
	async fn can_detect_dropped_out_substream_without_writing_data() {
		const PROTO_NAME: &str = "/test/proto/1";
		let (listener_addr_tx, listener_addr_rx) = oneshot::channel();

		let client = tokio::spawn(async move {
			let socket = TcpStream::connect(listener_addr_rx.await.unwrap()).await.unwrap();
			let NotificationsOutOpen { handshake, mut substream, .. } =
				OutboundUpgrade::upgrade_outbound(
					NotificationsOut::new(
						PROTO_NAME,
						Vec::new(),
						&b"initial message"[..],
						1024 * 1024,
						PeerId::random(),
					),
					socket.compat(),
					ProtocolName::Static(PROTO_NAME),
				)
				.await
				.unwrap();

			assert_eq!(handshake, b"hello world");

			future::poll_fn(|cx| match Pin::new(&mut substream).poll_flush(cx) {
				Poll::Pending => Poll::Pending,
				Poll::Ready(Ok(())) => {
					cx.waker().wake_by_ref();
					Poll::Pending
				},
				Poll::Ready(Err(e)) => {
					assert!(matches!(e, NotificationsOutError::Closed));
					Poll::Ready(())
				},
			})
			.await;
		});

		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		listener_addr_tx.send(listener.local_addr().unwrap()).unwrap();

		let (socket, _) = listener.accept().await.unwrap();
		let NotificationsInOpen { handshake, mut substream, .. } = InboundUpgrade::upgrade_inbound(
			NotificationsIn::new(PROTO_NAME, Vec::new(), 1024 * 1024),
			socket.compat(),
			ProtocolName::Static(PROTO_NAME),
		)
		.await
		.unwrap();

		assert_eq!(handshake, b"initial message");

		// Send the handhsake.
		substream.send_handshake(&b"hello world"[..]);
		future::poll_fn(|cx| Pin::new(&mut substream).poll_process(cx)).await.unwrap();

		drop(substream);

		client.await.unwrap();
	}
}
