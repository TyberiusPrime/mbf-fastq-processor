//! LSP backend implementation

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::lsp::completion::CompletionProvider;
use crate::lsp::diagnostics::DiagnosticsProvider;
use crate::lsp::hover::HoverProvider;

/// The LSP backend for mbf-fastq-processor
pub struct Backend {
    client: Client,
    /// Maps document URI to document content
    documents: DashMap<String, String>,
    completion_provider: CompletionProvider,
    diagnostics_provider: DiagnosticsProvider,
    hover_provider: HoverProvider,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            completion_provider: CompletionProvider::new(),
            diagnostics_provider: DiagnosticsProvider::new(),
            hover_provider: HoverProvider::new(),
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: &Url, text: &str) {
        let diagnostics = self.diagnostics_provider.get_diagnostics(text);
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        log::info!("Initializing mbf-fastq-processor language server");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "[".to_string(),
                        "=".to_string(),
                        " ".to_string(),
                        "\"".to_string(),
                        "'".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "mbf-fastq-processor-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        log::info!("Language server initialized");
        self.client
            .log_message(MessageType::INFO, "mbf-fastq-processor LSP server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down language server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::info!("Document opened: {}", params.text_document.uri);
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;

        self.documents.insert(uri.clone(), text.clone());
        self.publish_diagnostics(&params.text_document.uri, &text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::info!("Document changed: {}", params.text_document.uri);
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();
            self.documents.insert(uri.clone(), text.clone());
            self.publish_diagnostics(&params.text_document.uri, &text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        log::info!("Document closed: {}", params.text_document.uri);
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        log::info!(
            "Completion requested at {}:{}:{}",
            params.text_document_position.text_document.uri,
            params.text_document_position.position.line,
            params.text_document_position.position.character
        );

        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(&uri) {
            let completions = self
                .completion_provider
                .get_completions(doc.value(), position);
            Ok(Some(CompletionResponse::Array(completions)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        log::info!(
            "Hover requested at {}:{}:{}",
            params.text_document_position_params.text_document.uri,
            params.text_document_position_params.position.line,
            params.text_document_position_params.position.character
        );

        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri) {
            Ok(self.hover_provider.get_hover(doc.value(), position))
        } else {
            Ok(None)
        }
    }
}
