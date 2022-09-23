// You're reading the source code for `libosbuild`, a Rust library that provides the primitives
// to implement modules for `osbuild`.
// OSBuild is a pipeline-based build system for operating system artifacts. It defines a
// universal pipeline description and a build system to execute them, producing artifacts like
// operating system images, working towards an image build pipeline that is more comprehensible,
// reproducible, and extendable.
//
// You can find out more on [osbuild's homepage](https://osbuild.org/) or
// [osbuild's GitHub](https://github.com/osbuild/osbuild).

/// Core tasks, providing all functionality of the main `osbuild` executable.
pub mod core {}

/// Preprocessor tasks, providing all functionality of the `osbuild-mpp` executable.
pub mod preprocessor {
    #[derive(Debug)]
    pub enum PreprocessorError {}
}

/// Manifest tasks
pub mod manifest {
    #[derive(Debug)]
    pub enum ManifestError {}

    pub enum Version {
        V1,
        V2,
    }

    pub struct Manifest {
        version: Version,
    }

    pub struct Validator {
        manifest: Manifest,
    }

    /// Manifests are described in JSON, this module provides functions and objects to parse those
    /// JSON descriptions into manifests.
    pub mod description {
        /// Version 1 of manifest descriptions, this version is *DEPRECATED*.
        pub mod v1 {
            pub struct ManifestDescription {}

            impl ManifestDescription {
                fn load(&self) {}
                fn load_assembler(&self) {}
                fn load_build(&self) {}
                fn load_pipeline(&self) {}
                fn load_source(&self) {}
                fn load_stage(&self) {}
            }

            pub struct Validator {
                manifest: Manifest,
            }

            impl Validator {
                fn validate_module(&self) {}
                fn validate_pipeline(&self) {}
                fn validate_stage(&self) {}
                fn validate_stage_modules(&self) {}
            }
        }

        /// Version 2 of manifest descriptions, this version is current.
        pub mod v2 {
            pub struct ManifestDescription {}

            impl ManifestDescription {
                fn load(&self) {}
                fn load_device(&self) {}
                fn load_input(&self) {}
                fn load_mount(&self) {}
                fn load_pipeline(&self) {}
                fn load_stage(&self) {}
            }

            fn describe(manifest: Manifest, with_id: bool) {}

            pub struct Validator {
                manifest: Manifest,
            }

            impl Validator {
                fn validate_module(&self) {}
                fn validate_pipeline(&self) {}
                fn validate_stage(&self) {}
                fn validate_stage_modules(&self) {}
            }
        }
    }
}

/// Dependency tasks
pub mod dependency {
    pub mod solver {}
}

/// Sandbox tasks
pub mod sandbox {}

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
}

/// Communication tasks
pub mod communication {
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

                use std::panic;

                use std::fs::{metadata, remove_file};
                use std::os::unix::net::UnixDatagram;

                use rand::distributions::Alphanumeric;
                use rand::{thread_rng, Rng};

                fn with_path<T>(test: T) -> ()
                where
                    T: FnOnce(&str) + panic::UnwindSafe,
                {
                    // generate a path that does not exist
                    let data = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(32)
                        .map(char::from)
                        .collect::<String>();

                    let path = data.as_str();

                    let result = panic::catch_unwind(|| test(path));

                    // remove the path again if it was created during the test
                    if metadata(path).is_ok() {
                        remove_file(path).expect("Unable to remove test file.");
                    }

                    assert!(result.is_ok());
                }

                #[test]
                fn unixdgramsocket_non_existent_path() {
                    with_path(|path| {
                        assert!(UnixDGRAMSocket::new(path.to_string(), None).is_err());
                    })
                }

                #[test]
                fn unixdgramsocket_exists() {
                    with_path(|path| {
                        let _sock = UnixDatagram::bind(path).unwrap();
                        assert!(UnixDGRAMSocket::new(path.to_string(), None).is_ok());
                    })
                }

                #[test]
                fn unixdgramsocket_send() {
                    with_path(|path| {
                        let sock = UnixDatagram::bind(path).unwrap();

                        let transport = UnixDGRAMSocket::new(path.to_string(), None).unwrap();
                        transport.send(b"foo").unwrap();

                        let mut buffer = vec![0; 3];
                        sock.recv_from(buffer.as_mut_slice()).unwrap();

                        assert_eq!(buffer, b"foo");
                    })
                }

                #[test]
                fn unixdgramsocket_send_all() {
                    with_path(|path| {
                        let sock = UnixDatagram::bind(path).unwrap();

                        let transport = UnixDGRAMSocket::new(path.to_string(), None).unwrap();
                        transport.send_all(b"foo").unwrap();

                        let mut buffer = vec![0; 3];
                        sock.recv_from(buffer.as_mut_slice()).unwrap();

                        assert_eq!(buffer, b"foo");
                    })
                }

                #[test]
                fn unixstreamsocket_non_existent_path() {
                    with_path(|path| {
                        assert!(UnixSTREAMSocket::new(path.to_string(), None).is_err());
                    })
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
                    pub name: String,
                }

                #[derive(Serialize, Deserialize, Debug, Clone)]
                pub struct Method {
                    pub r#type: MessageType,
                    pub method: String,
                    pub data: MethodData,
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
    }
}
