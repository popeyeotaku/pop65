use std::{collections::HashMap, fs};

use tokio::{
    io::{stdin, stdout},
    sync::Mutex,
};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability,
        InitializeParams, InitializeResult, Location, OneOf, PositionEncodingKind, ReferenceParams,
        RenameParams, ServerCapabilities, SymbolInformation, SymbolKind,
        TextDocumentSyncCapability, TextDocumentSyncKind, Url, WorkspaceEdit,
        WorkspaceSymbolParams,
    },
    Client, LanguageServer, LspService, Server,
};

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

/// A symbol on the LSP side.
struct Symbol {
    pub name: String,
    pub defined_at: Location,
    pub references: Vec<Location>,
}

struct Backend {
    _client: Client,
    client_sources: Mutex<HashMap<Url, (i32, String)>>,
    symtab: Mutex<Option<HashMap<String, Symbol>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            _client: client,
            client_sources: Mutex::new(HashMap::new()),
            symtab: Mutex::new(None),
        }
    }

    /// Refresh the internal symbol table.
    fn refresh(&self) {
        todo!()
    }

    /// Return the text of a file.
    async fn get_file(&self, uri: Url) -> Option<String> {
        if let Some((_, text)) = self.client_sources.blocking_lock().get(&uri) {
            Some(text.clone())
        } else {
            let path = uri.to_file_path().ok()?;
            let text = fs::read_to_string(path).ok()?;
            Some(text)
        }
    }

    /// Find the symbol corresponding to the location.
    async fn find_symbol<F, T>(&self, reference: &Location, f: F) -> Option<T>
    where
        F: FnOnce(&Symbol) -> T,
    {
        self.refresh();
        for symbol in self.symtab.blocking_lock().as_ref()?.values() {
            if symbol.references.contains(reference) {
                return Some(f(symbol));
            }
        }
        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF8),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) -> () {
        let uri = &params.text_document.uri;
        let text = &params.text_document.text;
        let already_there = self
            .client_sources
            .blocking_lock()
            .insert(uri.clone(), (i32::MIN, text.clone()));
        if already_there.is_some() {
            panic!("already opened {}", uri.as_str());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) -> () {
        let uri = &params.text_document.uri;
        let was_there = self.client_sources.blocking_lock().remove(uri);
        if was_there.is_none() {
            panic!("closing {}, which was never opened", uri.as_str());
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) -> () {
        let uri = &params.text_document.uri;
        let version = params.text_document.version;
        if let Some(current) = self.client_sources.blocking_lock().get_mut(uri) {
            if current.0 < version {
                for change in params.content_changes {
                    assert!(change.range.is_none());
                    *current = (version, change.text.clone());
                }
            }
        } else {
            panic!("changing {}, which was not opened", uri.as_str())
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        todo!()
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.refresh();
        todo!()
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        self.refresh();
        if let Some(symtab) = self.symtab.blocking_lock().as_ref() {
            let mut symbols = Vec::new();
            for symbol in symtab.values() {
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: symbol.name.clone(),
                    kind: SymbolKind::VARIABLE,
                    location: symbol.defined_at.clone(),
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Ok(Some(symbols))
        } else {
            Ok(None)
        }
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        todo!()
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        todo!()
    }
}
