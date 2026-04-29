use crate::*;

use rmcp::{
    RoleClient,
    model::ClientInfo,
    service::{RunningService, Service, ServiceError, serve_client},
    transport::{
        IntoTransport, TokioChildProcess,
        streamable_http_client::{
            StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
        },
    },
};
use std::borrow::Cow;

/// MCP Client for connecting to MCP servers via Streamable HTTP
pub(crate) struct McpClient {
    /// The peer connection to the MCP server
    service: RunningService<RoleClient, SimpleClientService>,
}

impl McpClient {
    /// Connect to an MCP server via Streamable HTTP
    ///
    /// # Arguments
    /// * `url` - The URL of the MCP server
    ///
    /// # Returns
    /// A connected MCP client
    pub(crate) async fn connect(url: &str) -> anyhow::Result<Self> {
        let config = StreamableHttpClientTransportConfig::with_uri(url);
        let transport = StreamableHttpClientTransport::from_config(config);
        Self::new(transport).await
    }

    pub(crate) async fn connect_unix_socket(path: &str) -> anyhow::Result<Self> {
        let transport = StreamableHttpClientTransport::from_unix_socket(path, "http://localhost/");
        Self::new(transport).await
    }

    pub(crate) async fn stdio(exe: &str) -> anyhow::Result<Self> {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.args(["-c", exe]);
        let transport = TokioChildProcess::new(cmd)?;
        Self::new(transport).await
    }

    /// Connect to an MCP server with custom configuration
    ///
    /// # Arguments
    /// * `config` - The transport configuration
    ///
    /// # Returns
    /// A connected MCP client
    async fn new<T, E, A>(transport: T) -> anyhow::Result<Self>
    where
        T: IntoTransport<RoleClient, E, A> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let client_info = ClientInfo::default();
        let service = SimpleClientService { info: client_info };

        let service = serve_client(service, transport).await?;

        Ok(Self { service })
    }

    /// List all tools, handling pagination automatically
    ///
    /// # Returns
    /// All tools from the server
    pub(crate) async fn list_all_tools(&self) -> Result<Vec<rmcp::model::Tool>, ServiceError> {
        self.service.peer().list_all_tools().await
    }

    /// Call a tool on the MCP server
    ///
    /// # Arguments
    /// * `name` - The name of the tool to call
    /// * `arguments` - The arguments to pass to the tool (as a JSON object)
    ///
    /// # Returns
    /// The result of the tool call
    pub(crate) async fn call_tool(
        &self,
        name: impl Into<Cow<'static, str>>,
        arguments: Option<JsonValue>,
    ) -> Result<rmcp::model::CallToolResult, ServiceError> {
        let arguments_map: Option<serde_json::Map<String, JsonValue>> =
            arguments.and_then(|v| serde_json::from_value(v).ok());

        let mut params = rmcp::model::CallToolRequestParams::new(name);
        params.arguments = arguments_map;

        self.service.peer().call_tool(params).await
    }

    pub(crate) async fn close(&mut self) -> anyhow::Result<()> {
        self.service.close().await?;
        Ok(())
    }
}

/// A simple client service implementation
#[derive(Debug, Clone)]
struct SimpleClientService {
    info: ClientInfo,
}

impl Service<RoleClient> for SimpleClientService {
    async fn handle_request(
        &self,
        _request: rmcp::model::ServerRequest,
        _context: rmcp::service::RequestContext<RoleClient>,
    ) -> Result<rmcp::model::ClientResult, rmcp::ErrorData> {
        // We don't expect any requests from the server for a simple client
        Err(rmcp::ErrorData::new(
            rmcp::model::ErrorCode::METHOD_NOT_FOUND,
            "Method not found",
            None,
        ))
    }

    async fn handle_notification(
        &self,
        _notification: rmcp::model::ServerNotification,
        _context: rmcp::service::NotificationContext<RoleClient>,
    ) -> Result<(), rmcp::ErrorData> {
        // Ignore notifications for now
        Ok(())
    }

    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }
}
