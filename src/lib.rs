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
    pub enum AssemblerError {}

    pub trait Assembler {
        fn cached(&self) -> Result<bool, AssemblerError>;

        fn fetch_all(&self) -> Result<(), AssemblerError>;
        fn fetch_one(&self) -> Result<(), AssemblerError>;
    }

    pub enum SourceError {}

    pub trait Source {
        fn cached(&self) -> Result<bool, SourceError>;

        fn fetch_all(&self) -> Result<(), SourceError>;
        fn fetch_one(&self) -> Result<(), SourceError>;
    }

    pub enum StageError {}

    pub trait Stage {
        fn cached(&self) -> Result<bool, StageError>;

        fn fetch_all(&self) -> Result<(), StageError>;
        fn fetch_one(&self) -> Result<(), StageError>;
    }

    /// Modules are executed in a sandbox and talk to the main osbuild process on the host
    /// machine through a transport (AF_UNIX socket). The `channel` module provides the necessary
    /// functionality for your osbuild module to do so.
    pub mod channel {
        /// Transports define the underlying method used to send and receive raw bytes. This is
        /// usually an AF_UNIX socket set to SOCK_DGRAM but premature abstraction has led to
        /// this being swappable if ever necessary (say: AF_INET + SOCK_STREAM).
        pub mod transport {
            use log::*;

            use std::os::unix::net::UnixDatagram;

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
                fn open(&mut self, path: &str) -> Result<(), TransportError>;
                fn close(&self) -> Result<(), TransportError>;

                fn recv(&self, buf: &mut [u8]) -> Result<usize, TransportError>;

                fn send(&self, buf: &mut [u8]) -> Result<usize, TransportError>;

                fn new_client() -> Result<Self, TransportError>
                where
                    Self: Sized;
            }

            pub struct UnixSocket {
                path: Option<String>,
                socket: UnixDatagram,
            }

            impl Transport for UnixSocket {
                fn new_client() -> Result<Self, TransportError> {
                    Ok(Self {
                        socket: UnixDatagram::unbound()?,
                        path: None,
                    })
                }

                fn open(&mut self, path: &str) -> Result<(), TransportError> {
                    self.path = Some(path.to_string());

                    debug!("UnixSocket.open: {:?}", path);

                    Ok(self.socket.connect(path)?)
                }

                fn close(&self) -> Result<(), TransportError> {
                    debug!("UnixSocket.close: {:?}", self.path);
                    Ok(())
                }

                fn recv(&self, buf: &mut [u8]) -> Result<usize, TransportError> {
                    Ok(self.socket.recv(buf)?)
                }

                fn send(&self, buf: &mut [u8]) -> Result<usize, TransportError> {
                    Ok(self.socket.send(buf)?)
                }
            }
        }

        /// Protocols define how to interpret the bytes sent over a `Transport` into the message
        /// objects expected.
        pub mod protocol {
            pub enum ProtocolError {}
            pub trait Protocol {}

            pub mod message {
                use serde::{Deserialize, Serialize};

                pub enum MessageError {}

                /// All types of objects are contained inside a wrapper object which contains the type and
                /// the data used.
                #[derive(Serialize, Deserialize, Debug)]
                pub struct Envelope {
                    r#type: String,
                    data: String,
                }

                /// The various types of objects that can be encoded and passed over the wire.
                #[derive(Serialize, Deserialize, Debug)]
                pub struct Message {}

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Method {}

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Reply {}

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Signal {}

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Exception {}

                impl Envelope {
                    fn new() -> Self {
                        Self {
                            r#type: "bar".to_string(),
                            data: "foo".to_string(),
                        }
                    }
                }
            }

            pub mod encoding {
                use super::message::*;

                pub enum EncodingError {
                    ParseError(serde_json::Error)
                }

                impl From<serde_json::Error> for EncodingError {
                    fn from(err: serde_json::Error) -> Self {
                        Self::ParseError(err)
                    }
                }

                pub trait Encoding {
                    fn encode_message(&self, message: Message) -> Result<Vec<u8>, EncodingError>;
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
                    fn decode_exception(&self, exception: &str)
                        -> Result<Exception, EncodingError>;
                }

                pub struct JSONEncoding {}

                impl Encoding for JSONEncoding {
                    fn encode_message(&self, message: Message) -> Result<Vec<u8>, EncodingError> {
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
            }
        }

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
            fn open(&mut self, path: &str) -> Result<(), ChannelError>;
            fn close(&self) -> Result<(), ChannelError>;
        }

        /// The CommandChannel is used to receive commands from the host system.
        pub struct CommandChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            _protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for CommandChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
                self.transport.open(path)?;
                Ok(())
            }

            fn close(&self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// The LogChannel is used to send logs back to the host system.
        pub struct LogChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            _protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for LogChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
                self.transport.open(path)?;
                Ok(())
            }

            fn close(&self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }

        /// The ProgressChannel is used send progress information back to the host system.
        pub struct ProgressChannel<'a> {
            transport: &'a mut dyn transport::Transport,
            _protocol: &'a mut dyn protocol::Protocol,
        }

        impl Channel for ProgressChannel<'_> {
            fn open(&mut self, path: &str) -> Result<(), ChannelError> {
                self.transport.open(path)?;
                Ok(())
            }

            fn close(&self) -> Result<(), ChannelError> {
                self.transport.close()?;
                Ok(())
            }
        }
    }
}

/// Traits for handling osbuild manifest files.
pub mod manifest {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_better() {
        let result = 2 + 4;
        assert_eq!(result, 6);
    }
}
