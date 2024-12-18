use envconfig::Envconfig;
use http::{Method, StatusCode};
use integrationos_domain::{IntegrationOSError, InternalError};
use integrationos_emit::domain::config::EmitterConfig;
use integrationos_emit::server::Server;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::Debug;
use std::{collections::HashMap, sync::OnceLock, time::Duration};
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{clients::Cli as Docker, Container},
};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

static DOCKER: OnceLock<Docker> = OnceLock::new();
static MONGO: OnceLock<Container<'static, Mongo>> = OnceLock::new();
static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestServer {
    pub port: u16,
    pub client: reqwest::Client,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ApiResponse<T: DeserializeOwned = Value> {
    pub code: StatusCode,
    pub data: T,
}

impl TestServer {
    pub async fn new() -> Result<Self, IntegrationOSError> {
        TRACING.get_or_init(|| {
            let filter = EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();

            tracing_subscriber::fmt().with_env_filter(filter).init();
        });
        let docker = DOCKER.get_or_init(Default::default);
        let mongo = MONGO.get_or_init(|| docker.run(Mongo));
        let port = mongo.get_host_port_ipv4(27017);

        let database_uri = format!("mongodb://127.0.0.1:{port}/?directConnection=true");
        let database_name = Uuid::new_v4().to_string();

        let port = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to port")
            .local_addr()
            .expect("Failed to get local address")
            .port();

        let config = EmitterConfig::init_from_hashmap(&HashMap::from([
            (
                "INTERNAL_SERVER_ADDRESS".to_string(),
                format!("0.0.0.0:{port}"),
            ),
            ("CONTROL_DATABASE_URL".to_string(), database_uri.clone()),
            ("CONTROL_DATABASE_NAME".to_string(), database_name.clone()),
            ("CONTEXT_DATABASE_URL".to_string(), database_uri.clone()),
            ("CONTEXT_DATABASE_NAME".to_string(), database_name.clone()),
            ("EVENT_DATABASE_URL".to_string(), database_uri.clone()),
            ("EVENT_DATABASE_NAME".to_string(), database_name.clone()),
        ]))
        .expect("Failed to initialize storage config");

        let server = Server::init(config.clone())
            .await
            .expect("Failed to initialize storage");

        tokio::task::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        Ok(Self { port, client })
    }

    pub async fn send_request<T: Serialize, U: DeserializeOwned + Debug>(
        &self,
        path: &str,
        method: Method,
        payload: Option<&T>,
        header: Option<&HashMap<String, String>>,
    ) -> Result<ApiResponse<U>, IntegrationOSError> {
        let uri = format!("http://localhost:{}/{path}", self.port);
        println!("Sending request to {uri}");
        let mut req = self.client.request(method, uri);
        if let Some(payload) = payload {
            req = req.json(payload);
        }

        if let Some(header) = header {
            for (key, value) in header {
                req = req.header(key, value);
            }
        }

        let res = req.send().await.map_err(|e| {
            InternalError::io_err(&format!("Failed to send request: {:?}", e.source()), None)
        })?;

        let status = res.status();
        let json = res.json().await;

        Ok(ApiResponse {
            code: status,
            data: json.map_err(|e| {
                InternalError::deserialize_error(
                    &format!("Failed to deserialize response: {}", e),
                    None,
                )
            })?,
        })
    }
}
