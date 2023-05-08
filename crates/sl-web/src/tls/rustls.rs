/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_web
 * Module:  tls/rustls
 *
 * Purpose:
 *   Wrapper around Rustls for constructing the stream acceptor.
 *
 */

use std::{
    fs::File,
    io::BufReader,
    sync::Arc,
};

use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{
        Certificate,
        PrivateKey,
        ServerConfig,
    },
    TlsAcceptor,
    TlsStream,
};

use super::{
    Error,
    Result,
};


#[derive(Clone)]
pub struct Acceptor(TlsAcceptor);

impl Acceptor {
    pub fn from(config: ServerConfig) -> Self { Self(TlsAcceptor::from(Arc::new(config))) }
}

impl super::Acceptor for Acceptor {
    type Stream = TlsStream<TcpStream>;

    fn accept(&self, stream: Self::Stream) -> Self::Future {
        let acceptor = self.0.acceptor.clone();
        acceptor.accept(stream)
    }
}


pub fn from_cert_and_key() -> Result<impl super::Acceptor> {
    let config = load_tls_config();
    Acceptor::from(config)
}


fn load_tls_config() -> Result<ServerConfig> {
    let certs = load_certs(".cert/config/live/local.vroov.com/fullchain.pem")?;
    let key = load_private_key(".cert/config/live/local.vroov.com/privkey.pem")?;

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|_| Error::TlsConfigError)
}

fn load_certs(filename: &str) -> Result<Vec<Certificate>> {
    let certfile = File::open(filename)?;
    let mut buf = BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut buf)?
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect()
        .map_err(|_| Error::CertificateLoadError);

    Ok(certs)
}

fn load_private_key(filename: &str) -> Result<PrivateKey> {
    let keyfile = File::open(filename)?;
    let mut buf = BufReader::new(keyfile);

    match rustls_pemfile::read_one(&mut buf)? {
        | Some(rustls_pemfile::Item::RSAKey(key)) => Ok(PrivateKey(key)),
        | Some(rustls_pemfile::Item::PKCS8Key(key)) => Ok(PrivateKey(key)),
        | Some(rustls_pemfile::Item::ECKey(key)) => Ok(PrivateKey(key)),
        | _ => Err(Error::PrivateKeyLoadError),
    }
}
