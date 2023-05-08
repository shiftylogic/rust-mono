/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_web
 * Module:  tls
 *
 * Purpose:
 *   Wraps TLS on a TCP socket. Can switch between different TLS
 *   implementations using compile features.
 *
 *   Supported:
 *      Platform Native (via native-tls)
 *          -> SChannel / Crypto API (Windows)
 *          -> Secure Transport (macos)
 *          -> OpenSSL (Linux)
 *      Rust Native (via rustls)
 *
 */

// The TLS features are mutually exclusive. This fails to compile rather than
// selecting a default and not being aware that the other was not being used.
#[cfg(all(feature = "rustls", feature = "nativetls"))]
compile_error!("features \"rustls\" and \"nativetls\" should not be enabled together");

#[cfg(feature = "rustls")]
mod rustls;


#[derive(Debug)]
pub enum Error {
    CertificateLoadError,
    PrivateKeyLoadError,
    TlsConfigError,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Acceptor {
    type Stream;
    type Future: std::future::Future<Output = std::io::Result<Self::Stream>>;

    fn accept(&self, stream: Self::Stream) -> Self::Future;
}


#[cfg(feature = "rustls")]
pub use rustls::from_cert_and_key;
