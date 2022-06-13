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

/// Traits for implementing modules such as assemblers, sources, or stages.
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
    /// machine through a transport (AF_UNIX socket). The `channel` module provides the necessary
    /// functionality for your osbuild module to do so.
    pub mod channel {
        /// Transports define the underlying method used to send and receive raw bytes. This is
        /// usually an AF_UNIX socket set to SOCK_DGRAM but premature abstraction has led to
        /// this being swappable if ever necessary (say: AF_INET + SOCK_STREAM).
        pub mod transport {
            use log::*;

            use std::fs::remove_file;
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
                    let socket = UnixDatagram::bind("")?;

                    let instance = Self { socket: socket };

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
                fn new(dst: String, src: Option<String>) -> Result<Self, TransportError> {
                    Ok(Self {
                        socket: UnixStream::connect(dst)?,
                    })
                }

                fn close(&mut self) -> Result<(), TransportError> {
                    Ok(())
                }

                fn recv(&self, buf: &mut [u8]) -> Result<usize, TransportError> {
                    Ok(1)
                }

                fn send(&self, buf: &[u8]) -> Result<usize, TransportError> {
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

            pub struct JSONProtocol {}

            impl Protocol for JSONProtocol {
                fn new() -> Result<Self, ProtocolError> {
                    Ok(Self {})
                }
            }

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

                /// The various types of objects that can be encoded and passed over the wire.
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

                impl Envelope {
                    fn new() -> Self {
                        Self {
                            r#type: "bar".to_string(),
                            data: "foo".to_string(),
                        }
                    }
                }

                pub mod encoding {
                    use super::*;

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
                        fn encode_message(
                            &self,
                            message: Message,
                        ) -> Result<Vec<u8>, EncodingError>;
                        fn decode_message(&self, message: &str) -> Result<Message, EncodingError>;

                        fn encode_method(&self, method: Method) -> Result<Vec<u8>, EncodingError>;
                        fn decode_method(&self, method: &str) -> Result<Method, EncodingError>;

                        fn encode_reply(&self, reply: Reply) -> Result<Vec<u8>, EncodingError>;
                        fn decode_reply(&self, reply: &str) -> Result<Reply, EncodingError>;

                        fn encode_signal(&self, signal: Signal) -> Result<Vec<u8>, EncodingError>;
                        fn decode_signal(&self, signal: &str) -> Result<Signal, EncodingError>;

                        fn encode_exception(
                            &self,
                            exception: Exception,
                        ) -> Result<Vec<u8>, EncodingError>;
                        fn decode_exception(
                            &self,
                            exception: &str,
                        ) -> Result<Exception, EncodingError>;
                    }

                    pub struct JSONEncoding {}

                    impl Encoding for JSONEncoding {
                        fn encode_message(
                            &self,
                            message: Message,
                        ) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_string(&message)?
                                .as_str()
                                .as_bytes()
                                .to_vec())
                        }

                        fn decode_message(&self, message: &str) -> Result<Message, EncodingError> {
                            Ok(Message {})
                        }

                        fn encode_method(&self, method: Method) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_string(&method)?.as_str().as_bytes().to_vec())
                        }

                        fn decode_method(&self, method: &str) -> Result<Method, EncodingError> {
                            Ok(Method {})
                        }

                        fn encode_reply(&self, reply: Reply) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_string(&reply)?.as_str().as_bytes().to_vec())
                        }

                        fn decode_reply(&self, reply: &str) -> Result<Reply, EncodingError> {
                            Ok(Reply {})
                        }

                        fn encode_signal(&self, signal: Signal) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_string(&signal)?.as_str().as_bytes().to_vec())
                        }

                        fn decode_signal(&self, signal: &str) -> Result<Signal, EncodingError> {
                            Ok(Signal {})
                        }

                        fn encode_exception(
                            &self,
                            exception: Exception,
                        ) -> Result<Vec<u8>, EncodingError> {
                            Ok(serde_json::to_string(&exception)?
                                .as_str()
                                .as_bytes()
                                .to_vec())
                        }

                        fn decode_exception(
                            &self,
                            exception: &str,
                        ) -> Result<Exception, EncodingError> {
                            Ok(Exception {})
                        }
                    }

                    #[cfg(test)]
                    mod test {
                        use std::str;

                        use super::*;

                        #[test]
                        fn test_encode_message() {
                            let encoding = JSONEncoding {};
                            let message = Message {};

                            assert!(encoding.encode_message(message).is_ok());

                            let data = encoding.encode_message(message).unwrap();

                            assert!(encoding
                                .decode_message(str::from_utf8(&data).unwrap())
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_reply() {
                            let encoding = JSONEncoding {};
                            let reply = Reply {};

                            assert!(encoding.encode_reply(reply).is_ok());

                            let data = encoding.encode_reply(reply).unwrap();

                            assert!(encoding
                                .decode_reply(str::from_utf8(&data).unwrap())
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_method() {
                            let encoding = JSONEncoding {};
                            let method = Method {};

                            assert!(encoding.encode_method(method).is_ok());

                            let data = encoding.encode_method(method).unwrap();

                            assert!(encoding
                                .decode_method(str::from_utf8(&data).unwrap())
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_signal() {
                            let encoding = JSONEncoding {};
                            let signal = Signal {};

                            assert!(encoding.encode_signal(signal).is_ok());

                            let data = encoding.encode_signal(signal).unwrap();

                            assert!(encoding
                                .decode_signal(str::from_utf8(&data).unwrap())
                                .is_ok());
                        }

                        #[test]
                        fn test_encode_exception() {
                            let encoding = JSONEncoding {};
                            let exception = Exception {};

                            assert!(encoding.encode_exception(exception).is_ok());

                            let data = encoding.encode_exception(exception).unwrap();

                            assert!(encoding
                                .decode_exception(str::from_utf8(&data).unwrap())
                                .is_ok());
                        }
                    }
                }
            }
        }

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
            fn open(&mut self, dst: &str) -> Result<(), ChannelError>;
            fn close(&mut self) -> Result<(), ChannelError>;
        }

        /// The CommandChannel is used to receive commands from the host system.
        pub struct CommandChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for CommandChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
                Ok(())
            }

            fn close(&mut self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// The LogChannel is used to send logs back to the host system.
        pub struct LogChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for LogChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
                Ok(())
            }

            fn close(&mut self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// The ProgressChannel is used send progress information back to the host system.
        pub struct ProgressChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for ProgressChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
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
