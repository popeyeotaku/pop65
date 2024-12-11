use tokio::io::{stdin, stdout};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{InitializeParams, InitializeResult},
    Client, LanguageServer, LspService, Server,
};

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        todo!()
    }

    async fn shutdown(&self) -> Result<()> {
        todo!()
    }
}
