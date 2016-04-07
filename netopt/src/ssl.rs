use std::net::TcpStream;
use std::io;
use std::sync::Arc;
use std::path::Path;
use openssl::ssl::{self, SslMethod, SSL_VERIFY_NONE};
use openssl::x509::X509FileType;

pub type SslStream = ssl::SslStream<TcpStream>;
pub type SslError = ssl::error::SslError;

#[derive(Debug, Clone)]
pub struct SslContext {
    inner: Arc<ssl::SslContext>
}

impl Default for SslContext {
    fn default() -> SslContext {
        SslContext {
            inner: Arc::new(ssl::SslContext::new(SslMethod::Tlsv1_2).unwrap())
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
        let mut ctx = try!(ssl::SslContext::new(SslMethod::Sslv23));
        try!(ctx.set_cipher_list("DEFAULT"));
        try!(ctx.set_certificate_file(cert.as_ref(), X509FileType::PEM));
        try!(ctx.set_private_key_file(key.as_ref(), X509FileType::PEM));
        ctx.set_verify(SSL_VERIFY_NONE, None);
        Ok(SslContext { inner: Arc::new(ctx) })
    }

    pub fn accept(&self, stream: TcpStream) -> Result<SslStream, io::Error> {
        match ssl::SslStream::accept(&*self.inner, stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, err).into())
        }
    }

    pub fn connect(&self, stream: TcpStream) -> Result<SslStream, io::Error> {
        match ssl::SslStream::connect(&*self.inner, stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, err).into())
        }
    }
}
