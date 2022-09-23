/// Transports define the underlying method used to send and receive raw bytes. This is
/// usually an AF_UNIX socket set to SOCK_DGRAM but premature abstraction has led to
/// this being swappable if ever necessary (say: AF_INET + SOCK_STREAM).
pub mod transport; 

/// Protocols define how to interpret the bytes sent over a `Transport` into the message
/// objects expected.
pub mod protocol;

use transport::Transport;

use protocol::message::encoding::*;
use protocol::message::*;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::str;

#[derive(Debug)]
pub enum ChannelError {
    TransportError(transport::TransportError),
    ProtocolError(protocol::ProtocolError),
    EncodingError(protocol::message::encoding::EncodingError),
}

impl From<transport::TransportError> for ChannelError {
    fn from(err: transport::TransportError) -> Self {
        Self::TransportError(err)
    }
}

impl From<protocol::ProtocolError> for ChannelError {
    fn from(err: protocol::ProtocolError) -> Self {
        Self::ProtocolError(err)
    }
}

impl From<protocol::message::encoding::EncodingError> for ChannelError {
    fn from(err: protocol::message::encoding::EncodingError) -> Self {
        Self::EncodingError(err)
    }
}

/// The `Channel` trait encapsulates bidirectional communication between the buildroot
/// where modules are executed and the host system. It allows for sending `Message`'s back
/// and forth and the setup of communications.
pub trait Channel {
    /// Prepare a channel with new_default settings as proposed by the `osbuild` version.
    fn new_default() -> Result<Self, ChannelError>
    where
        Self: Sized;

    /// Open the channel to a given destination, the destination is dependent on the
    /// transport used in the implementation.
    fn open(&mut self, dst: &str) -> Result<(), ChannelError>;

    /// Send a `Message` across the `Channel` using the encoding specified by the protocol
    /// used in the implementation.
    fn send<T: Message + Serialize>(&mut self, object: T) -> Result<usize, ChannelError>;

    /// Send a `Message` and receive a `Message` across the `Channel`.
    fn send_and_recv<T0: Message + Serialize, T1: Message + DeserializeOwned>(
        &mut self,
        object: T0,
    ) -> Result<T1, ChannelError>;

    /// Receive a `Message` across the `Channel`, you have to indicate the type of Message
    /// you want to receive.
    fn recv<T: Message + DeserializeOwned>(&mut self) -> Result<T, ChannelError>;

    fn close(&mut self) -> Result<(), ChannelError>;
}

/// `CommandChannel` is used to receive and send commands from and to the host system.
pub struct CommandChannel {
    pub transport: Box<dyn transport::Transport>,
    pub protocol: Box<dyn protocol::Protocol>,
}

impl Channel for CommandChannel {
    fn new_default() -> Result<Self, ChannelError> {
        Ok(Self {
            transport: Box::new(transport::UnixDGRAMSocket::new(
                "/run/osbuild/api/log".to_string(),
                None,
            )?),
            protocol: Box::new(protocol::JSONProtocol {}),
        })
    }

    fn send<T: Message + Serialize>(&mut self, object: T) -> Result<usize, ChannelError> {
        let enc = JSONEncoding {};

        Ok(self.transport.send_all(&enc.encode(object)?)?)
    }

    fn recv<T: Message + DeserializeOwned>(&mut self) -> Result<T, ChannelError> {
        let enc = JSONEncoding {};

        // XXX let the protocol handle this, it knows boundaries for encoded messages
        let mut dat = vec![0u8; 1024];

        self.transport.recv(&mut dat)?;

        Ok(enc.decode::<T>(str::from_utf8(&dat).unwrap())?)
    }

    fn send_and_recv<T0: Message + Serialize, T1: Message + DeserializeOwned>(
        &mut self,
        object: T0,
    ) -> Result<T1, ChannelError> {
        let enc = JSONEncoding {};

        self.transport.send_all(&enc.encode(object)?)?;

        // XXX let the protocol handle this, it knows boundaries for encoded messages
        let mut dat = vec![0u8; 1024];

        self.transport.recv(&mut dat)?;

        Ok(enc.decode::<T1>(str::from_utf8(&dat).unwrap())?)
    }

    fn open(&mut self, _path: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), ChannelError> {
        self.transport.close()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::remove_file;
    use std::os::unix::net::UnixDatagram;

    use super::*;

    #[test]
    fn command_channel_send() {
        let path = "/tmp/channel";
        let sock = UnixDatagram::bind(path.to_string()).unwrap();

        let mut channel = CommandChannel {
            transport: Box::new(
                transport::UnixDGRAMSocket::new(path.to_string(), None).unwrap(),
            ),
            protocol: Box::new(protocol::JSONProtocol {}),
        };

        let method = Method {
            r#type: MessageType::Method,
            method: "test".to_string(),
            data: MethodData {
                name: "name".to_string(),
            },
        };

        let size = channel.send(method).unwrap();
        let mut buffer = vec![0; size];

        sock.recv_from(buffer.as_mut_slice()).unwrap();

        // XXX kinda weird, do we want to take this from an encoding step instead to
        // confirm the message wasn't erroneously translated or is a literal fine?
        assert_eq!(
            buffer,
            b"{\"type\":\"Method\",\"method\":\"test\",\"data\":{\"name\":\"name\"}}"
        );

        remove_file(path).unwrap();
    }
}
