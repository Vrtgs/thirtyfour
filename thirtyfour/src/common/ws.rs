use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream, tungstenite};

/// Open a WebSocket, verifying `wss://` peers against the bundled webpki roots.
pub(crate) async fn connect(
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let config =
        rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    let connector = Connector::Rustls(Arc::new(config));
    let (ws, _resp) =
        tokio_tungstenite::connect_async_tls_with_config(url, None, false, Some(connector)).await?;
    Ok(ws)
}
