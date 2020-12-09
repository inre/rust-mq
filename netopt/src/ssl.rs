use std::net::TcpStream;
use std::io;
use std::sync::Arc;
use std::path::Path;
use openssl::ssl::{self, SslFiletype, SslMethod, SslVerifyMode};

pub type SslStream = ssl::SslStream<TcpStream>;
pub type SslError = ssl::Error;

#[derive(Debug, Clone)]
pub struct SslContext {
    inner: Arc<ssl::SslContext>
}

impl Default for SslContext {
    fn default() -> SslContext {
        SslContext {
            inner: Arc::new(ssl::SslContext::builder(SslMethod::tls()).unwrap().build())
        }
    }
}

impl SslContext {
    pub fn new(context: ssl::SslContext) -> Self {
        SslContext {
            inner: Arc::new(context)
        }
    }

    pub fn with_cert_and_key<C, K>(cert: C, key: K) -> Result<SslContext, SslError>
    where C: AsRef<Path>, K: AsRef<Path> {
        let mut ctx = ssl::SslContext::builder(SslMethod::tls())?;
        ctx.set_cipher_list("DEFAULT")?;
        ctx.set_certificate_file(cert.as_ref(), SslFiletype::PEM)?;
        ctx.set_private_key_file(key.as_ref(), SslFiletype::PEM)?;
        ctx.set_verify(SslVerifyMode::NONE);
        Ok(SslContext { inner: Arc::new(ctx.build()) })
    }

    pub fn with_cert_and_key_and_ca<C, K, A>(cert: C, key: K, ca: A) -> Result<SslContext, SslError>
    where C: AsRef<Path>, K: AsRef<Path>, A: AsRef<Path> {
        let mut ctx = ssl::SslContext::builder(SslMethod::tls())?;
        ctx.set_cipher_list("DEFAULT")?;
        ctx.set_certificate_file(cert.as_ref(), SslFiletype::PEM)?;
        ctx.set_private_key_file(key.as_ref(), SslFiletype::PEM)?;
        ctx.set_ca_file(ca.as_ref())?;
        ctx.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
        Ok(SslContext { inner: Arc::new(ctx.build()) })
    }

    pub fn accept(&self, stream: TcpStream) -> Result<SslStream, io::Error> {
        match ssl::Ssl::new(&*self.inner)?.accept(stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, err).into())
        }
    }

    pub fn connect(&self, stream: TcpStream) -> Result<SslStream, io::Error> {
        match ssl::Ssl::new(&*self.inner)?.connect(stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, err).into())
        }
    }
}
