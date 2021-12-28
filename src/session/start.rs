use crate::common::command::{Command, FormatRequestData};
use crate::error::{WebDriverError, WebDriverResult};
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::session::handle::SessionHandle;
use crate::session::task::WebDriverSession;
use crate::{SessionId, TimeoutConfiguration};
use serde::Deserialize;
use tokio::sync::mpsc::unbounded_channel;

/// Start a new WebDriver session, returning the session id and the
/// capabilities JSON that was received back from the server.
pub async fn start_session(
    http_client: Box<dyn WebDriverHttpClientAsync>,
    capabilities: serde_json::Value,
) -> WebDriverResult<SessionHandle> {
    let v = match http_client
        .execute(Command::NewSession(capabilities.clone()).format_request(&SessionId::null()))
        .await
    {
        Ok(x) => Ok(x),
        Err(e) => {
            // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
            // Retry if we get a 500. If it happens twice in a row then the second error
            // will be returned.
            if let WebDriverError::UnknownError(x) = &e {
                if x.status == 500 {
                    http_client
                        .execute(
                            Command::NewSession(capabilities).format_request(&SessionId::null()),
                        )
                        .await
                } else {
                    Err(e)
                }
            } else {
                Err(e)
            }
        }
    }?;

    #[derive(Debug, Deserialize)]
    struct ConnectionData {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        #[serde(default)]
        capabilities: serde_json::Value,
    }

    #[derive(Debug, Deserialize)]
    struct ConnectionResp {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        value: ConnectionData,
    }

    let resp: ConnectionResp = serde_json::from_value(v)?;
    let data = resp.value;
    let session_id = SessionId::from(if resp.session_id.is_empty() {
        data.session_id
    } else {
        resp.session_id
    });

    // Set default timeouts.
    http_client
        .execute(Command::SetTimeouts(TimeoutConfiguration::default()).format_request(&session_id))
        .await?;

    let (tx, rx) = unbounded_channel();

    // Spawn async task to process the session.
    let session = WebDriverSession::new(session_id.clone(), http_client, data.capabilities);
    let config = session.config();
    crate::runtime::imports::spawn(session.run(rx));

    let handle = SessionHandle {
        tx,
        config,
    };
    Ok(handle)
}
