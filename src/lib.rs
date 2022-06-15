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
                    let size = self.socket.recv(buf)?;

                    Ok(size)
                }

                fn send(&self, buf: &[u8]) -> Result<usize, TransportError> {
                    let size = self.socket.send(buf)?;

                    Ok(size)
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

                #[derive(Debug)]
                pub enum MessageError {}

                /// All types of objects are contained inside a wrapper object which contains the type and
                /// the data used.
                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Envelope {
                    r#type: String,
                    data: String,
                }

                #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
                pub struct Message {}

                #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
                pub struct Method {}

                #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
                pub struct Reply {}

                #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
                pub struct Signal {}

                #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
                pub struct Exception {}

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
                        fn test_encode_message() {
                            let encoding = JSONEncoding {};
                            let message = Message {};

                            assert!(encoding.encode(message).is_ok());
                            assert!(encoding
                                .decode::<Message>(
                                    str::from_utf8(&encoding.encode(message).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_reply() {
                            let encoding = JSONEncoding {};
                            let reply = Reply {};

                            assert!(encoding.encode(reply).is_ok());
                            assert!(encoding
                                .decode::<Reply>(
                                    str::from_utf8(&encoding.encode(reply).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_method() {
                            let encoding = JSONEncoding {};
                            let method = Method {};

                            assert!(encoding.encode(method).is_ok());
                            assert!(encoding
                                .decode::<Method>(
                                    str::from_utf8(&encoding.encode(method).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_signal() {
                            let encoding = JSONEncoding {};
                            let signal = Signal {};

                            assert!(encoding.encode(signal).is_ok());
                            assert!(encoding
                                .decode::<Signal>(
                                    str::from_utf8(&encoding.encode(signal).unwrap()).unwrap()
                                )
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_exception() {
                            let encoding = JSONEncoding {};
                            let exception = Exception {};

                            assert!(encoding.encode(exception).is_ok());
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

        #[derive(Debug)]
        pub enum ChannelError {
            TransportError(transport::TransportError),
            ProtocolError(protocol::ProtocolError),
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

        pub trait Channel {
            /// Open a channel with new_default settings as proposed by the `lib_osbuild` version.
            fn new_default() -> Result<Self, ChannelError>
            where
                Self: Sized;

            fn open(&mut self, dst: &str) -> Result<(), ChannelError>;
            fn close(&mut self) -> Result<(), ChannelError>;
        }

        /// Used to receive commands from the host system.
        pub struct CommandChannel {
            transport: Box<dyn transport::Transport>,
            _protocol: Box<dyn protocol::Protocol>,
        }

        impl Channel for CommandChannel {
            fn new_default() -> Result<Self, ChannelError> {
                Ok(Self {
                    transport: Box::new(transport::UnixDGRAMSocket::new(
                        "/run/osbuild/api/log".to_string(),
                        None,
                    )?),
                    _protocol: Box::new(protocol::JSONProtocol {}),
                })
            }

            fn open(&mut self, _path: &str) -> Result<(), ChannelError> {
                Ok(())
            }

            fn close(&mut self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// Used to send logs back to the host system.
        pub struct LogChannel {
            transport: Box<dyn transport::Transport>,
            _protocol: Box<dyn protocol::Protocol>,
        }

        impl Channel for LogChannel {
            fn new_default() -> Result<Self, ChannelError> {
                Ok(Self {
                    transport: Box::new(transport::UnixDGRAMSocket::new(
                        "/run/osbuild/api/log".to_string(),
                        None,
                    )?),
                    _protocol: Box::new(protocol::JSONProtocol {}),
                })
            }

            fn open(&mut self, _path: &str) -> Result<(), ChannelError> {
                Ok(())
            }

            fn close(&mut self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// Used send progress information back to the host system.
        pub struct ProgressChannel {
            transport: Box<dyn transport::Transport>,
            _protocol: Box<dyn protocol::Protocol>,
        }

        impl Channel for ProgressChannel {
            fn new_default() -> Result<Self, ChannelError> {
                Ok(Self {
                    transport: Box::new(transport::UnixDGRAMSocket::new(
                        "/run/osbuild/api/progress".to_string(),
                        None,
                    )?),
                    _protocol: Box::new(protocol::JSONProtocol {}),
                })
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
