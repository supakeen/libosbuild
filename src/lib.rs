// You're reading the source code for `lib_osbuild`, a Rust library that provides the primitives
// to implement modules for `osbuild`.
//
// OSBuild is a pipeline-based build system for operating system artifacts. It defines a
// universal pipeline description and a build system to execute them, producing artifacts like
// operating system images, working towards an image build pipeline that is more comprehensible,
// reproducible, and extendable.
//
// You can find out more on [osbuild's homepage](https://osbuild.org/) or
// [osbuild's GitHub](https://github.com/osbuild/osbuild).

/// Traits for implementing osbuild modules such as assemblers, sources, or stages.
pub mod module {
    #[derive(Debug)]
    pub enum AssemblerError {}

    pub trait Assembler {}

    #[derive(Debug)]
    pub enum SourceError {}

    pub trait Source {
        fn cached(&self) -> Result<bool, SourceError>;

        fn fetch_all(&self) -> Result<(), SourceError>;
        fn fetch_one(&self) -> Result<(), SourceError>;
    }

    #[derive(Debug)]
    pub enum StageError {}

    pub trait Stage {}

    /// Modules are executed in a sandbox and talk to the main osbuild process on the host
    /// machine through a transport. The `channel` module provides abstractions for an `osbuild`
    /// module to talk to the host system.
    pub mod channel {
        /// Transports define the underlying method used to send and receive raw bytes. This is
        /// usually an AF_UNIX socket set to SOCK_DGRAM but premature abstraction has led to
        /// this being swappable if ever necessary (say: AF_INET + SOCK_STREAM).
        pub mod transport {
            use std::net::Shutdown;
            use std::os::unix::net::{UnixDatagram, UnixStream};

            #[derive(Debug)]
            pub enum TransportError {
                IOError(std::io::Error),
                SocketError,
            }

            impl From<std::io::Error> for TransportError {
                fn from(err: std::io::Error) -> Self {
                    Self::IOError(err)
                }
            }

            pub trait Transport {
                fn new(dst: String, src: Option<String>) -> Result<Self, TransportError>
                where
                    Self: Sized;

                fn close(&mut self) -> Result<(), TransportError>;

                fn recv(&self, buf: &mut [u8]) -> Result<usize, TransportError>;
                fn send(&self, buf: &[u8]) -> Result<usize, TransportError>;
                fn send_all(&self, buf: &[u8]) -> Result<usize, TransportError>;
            }

            /// A UnixDGRAMSocket Transport to send data back and forth over a SOCK_DGRAM, AF_UNIX
            /// socket.
            pub struct UnixDGRAMSocket {
                socket: UnixDatagram,
            }

            impl Transport for UnixDGRAMSocket {
                fn new(dst: String, src: Option<String>) -> Result<Self, TransportError> {
                    let socket = UnixDatagram::bind(src.unwrap_or_else(|| "".to_string()))?;

                    let instance = Self { socket };

                    instance.socket.connect(dst)?;

                    Ok(instance)
                }

                fn close(&mut self) -> Result<(), TransportError> {
                    self.socket.shutdown(Shutdown::Both)?;

                    Ok(())
                }

                fn recv(&self, buf: &mut [u8]) -> Result<usize, TransportError> {
                    Ok(self.socket.recv(buf)?)
                }

                fn send(&self, buf: &[u8]) -> Result<usize, TransportError> {
                    Ok(self.socket.send(buf)?)
                }

                fn send_all(&self, buf: &[u8]) -> Result<usize, TransportError> {
                    let mut sent = 0;

                    while sent < buf.len() {
                        sent += self.send(buf)?;
                    }

                    Ok(sent)
                }
            }

            /// A UnixSTREAMSocket Transport to send data back and forth over a SOCK_STREAM, AF_UNIX
            /// socket.
            pub struct UnixSTREAMSocket {
                socket: UnixStream,
            }

            impl Transport for UnixSTREAMSocket {
                fn new(dst: String, _src: Option<String>) -> Result<Self, TransportError> {
                    Ok(Self {
                        socket: UnixStream::connect(dst)?,
                    })
                }

                fn close(&mut self) -> Result<(), TransportError> {
                    self.socket.shutdown(Shutdown::Both)?;

                    Ok(())
                }

                fn recv(&self, _buf: &mut [u8]) -> Result<usize, TransportError> {
                    Ok(1)
                }

                fn send(&self, _buf: &[u8]) -> Result<usize, TransportError> {
                    Ok(1)
                }

                fn send_all(&self, buf: &[u8]) -> Result<usize, TransportError> {
                    let mut sent = 0;

                    while sent < buf.len() {
                        sent += self.send(buf)?;
                    }

                    Ok(sent)
                }
            }

            #[cfg(test)]
            mod test {
                use super::*;

                use std::fs::remove_file;
                use std::os::unix::net::UnixDatagram;

                #[test]
                fn unixdgramsocket_non_existent_path() {
                    assert!(UnixDGRAMSocket::new("/tmp/non-existent".to_string(), None).is_err());
                }

                #[test]
                fn unixdgramsocket_non_existent_directory() {
                    assert!(
                        UnixDGRAMSocket::new("/non-existent/non-existent".to_string(), None)
                            .is_err()
                    );
                }

                #[test]
                fn unixdgramsocket_exists() {
                    // XXX can we use autobound sockets here as well?
                    let path = "/tmp/socket";

                    let _sock = UnixDatagram::bind(path).unwrap();

                    assert!(UnixDGRAMSocket::new(path.to_string(), None).is_ok());

                    remove_file(path).unwrap();
                }

                #[test]
                fn unixdgramsocket_send() {
                    // XXX can we use autobound sockets here as well?
                    let path = "/tmp/socket-send";
                    let sock = UnixDatagram::bind(path).unwrap();

                    let transport = UnixDGRAMSocket::new(path.to_string(), None).unwrap();
                    transport.send(b"foo").unwrap();

                    let mut buffer = vec![0; 3];
                    sock.recv_from(buffer.as_mut_slice()).unwrap();

                    assert_eq!(buffer, b"foo");

                    remove_file(path).unwrap();
                }

                #[test]
                fn unixstreamsocket_non_existent_path() {
                    assert!(UnixSTREAMSocket::new("/tmp/non-existent".to_string(), None).is_err());
                }

                #[test]
                fn unixstreamsocket_non_existent_directory() {
                    assert!(
                        UnixSTREAMSocket::new("/non-existent/non-existent".to_string(), None)
                            .is_err()
                    );
                }
            }
        }

        /// Protocols define how to interpret the bytes sent over a `Transport` into the message
        /// objects expected.
        pub mod protocol {
            #[derive(Debug)]
            pub enum ProtocolError {}

            pub trait Protocol {
                fn new() -> Result<Self, ProtocolError>
                where
                    Self: Sized;
            }

            /// Encodes messages as JSON.
            pub struct JSONProtocol {}

            impl Protocol for JSONProtocol {
                fn new() -> Result<Self, ProtocolError> {
                    Ok(Self {})
                }
            }

            /// Message types that exist in the protocols. Some of these messages can only be sent
            /// over certain types of transports).
            pub mod message {
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub enum MessageType {
                    Method,
                    Reply,
                    Signal,
                    Exception,
                }

                #[derive(Debug)]
                pub enum MessageError {}

                pub trait Message {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct MethodData {
                    name: String,
                }

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Method {
                    r#type: MessageType,
                    method: String,
                    data: MethodData,
                }

                impl Message for Method {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct ReplyData {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Reply {
                    r#type: MessageType,
                    data: ReplyData,
                }

                impl Message for Reply {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct SignalData {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Signal {
                    r#type: MessageType,
                    data: SignalData,
                }

                impl Message for Signal {}

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct ExceptionData {
                    name: String,
                    value: String,
                    backtrace: String,
                }

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Exception {
                    r#type: MessageType,
                    data: ExceptionData,
                }

                impl Message for Exception {}

                pub mod encoding {
                    use super::*;
                    use serde::de::DeserializeOwned;

                    #[derive(Debug)]
                    pub enum EncodingError {
                        ParseError(serde_json::Error),
                    }

                    impl From<serde_json::Error> for EncodingError {
                        fn from(err: serde_json::Error) -> Self {
                            Self::ParseError(err)
                        }
                    }

                    pub trait Encoding {
                        fn encode<T: Serialize>(&self, object: T)
                            -> Result<Vec<u8>, EncodingError>;
                        fn decode<T: DeserializeOwned>(
                            &self,
                            data: &str,
                        ) -> Result<T, EncodingError>;
                    }

                    pub struct JSONEncoding {}

                    impl Encoding for JSONEncoding {
                        fn encode<T: Serialize>(
                            &self,
                            object: T,
                        ) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_vec(&object)?)
                        }

                        fn decode<T: DeserializeOwned>(
                            &self,
                            data: &str,
                        ) -> Result<T, EncodingError> {
                            Ok(serde_json::from_str(data)?)
                        }
                    }

                    #[cfg(test)]
                    mod test {
                        use super::*;
                        use std::str;

                        #[test]
                        fn test_encode_reply() {
                            let encoding = JSONEncoding {};
                            let reply = Reply {
                                r#type: MessageType::Reply,
                                data: ReplyData {},
                            };

                            assert!(encoding
                                .decode::<Reply>(
                                    str::from_utf8(&encoding.encode(reply).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_method() {
                            let encoding = JSONEncoding {};
                            let method = Method {
                                r#type: MessageType::Method,
                                method: "test".to_string(),
                                data: MethodData {
                                    name: "name".to_string(),
                                },
                            };

                            assert!(encoding
                                .decode::<Method>(
                                    str::from_utf8(&encoding.encode(method).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_signal() {
                            let encoding = JSONEncoding {};
                            let signal = Signal {
                                r#type: MessageType::Signal,
                                data: SignalData {},
                            };

                            assert!(encoding
                                .decode::<Signal>(
                                    str::from_utf8(&encoding.encode(signal).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_exception() {
                            let encoding = JSONEncoding {};
                            let exception = Exception {
                                r#type: MessageType::Exception,
                                data: ExceptionData {
                                    name: "foo".to_string(),
                                    value: "foo".to_string(),
                                    backtrace: "foo".to_string(),
                                },
                            };

                            assert!(encoding
                                .decode::<Exception>(
                                    str::from_utf8(&encoding.encode(exception).unwrap()).unwrap()
                                )
                                .is_ok());
                        }
                    }
                }
            }
        }

        use transport::Transport;

        use protocol::message::encoding::*;
        use protocol::message::*;

        use serde::Serialize;

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
            /// Prepare a channel with new_default settings as proposed by the `lib_osbuild` version.
            fn new_default() -> Result<Self, ChannelError>
            where
                Self: Sized;

            /// Open the channel to a given destination, the destination is dependent on the
            /// transport used in the implementation.
            fn open(&mut self, dst: &str) -> Result<(), ChannelError>;

            /// Send a `Message` across the `Channel` using the encoding specified by the protocol
            /// used in the implementation.
            fn send<T: Message + Serialize>(&mut self, object: T) -> Result<(), ChannelError>;

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

            fn send<T: Message + Serialize>(&mut self, object: T) -> Result<(), ChannelError> {
                let enc = JSONEncoding {};

                self.transport.send_all(&enc.encode(object)?)?;

                Ok(())
            }

            fn open(&mut self, _path: &str) -> Result<(), ChannelError> {
                Ok(())
            }

            fn close(&mut self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }
    }
}

/// Traits for handling osbuild manifest files.
pub mod manifest {}
