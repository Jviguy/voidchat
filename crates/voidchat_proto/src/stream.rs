use crate::{read_packet, Packet};
use futures_lite::{AsyncRead, Stream};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

pub struct PacketStream<S: AsyncRead + Unpin> {
    socket: S,
    buf: Vec<u8>,
    temp_buf: [u8; 1024],
}

impl<S: AsyncRead + Unpin> PacketStream<S> {
    fn new(socket: S) -> Self {
        PacketStream {
            socket,
            buf: Vec::with_capacity(4096),
            temp_buf: [0u8; 1024],
        }
    }

    /// Checks if there is a full packet in the buffer.
    /// returns (payload len, packet total len)
    /// packet total len is payload len + 4 as we frame with a uint32 size.
    /// that is for now could be changed later.
    fn try_parse_packet(&self) -> Option<(usize, usize)> {
        if self.buf.len() < 4 {
            return None;
        }

        let len_bytes = self.buf[0..4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes) as usize;
        let full_len = len + 4;

        if self.buf.len() < full_len {
            // we haven't gotten the full packet
            return None;
        }

        Some((len, full_len))
    }
}

#[derive(Error, Debug)]
pub enum PacketReadError {
    #[error("Network disconnected io error.")]
    Disconnect(#[from] io::Error),
    #[error("Parse error")]
    ParseError(#[from] bincode::error::DecodeError),
}

impl<S: AsyncRead + Unpin> Unpin for PacketStream<S> {}

impl<S: AsyncRead + Unpin> Stream for PacketStream<S> {
    type Item = Result<Packet, PacketReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some((pack_len, full_len)) = this.try_parse_packet() {
                // we have a full packet yippee try to read.
                let start = full_len - pack_len;
                let mut packet_bytes = &this.buf[start..full_len];
                let packet_result = read_packet(&mut packet_bytes)?;

                this.buf.drain(0..full_len);

                return Poll::Ready(Some(Ok(packet_result)));
            } else {
                // no packet. we must poll reading.
                let read_future = Pin::new(&mut this.socket);
                match read_future.poll_read(cx, &mut this.temp_buf) {
                    // if no bytes are read, we probably errored, or we got everything.
                    Poll::Ready(Ok(0)) => {
                        if this.buf.is_empty() {
                            return Poll::Ready(None);
                        } else {
                            let err = Err(PacketReadError::from(io::Error::from(
                                io::ErrorKind::UnexpectedEof,
                            )));
                            return Poll::Ready(Some(err));
                        }
                    }
                    // if n bytes are read.
                    Poll::Ready(Ok(n)) => {
                        this.buf.extend_from_slice(&this.temp_buf[..n]);
                        // loop again to see if we have a full packet. if not
                        // we will come back here and read n bytes again or poll,
                        // or eof or etc.
                        continue;
                    }
                    // if there's an error just map it in.
                    Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(PacketReadError::from(e)))),
                    // if its still working then we just yield the task.
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            }
        }
    }
}
