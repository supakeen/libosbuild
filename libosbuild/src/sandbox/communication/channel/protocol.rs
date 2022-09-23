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
            fn encode<T: Serialize>(&self, object: T) -> Result<Vec<u8>, EncodingError>;
            fn decode<T: DeserializeOwned>(&self, data: &str) -> Result<T, EncodingError>;
        }

        pub struct JSONEncoding {}

        impl Encoding for JSONEncoding {
            fn encode<T: Serialize>(&self, object: T) -> Result<Vec<u8>, EncodingError> {
                Ok(serde_json::to_vec(&object)?)
            }

            fn decode<T: DeserializeOwned>(&self, data: &str) -> Result<T, EncodingError> {
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
                    .decode::<Reply>(str::from_utf8(&encoding.encode(reply).unwrap()).unwrap())
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
                    .decode::<Method>(str::from_utf8(&encoding.encode(method).unwrap()).unwrap())
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
                    .decode::<Signal>(str::from_utf8(&encoding.encode(signal).unwrap()).unwrap())
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
