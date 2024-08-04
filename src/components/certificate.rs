use anyhow::Context as _;
use openssl::{
    hash::{DigestBytes, MessageDigest},
    x509::X509,
};
use windmark::context::RouteContext;

use crate::i18n::Lang;

pub const CERT_HASH_LEN: usize = 64;
pub type CertHash = Box<DigestBytes>;

pub fn hash_certificate(cert: &X509) -> anyhow::Result<CertHash> {
    cert.digest(MessageDigest::sha512())
        .context("failed to hash certificate")
        .map(Box::new)
}

/// Error with a certificate required response if a certificate is not present
pub fn require_certificate(
    context: &RouteContext,
    lang: &'static Lang,
) -> Result<(), windmark::response::Response> {
    if context.certificate.is_none() {
        Err(windmark::response::Response::client_certificate_required(
            &lang.cert_required,
        ))?;
    }
    Ok(())
}
