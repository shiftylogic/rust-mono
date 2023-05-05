/*++
 *
 * Crate: sl_web
 *
 */

use std::{
    fs::File,
    io::BufReader,
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpListener,
        TcpStream,
    },
    runtime::Builder,
};
use tokio_rustls::{
    rustls::{
        Certificate,
        PrivateKey,
        ServerConfig,
    },
    TlsAcceptor,
};

#[derive(Debug)]
pub enum Error {
    IOError,
    TLSError,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        log::error!("IO Error: {:?}", err);
        Self::IOError
    }
}

impl From<tokio_rustls::rustls::Error> for Error {
    fn from(err: tokio_rustls::rustls::Error) -> Self {
        log::error!("TLS Error: {:?}", err);
        Self::TLSError
    }
}

pub fn run() -> Result<(), Error> {
    // Config
    let thread_count = 4;
    let thread_stack_size = 2 * 1024 * 1024;

    let rt = Builder::new_multi_thread()
        .worker_threads(thread_count)
        .thread_stack_size(thread_stack_size)
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(run_core())
}

async fn run_core() -> Result<(), Error> {
    let config = load_tls_config()?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        log::info!("Client connecting ({})...", peer_addr);
        tokio::spawn(accept_tls(peer_addr, stream, acceptor.clone()));
    }
}

async fn accept_tls(
    addr: SocketAddr,
    stream: TcpStream,
    acceptor: TlsAcceptor,
) -> Result<(), Error> {
    let mut stream = acceptor.accept(stream).await?;
    let mut output = tokio::io::sink();

    stream
        .write_all(
            &b"HTTP/1.1 200 OK\n\nConnection: close\r\nContent-Length: 11\r\n\r\nHello dude!"[..],
        )
        .await?;
    stream.shutdown().await?;
    tokio::io::copy(&mut stream, &mut output).await?;

    log::info!("Hello: {}", addr);
    Ok(())
}

async fn handler() -> &'static str { "Hello, dude!" }

fn load_tls_config() -> Result<ServerConfig, Error> {
    let certs = load_certs(".cert/config/live/local.vroov.com/fullchain.pem")?;
    let key = load_private_key(".cert/config/live/local.vroov.com/privkey.pem")?;

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| Error::from(err))
}

fn load_certs(filename: &str) -> Result<Vec<Certificate>, Error> {
    let certfile = File::open(filename)?;
    let mut buf = BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut buf)?
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect();

    Ok(certs)
}

fn load_private_key(filename: &str) -> Result<PrivateKey, Error> {
    let keyfile = File::open(filename)?;
    let mut buf = BufReader::new(keyfile);

    match rustls_pemfile::read_one(&mut buf)? {
        | Some(rustls_pemfile::Item::RSAKey(key)) => Ok(PrivateKey(key)),
        | Some(rustls_pemfile::Item::PKCS8Key(key)) => Ok(PrivateKey(key)),
        | Some(rustls_pemfile::Item::ECKey(key)) => Ok(PrivateKey(key)),
        | _ => Err(Error::TLSError),
    }
}
