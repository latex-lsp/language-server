use async_trait::async_trait;
use lsp_types::*;

pub type Result<T> = std::result::Result<T, String>;

/// Defines the server-side implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
///
/// Empty default implementations are provided for convenience.
#[allow(unused_variables)]
#[async_trait]
pub trait LanguageServer {
    /// The [`initialize`](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#initialize)
    /// request is sent as the first request from the client to the server.
    async fn initialize(&self, params: InitializedParams) -> Result<InitializeResult>;

    /// The [`initialized`](https://microsoft.github.io/language-server-protocol/specification#initialized)
    /// notification is sent from the client to the server after the client received the result of the `initialize`
    /// request but before the client is sending any other request or notification to the server.
    async fn initialized(&self, params: InitializedParams) {}

    /// The [`shutdown`](https://microsoft.github.io/language-server-protocol/specification#shutdown)
    /// request is sent from the client to the server. It asks the server to shut down,
    /// but to not exit (otherwise the response might not be delivered correctly to the client).
    async fn shutdown(&self, params: ()) -> Result<()> {
        Ok(())
    }

    /// A [notification](https://microsoft.github.io/language-server-protocol/specification#exit) to ask the server to exit its process.
    /// The server should exit with success code 0 if the shutdown request has been received before; otherwise with error code 1.
    async fn exit(&self, params: ()) {}

    /// The [`window/workDoneProgress/cancel`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_cancel)
    /// notification is sent from the client to the server to cancel a progress initiated on the server side using the
    /// [`window/workDoneProgress/create`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_create).
    async fn work_done_progress_cancel(&self, params: WorkDoneProgressCancelParams) {}

    /// The [`workspace/didChangeWorkspaceFolders`](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWorkspaceFolders)
    /// notification is sent from the client to the server to inform the server about workspace folder configuration changes.
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {}

    /// A [notification](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)
    /// sent from the client to the server to signal the change of configuration settings.
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {}

    /// The [watched files notification](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)
    /// is sent from the client to the server when the client detects changes to files watched by the language client.
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    /// The [workspace symbol request](https://microsoft.github.io/language-server-protocol/specification#workspace_symbol)
    /// is sent from the client to the server to list project-wide symbols matching the query string.
    async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>> {
        Ok(Vec::new())
    }

    /// The [`workspace/executeCommand`](https://microsoft.github.io/language-server-protocol/specification#workspace_executeCommand)
    /// request is sent from the client to the server to trigger command execution on the server.
    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    /// The [document open notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)
    /// is sent from the client to the server to signal newly opened text documents.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {}

    /// The [document change notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)
    /// is sent from the client to the server to signal changes to a text document.
    async fn did_change(&self, params: DidChangeTextDocumentParams) {}

    /// The [document will save notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_willSave)
    /// is sent from the client to the server before the document is actually saved.
    async fn will_save(&self, params: WillSaveTextDocumentParams) {}

    /// The [document will save request](https://microsoft.github.io/language-server-protocol/specification#textDocument_willSaveWaitUntil)
    /// is sent from the client to the server before the document is actually saved.
    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document save notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)
    /// is sent from the client to the server when the document was saved in the client.
    async fn did_save(&self, params: DidSaveTextDocumentParams) {}

    /// The [document close notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)
    /// is sent from the client to the server when the document got closed in the client.
    async fn did_close(&self, params: DidCloseTextDocumentParams) {}

    /// The [Completion request](https://microsoft.github.io/language-server-protocol/specification#textDocument_completion)
    /// is sent from the client to the server to compute completion items at a given cursor position.
    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        Ok(CompletionResponse::Array(Vec::new()))
    }

    /// The [request](https://microsoft.github.io/language-server-protocol/specification#completionItem_resolve)
    /// is sent from the client to the server to resolve additional information for a given completion item.
    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    /// The [hover request](https://microsoft.github.io/language-server-protocol/specification#textDocument_hover)
    /// is sent from the client to the server to request hover information at a given text document position.
    async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    /// The [signature help request](https://microsoft.github.io/language-server-protocol/specification#textDocument_signatureHelp)
    /// is sent from the client to the server to request signature information at a given cursor position.
    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        Ok(None)
    }

    /// The [go to declaration](https://microsoft.github.io/language-server-protocol/specification#textDocument_declaration)
    /// request is sent from the client to the server to resolve the declaration location of a symbol at a given text document position.
    async fn declaration(&self, params: GotoDefinitionParams) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to definition request](https://microsoft.github.io/language-server-protocol/specification#textDocument_definition)
    /// is sent from the client to the server to resolve the definition location of a symbol at a given text document position.
    async fn definition(&self, params: GotoDefinitionParams) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to type definition request](https://microsoft.github.io/language-server-protocol/specification#textDocument_typeDefinition)
    /// is sent from the client to the server to resolve the type definition location of a symbol at a given text document position.
    async fn type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [go to implementation request](https://microsoft.github.io/language-server-protocol/specification#textDocument_implementation)
    /// is sent from the client to the server to resolve the implementation location of a symbol at a given text document position.
    async fn implementation(&self, params: GotoDefinitionParams) -> Result<GotoDefinitionResponse> {
        Ok(GotoDefinitionResponse::Array(Vec::new()))
    }

    /// The [references request](https://microsoft.github.io/language-server-protocol/specification#textDocument_references)
    /// is sent from the client to the server to resolve project-wide references for the symbol denoted by the given text document position.
    async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        Ok(Vec::new())
    }

    /// The [document highlight request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentHighlight)
    /// is sent from the client to the server to resolve a document highlights for a given text document position.
    async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        Ok(Vec::new())
    }

    /// The [document symbol request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentSymbol)
    /// is sent from the client to the server.
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<DocumentSymbolResponse> {
        Ok(DocumentSymbolResponse::Flat(Vec::new()))
    }

    /// The [code action request](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)
    /// is sent from the client to the server to compute commands for a given text document and range.
    async fn code_action(&self, params: CodeActionParams) -> Result<CodeActionResponse> {
        Ok(CodeActionResponse::new())
    }

    /// The [code lens request](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeLens)
    /// is sent from the client to the server to compute code lenses for a given text document.
    async fn code_lens(&self, params: CodeLensParams) -> Result<Vec<CodeLens>> {
        Ok(Vec::new())
    }

    /// The [code lens resolve request](https://microsoft.github.io/language-server-protocol/specification#codeLens_resolve)
    /// is sent from the client to the server to resolve the command for a given code lens item.
    async fn code_lens_resolve(&self, item: CodeLens) -> Result<CodeLens> {
        Ok(item)
    }

    /// The [document links request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentLink)
    /// is sent from the client to the server to request the location of links in a document.
    async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        Ok(Vec::new())
    }

    /// The [document link resolve request](https://microsoft.github.io/language-server-protocol/specification#documentLink_resolve)
    /// is sent from the client to the server to resolve the target of a given document link.
    async fn document_link_resolve(&self, item: DocumentLink) -> Result<DocumentLink> {
        Ok(item)
    }

    /// The [document color request](https://microsoft.github.io/language-server-protocol/specification#textDocument_documentColor)
    /// is sent from the client to the server to list all color references found in a given text document.
    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        Ok(Vec::new())
    }

    /// The [color presentation request](https://microsoft.github.io/language-server-protocol/specification#textDocument_colorPresentation)
    /// is sent from the client to the server to obtain a list of presentations for a color value at a given location.
    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        Ok(Vec::new())
    }

    /// The [document formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_formatting)
    /// is sent from the client to the server to format a whole document.
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document range formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_rangeFormatting)
    /// is sent from the client to the server to format a given range in a document.
    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [document on type formatting request](https://microsoft.github.io/language-server-protocol/specification#textDocument_onTypeFormatting)
    /// is sent from the client to the server to format parts of the document during typing.
    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    /// The [rename request](https://microsoft.github.io/language-server-protocol/specification#textDocument_rename)
    /// is sent from the client to the server to ask the server to compute a workspace change so that the client
    /// can perform a workspace-wide rename of a symbol.
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(None)
    }

    /// The [prepare rename request](https://microsoft.github.io/language-server-protocol/specification#textDocument_prepareRename)
    /// is sent from the client to the server to setup and test the validity of a rename operation at a given location.
    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        Ok(None)
    }

    /// The [folding range request](https://microsoft.github.io/language-server-protocol/specification#textDocument_foldingRange)
    /// is sent from the client to the server to return all folding ranges found in a given text document.
    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        Ok(Vec::new())
    }

    /// The [selection range request](https://microsoft.github.io/language-server-protocol/specification#textDocument_selectionRange)
    /// is sent from the client to the server to return suggested selection ranges at an array of given positions.
    async fn selection_range(&self, params: SelectionRangeParams) -> Result<Vec<SelectionRange>> {
        Ok(Vec::new())
    }
}
