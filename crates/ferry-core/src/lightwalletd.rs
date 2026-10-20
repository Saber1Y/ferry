//! Connections to lightwalletd servers.
//!
//! ZIP 318 keeps synchronization and broadcast unlinkable, so FERRY always holds two server
//! addresses: one it syncs from and a different one it broadcasts through.

use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;

use crate::error::Error;

/// A lightwalletd server address.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

impl Server {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Server { host: host.into(), port }
    }

    /// Parses `host:port`.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let (host, port) = s
            .rsplit_once(':')
            .ok_or_else(|| Error::InvalidServer(s.to_owned()))?;
        let port = port.parse().map_err(|_| Error::InvalidServer(s.to_owned()))?;
        if host.is_empty() {
            return Err(Error::InvalidServer(s.to_owned()));
        }
        Ok(Server::new(host, port))
    }

    /// Local servers (a regtest chain on this machine) have no TLS certificate. Every other
    /// server must use TLS.
    pub fn is_local(&self) -> bool {
        matches!(self.host.as_str(), "localhost" | "127.0.0.1" | "::1")
    }

    pub fn uri(&self) -> String {
        let scheme = if self.is_local() { "http" } else { "https" };
        format!("{scheme}://{}:{}", self.host, self.port)
    }

    /// Opens a gRPC connection to this server.
    pub async fn connect(&self) -> Result<CompactTxStreamerClient<Channel>, Error> {
        let mut endpoint = Endpoint::from_shared(self.uri())
            .map_err(|_| Error::InvalidServer(self.uri()))?;
        if !self.is_local() {
            endpoint = endpoint.tls_config(ClientTlsConfig::new().with_webpki_roots())?;
        }
        let channel = endpoint.connect().await?;
        Ok(CompactTxStreamerClient::new(channel))
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_and_port() {
        let s = Server::parse("127.0.0.1:9067").unwrap();
        assert_eq!(s, Server::new("127.0.0.1", 9067));
        assert!(s.is_local());
        assert_eq!(s.uri(), "http://127.0.0.1:9067");
    }

    #[test]
    fn remote_servers_use_tls() {
        let s = Server::parse("zec.rocks:443").unwrap();
        assert!(!s.is_local());
        assert_eq!(s.uri(), "https://zec.rocks:443");
    }

    #[test]
    fn rejects_malformed_addresses() {
        assert!(Server::parse("zec.rocks").is_err());
        assert!(Server::parse(":443").is_err());
        assert!(Server::parse("zec.rocks:http").is_err());
    }
}
