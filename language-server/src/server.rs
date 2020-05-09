use crate::{
    client::{Client, ResponseHandler},
    jsonrpc::*,
    LanguageClient,
};
use async_trait::async_trait;
use language_server_macros::*;
use lsp_types::*;
use serde_json::json;

/// Defines the server-side implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
///
/// Empty default implementations are provided for convenience.
#[allow(unused_variables)]
#[jsonrpc_client(ident = "TestLanguageClient", keep_trait = true)]
#[jsonrpc_server]
#[async_trait]
pub trait LanguageServer {
    /// The [`initialize`](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#initialize)
    /// request is sent as the first request from the client to the server.
    #[jsonrpc_method(name = "initialize", kind = "request")]
    async fn initialize(
        &self,
        params: InitializedParams,
        client: LanguageClient,
    ) -> Result<InitializeResult>;

    /// The [`initialized`](https://microsoft.github.io/language-server-protocol/specification#initialized)
    /// notification is sent from the client to the server after the client received the result of the `initialize`
    /// request but before the client is sending any other request or notification to the server.
    #[jsonrpc_method(name = "initialized", kind = "notification")]
    async fn initialized(&self, params: InitializedParams, client: LanguageClient) {}

    /// The [`shutdown`](https://microsoft.github.io/language-server-protocol/specification#shutdown)
    /// request is sent from the client to the server. It asks the server to shut down,
    /// but to not exit (otherwise the response might not be delivered correctly to the client).
    #[jsonrpc_method(name = "shutdown", kind = "request")]
    async fn shutdown(&self, params: (), client: LanguageClient) -> Result<()> {
        Ok(())
    }

    /// A [notification](https://microsoft.github.io/language-server-protocol/specification#exit) to ask the server to exit its process.
    /// The server should exit with success code 0 if the shutdown request has been received before; otherwise with error code 1.
    #[jsonrpc_method(name = "exit", kind = "notification")]
    async fn exit(&self, params: (), client: LanguageClient) {}

    /// The [`window/workDoneProgress/cancel`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_cancel)
    /// notification is sent from the client to the server to cancel a progress initiated on the server side using the
    /// [`window/workDoneProgress/create`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_create).
    #[jsonrpc_method(name = "window/workDoneProgress/cancel", kind = "notification")]
    async fn work_done_progress_cancel(
        &self,
        params: WorkDoneProgressCancelParams,
        client: LanguageClient,
    ) {
    }

    /// The [`workspace/didChangeWorkspaceFolders`](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWorkspaceFolders)
    /// notification is sent from the client to the server to inform the server about workspace folder configuration changes.
    #[jsonrpc_method(name = "workspace/didChangeWorkspaceFolders", kind = "notification")]
    async fn did_change_workspace_folders(
        &self,
        params: DidChangeWorkspaceFoldersParams,
        client: LanguageClient,
    ) {
    }

    /// A [notification](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)
    /// sent from the client to the server to signal the change of configuration settings.
    #[jsonrpc_method(name = "workspace/didChangeConfiguration", kind = "notification")]
    async fn did_change_configuration(
        &self,
        params: DidChangeConfigurationParams,
        client: LanguageClient,
    ) {
    }

    /// The [watched files notification](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)
    /// is sent from the client to the server when the client detects changes to files watched by the language client.
    #[jsonrpc_method(name = "workspace/didChangeWatchedFiles", kind = "notification")]
    async fn did_change_watched_files(
        &self,
        params: DidChangeWatchedFilesParams,
        client: LanguageClient,
    ) {
    }

    /// The [workspace symbol request](https://microsoft.github.io/language-server-protocol/specification#workspace_symbol)
    /// is sent from the client to the server to list project-wide symbols matching the query string.
    #[jsonrpc_method(name = "workspace/symbol", kind = "request")]
    async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
        client: LanguageClient,
    ) -> Result<Vec<SymbolInformation>> {
        Ok(Vec::new())
    }

    /// The [`workspace/executeCommand`](https://microsoft.github.io/language-server-protocol/specification#workspace_executeCommand)
    /// request is sent from the client to the server to trigger command execution on the server.
    #[jsonrpc_method(name = "workspace/executeCommand", kind = "request")]
    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
        client: LanguageClient,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    /// The [document open notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)
    /// is sent from the client to the server to signal newly opened text documents.
    #[jsonrpc_method(name = "textDocument/didOpen", kind = "notification")]
    async fn did_open(&self, params: DidOpenTextDocumentParams, client: LanguageClient) {}

    /// The [document change notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)
    /// is sent from the client to the server to signal changes to a text document.
    #[jsonrpc_method(name = "textDocument/didChange", kind = "notification")]
    async fn did_change(&self, params: DidChangeTextDocumentParams, client: LanguageClient) {}

    /// The [document will save notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_willSave)
    /// is sent from the client to the server before the document is actually saved.
    #[jsonrpc_method(name = "textDocument/willSave", kind = "notification")]
    async fn will_save(&self, params: WillSaveTextDocumentParams, client: LanguageClient) {}

    /// The [document will save request](https://microsoft.github.io/language-server-protocol/specification#textDocument_willSaveWaitUntil)
    /// is sent from the client to the server before the document is actually saved.
    #[jsonrpc_method(name = "textDocument/willSaveWaitUntil", kind = "request")]
    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
        client: LanguageClient,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document save notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)
    /// is sent from the client to the server when the document was saved in the client.
    #[jsonrpc_method(name = "textDocument/didSave", kind = "notification")]
    async fn did_save(&self, params: DidSaveTextDocumentParams, client: LanguageClient) {}

    /// The [document close notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)
    /// is sent from the client to the server when the document got closed in the client.
    #[jsonrpc_method(name = "textDocument/didClose", kind = "notification")]
    async fn did_close(&self, params: DidCloseTextDocumentParams, client: LanguageClient) {}

    /// The [Completion request](https://microsoft.github.io/language-server-protocol/specification#textDocument_completion)
    /// is sent from the client to the server to compute completion items at a given cursor position.
    #[jsonrpc_method(name = "textDocument/completion", kind = "request")]
    async fn completion(
        &self,
        params: CompletionParams,
        client: LanguageClient,
    ) -> Result<CompletionResponse> {
        Ok(CompletionResponse::Array(Vec::new()))
    }

    /// The [request](https://microsoft.github.io/language-server-protocol/specification#completionItem_resolve)
    /// is sent from the client to the server to resolve additional information for a given completion item.
    #[jsonrpc_method(name = "completionItem/resolve", kind = "request")]
    async fn completion_resolve(
        &self,
        item: CompletionItem,
        client: LanguageClient,
    ) -> Result<CompletionItem> {
        Ok(item)
    }

    /// The [hover request](https://microsoft.github.io/language-server-protocol/specification#textDocument_hover)
    /// is sent from the client to the server to request hover information at a given text document position.
    #[jsonrpc_method(name = "textDocument/hover", kind = "request")]
    async fn hover(
        &self,
        params: TextDocumentPositionParams,
        client: LanguageClient,
    ) -> Result<Option<Hover>> {
        Ok(None)
    }

    /// The [signature help request](https://microsoft.github.io/language-server-protocol/specification#textDocument_signatureHelp)
    /// is sent from the client to the server to request signature information at a given cursor position.
    #[jsonrpc_method(name = "textDocument/signatureHelp", kind = "request")]
    async fn signature_help(
        &self,
        params: SignatureHelpParams,
        client: LanguageClient,
    ) -> Result<Option<SignatureHelp>> {
        Ok(None)
    }

    /// The [go to declaration](https://microsoft.github.io/language-server-protocol/specification#textDocument_declaration)
    /// request is sent from the client to the server to resolve the declaration location of a symbol at a given text document position.
    #[jsonrpc_method(name = "textDocument/declaration", kind = "request")]
    async fn declaration(
        &self,
        params: GotoDefinitionParams,
        client: LanguageClient,
    ) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to definition request](https://microsoft.github.io/language-server-protocol/specification#textDocument_definition)
    /// is sent from the client to the server to resolve the definition location of a symbol at a given text document position.
    #[jsonrpc_method(name = "textDocument/definition", kind = "request")]
    async fn definition(
        &self,
        params: GotoDefinitionParams,
        client: LanguageClient,
    ) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to type definition request](https://microsoft.github.io/language-server-protocol/specification#textDocument_typeDefinition)
    /// is sent from the client to the server to resolve the type definition location of a symbol at a given text document position.
    #[jsonrpc_method(name = "textDocument/typeDefinition", kind = "request")]
    async fn type_definition(
        &self,
        params: GotoDefinitionParams,
        client: LanguageClient,
    ) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to implementation request](https://microsoft.github.io/language-server-protocol/specification#textDocument_implementation)
    /// is sent from the client to the server to resolve the implementation location of a symbol at a given text document position.
    #[jsonrpc_method(name = "textDocument/implementation", kind = "request")]
    async fn implementation(
        &self,
        params: GotoDefinitionParams,
        client: LanguageClient,
    ) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [references request](https://microsoft.github.io/language-server-protocol/specification#textDocument_references)
    /// is sent from the client to the server to resolve project-wide references for the symbol denoted by the given text document position.
    #[jsonrpc_method(name = "textDocument/references", kind = "request")]
    async fn references(
        &self,
        params: ReferenceParams,
        client: LanguageClient,
    ) -> Result<Vec<Location>> {
        Ok(Vec::new())
    }

    /// The [document highlight request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentHighlight)
    /// is sent from the client to the server to resolve a document highlights for a given text document position.
    #[jsonrpc_method(name = "textDocument/documentHighlight", kind = "request")]
    async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
        client: LanguageClient,
    ) -> Result<Vec<DocumentHighlight>> {
        Ok(Vec::new())
    }

    /// The [document symbol request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentSymbol)
    /// is sent from the client to the server.
    #[jsonrpc_method(name = "textDocument/documentSymbol", kind = "request")]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
        client: LanguageClient,
    ) -> Result<DocumentSymbolResponse> {
        Ok(DocumentSymbolResponse::Flat(Vec::new()))
    }

    /// The [code action request](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)
    /// is sent from the client to the server to compute commands for a given text document and range.
    #[jsonrpc_method(name = "textDocument/codeAction", kind = "request")]
    async fn code_action(
        &self,
        params: CodeActionParams,
        client: LanguageClient,
    ) -> Result<CodeActionResponse> {
        Ok(CodeActionResponse::new())
    }

    /// The [code lens request](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeLens)
    /// is sent from the client to the server to compute code lenses for a given text document.
    #[jsonrpc_method(name = "textDocument/codeLens", kind = "request")]
    async fn code_lens(
        &self,
        params: CodeLensParams,
        client: LanguageClient,
    ) -> Result<Vec<CodeLens>> {
        Ok(Vec::new())
    }

    /// The [code lens resolve request](https://microsoft.github.io/language-server-protocol/specification#codeLens_resolve)
    /// is sent from the client to the server to resolve the command for a given code lens item.
    #[jsonrpc_method(name = "codeLens/resolve", kind = "request")]
    async fn code_lens_resolve(&self, item: CodeLens, client: LanguageClient) -> Result<CodeLens> {
        Ok(item)
    }

    /// The [document links request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentLink)
    /// is sent from the client to the server to request the location of links in a document.
    #[jsonrpc_method(name = "textDocument/documentLink", kind = "request")]
    async fn document_link(
        &self,
        params: DocumentLinkParams,
        client: LanguageClient,
    ) -> Result<Vec<DocumentLink>> {
        Ok(Vec::new())
    }

    /// The [document link resolve request](https://microsoft.github.io/language-server-protocol/specification#documentLink_resolve)
    /// is sent from the client to the server to resolve the target of a given document link.
    #[jsonrpc_method(name = "documentLink/resolve", kind = "request")]
    async fn document_link_resolve(
        &self,
        item: DocumentLink,
        client: LanguageClient,
    ) -> Result<DocumentLink> {
        Ok(item)
    }

    /// The [document color request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentColor)
    /// is sent from the client to the server to list all color references found in a given text document.
    #[jsonrpc_method(name = "textDocument/documentColor", kind = "request")]
    async fn document_color(
        &self,
        params: DocumentColorParams,
        client: LanguageClient,
    ) -> Result<Vec<ColorInformation>> {
        Ok(Vec::new())
    }

    /// The [color presentation request](https://microsoft.github.io/language-server-protocol/specification#textDocument_colorPresentation)
    /// is sent from the client to the server to obtain a list of presentations for a color value at a given location.
    #[jsonrpc_method(name = "textDocument/colorPresentation", kind = "request")]
    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
        client: LanguageClient,
    ) -> Result<Vec<ColorPresentation>> {
        Ok(Vec::new())
    }

    /// The [document formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_formatting)
    /// is sent from the client to the server to format a whole document.
    #[jsonrpc_method(name = "textDocument/formatting", kind = "request")]
    async fn formatting(
        &self,
        params: DocumentFormattingParams,
        client: LanguageClient,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document range formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_rangeFormatting)
    /// is sent from the client to the server to format a given range in a document.
    #[jsonrpc_method(name = "textDocument/rangeFormatting", kind = "request")]
    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
        client: LanguageClient,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document on type formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_onTypeFormatting)
    /// is sent from the client to the server to format parts of the document during typing.
    #[jsonrpc_method(name = "textDocument/onTypeFormatting", kind = "request")]
    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
        client: LanguageClient,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [rename request](https://microsoft.github.io/language-server-protocol/specification#textDocument_rename)
    /// is sent from the client to the server to ask the server to compute a workspace change so that the client
    /// can perform a workspace-wide rename of a symbol.
    #[jsonrpc_method(name = "textDocument/rename", kind = "request")]
    async fn rename(
        &self,
        params: RenameParams,
        client: LanguageClient,
    ) -> Result<Option<WorkspaceEdit>> {
        Ok(None)
    }

    /// The [prepare rename request](https://microsoft.github.io/language-server-protocol/specification#textDocument_prepareRename)
    /// is sent from the client to the server to setup and test the validity of a rename operation at a given location.
    #[jsonrpc_method(name = "textDocument/prepareRename", kind = "request")]
    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
        client: LanguageClient,
    ) -> Result<Option<PrepareRenameResponse>> {
        Ok(None)
    }

    /// The [folding range request](https://microsoft.github.io/language-server-protocol/specification#textDocument_foldingRange)
    /// is sent from the client to the server to return all folding ranges found in a given text document.
    #[jsonrpc_method(name = "textDocument/foldingRange", kind = "request")]
    async fn folding_range(
        &self,
        params: FoldingRangeParams,
        client: LanguageClient,
    ) -> Result<Vec<FoldingRange>> {
        Ok(Vec::new())
    }

    /// The [selection range request](https://microsoft.github.io/language-server-protocol/specification#textDocument_selectionRange)
    /// is sent from the client to the server to return suggested selection ranges at an array of given positions.
    #[jsonrpc_method(name = "textDocument/selectionRange", kind = "request")]
    async fn selection_range(
        &self,
        params: SelectionRangeParams,
        client: LanguageClient,
    ) -> Result<Vec<SelectionRange>> {
        Ok(Vec::new())
    }
}

#[async_trait]
pub trait RequestHandler {
    async fn handle_request(&self, request: Request, client: LanguageClient) -> Response;

    async fn handle_notification(&self, notification: Notification, client: LanguageClient);
}
