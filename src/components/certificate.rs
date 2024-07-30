use anyhow::Context;
use openssl::{hash::MessageDigest, x509::X509};
use windmark::context::RouteContext;

pub type CertHash = Vec<u8>;

pub fn hash_certificate(cert: &X509) -> anyhow::Result<CertHash> {
    cert.digest(MessageDigest::sha512())
        .context("failed to hash certificate")
        .map(|digest_bytes| digest_bytes.iter().copied().collect())
}

pub fn require_certificate(context: &RouteContext) -> Result<(), windmark::response::Response> {
    if context.certificate.is_none() {
        Err(
            windmark::response::Response::client_certificate_required(
            "A certificate is required to maintain state. \
                Please create or choose a certificate, or switch to a Gemini client that supports certificates. \
                Proxies to HTTP or other protocols probably will not work.",
            )
        )?;
    }
    Ok(())
}
