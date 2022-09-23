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
