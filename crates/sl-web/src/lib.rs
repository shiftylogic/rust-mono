/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate: sl_web
 *
 */

use std::net::SocketAddr;

use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpListener,
        TcpStream,
    },
    runtime::Builder,
};

mod tls;


#[derive(Debug)]
pub enum Error {
    IOError,
    TlsError,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        log::error!("IO Error: {:?}", err);
        Self::IOError
    }
}

impl From<tls::Error> for Error {
    fn from(err: tls::Error) -> Self {
        log::error!("TLS Error: {:?}", err);
        Self.TlsError
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
    let acceptor = tls::from_cert_and_key()?;
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
    acceptor: impl tls::Acceptor,
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
