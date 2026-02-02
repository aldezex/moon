use std::collections::HashMap;
use std::path::PathBuf;

use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_core::span::Span;
use moon_typechecker::{check_program, check_program_with_spans};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone)]
struct Document {
    text: String,
    version: Option<i32>,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: RwLock<HashMap<Url, Document>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn upsert_document(&self, uri: Url, text: String, version: Option<i32>) {
        let mut docs = self.documents.write().await;
        docs.insert(uri, Document { text, version });
    }

    async fn get_document(&self, uri: &Url) -> Option<Document> {
        let docs = self.documents.read().await;
        docs.get(uri).cloned()
    }

    async fn publish_diagnostics(&self, uri: Url) {
        let Some(doc) = self.get_document(&uri).await else {
            return;
        };

        let diags = diagnostics_for(&uri, &doc.text);
        self.client
            .publish_diagnostics(uri, diags, doc.version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            definition_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                // The MVP server does not do context-sensitive completion yet.
                resolve_provider: Some(false),
                trigger_characters: None,
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "moon-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "moon-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = Some(params.text_document.version);
        self.upsert_document(uri.clone(), text, version).await;
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = Some(params.text_document.version);

        // We advertise FULL sync, so we expect the full text on each change.
        let text = params
            .content_changes
            .into_iter()
            .last()
            .map(|c| c.text)
            .unwrap_or_default();

        self.upsert_document(uri.clone(), text, version).await;
        self.publish_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Some clients only ask for diagnostics on save.
        self.publish_diagnostics(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().await;
        docs.remove(&params.text_document.uri);

        // Clear diagnostics when the document is closed.
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(static_completions())))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();

        let Some(doc) = self.get_document(&uri).await else {
            return Ok(None);
        };

        let position = params.text_document_position_params.position;
        let offset = offset_from_position_utf16(&doc.text, position);

        let name = match ident_at_offset(&doc.text, offset) {
            Some(n) => n,
            None => return Ok(None),
        };

        let tokens = match lex(&doc.text) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };
        let program = match parse(tokens) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let defs = collect_top_level_defs(&program);
        let Some(span) = defs.get(&name) else {
            return Ok(None);
        };

        let location = Location {
            uri,
            range: range_from_span_utf16(&doc.text, *span),
        };
        Ok(Some(GotoDefinitionResponse::Scalar(location)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();

        let Some(doc) = self.get_document(&uri).await else {
            return Ok(None);
        };

        let position = params.text_document_position_params.position;
        let offset = offset_from_position_utf16(&doc.text, position);

        let tokens = match lex(&doc.text) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };
        let program = match parse(tokens) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let info = match check_program_with_spans(&program) {
            Ok(i) => i,
            Err(_) => return Ok(None),
        };

        let mut best: Option<(Span, moon_typechecker::Type)> = None;
        let mut best_len: usize = usize::MAX;

        for (sp, ty) in &info.expr_types {
            if sp.start <= offset && offset < sp.end {
                let len = sp.end.saturating_sub(sp.start).max(1);
                if len < best_len {
                    best = Some((*sp, ty.clone()));
                    best_len = len;
                }
            }
        }
        if best.is_none() && offset > 0 {
            let off = offset - 1;
            for (sp, ty) in &info.expr_types {
                if sp.start <= off && off < sp.end {
                    let len = sp.end.saturating_sub(sp.start).max(1);
                    if len < best_len {
                        best = Some((*sp, ty.clone()));
                        best_len = len;
                    }
                }
            }
        }

        let Some((span, ty)) = best else {
            return Ok(None);
        };

        let contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**Type:** `{ty}`"),
        });

        Ok(Some(Hover {
            contents,
            range: Some(range_from_span_utf16(&doc.text, span)),
        }))
    }
}

fn diagnostics_for(uri: &Url, text: &str) -> Vec<Diagnostic> {
    let path = uri_to_path(uri);
    let source = Source::new(path, text.to_string());

    let tokens = match lex(&source.text) {
        Ok(t) => t,
        Err(e) => {
            return vec![Diagnostic {
                range: range_from_span_utf16(&source.text, e.span),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("moon".to_string()),
                message: format!("lex error: {}", e.message),
                ..Default::default()
            }]
        }
    };

    let program = match parse(tokens) {
        Ok(p) => p,
        Err(e) => {
            return vec![Diagnostic {
                range: range_from_span_utf16(&source.text, e.span),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("moon".to_string()),
                message: format!("parse error: {}", e.message),
                ..Default::default()
            }]
        }
    };

    if let Err(e) = check_program(&program) {
        return vec![Diagnostic {
            range: range_from_span_utf16(&source.text, e.span),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("moon".to_string()),
            message: format!("type error: {}", e.message),
            ..Default::default()
        }];
    }

    Vec::new()
}

fn uri_to_path(uri: &Url) -> PathBuf {
    if uri.scheme() == "file" {
        // Some clients may send URLs that cannot be converted (e.g. non-UTF8 paths).
        if let Ok(path) = uri.to_file_path() {
            return path;
        }
    }
    PathBuf::from(uri.as_str())
}

fn range_from_span_utf16(text: &str, span: Span) -> Range {
    let start = position_from_offset_utf16(text, span.start);
    let end = position_from_offset_utf16(text, span.end);
    Range { start, end }
}

fn position_from_offset_utf16(text: &str, offset: usize) -> Position {
    // LSP positions are (line, character) where character is UTF-16 code units.
    let clamped = offset.min(text.len());
    let mut line: u32 = 0;
    let mut col_utf16: u32 = 0;

    for (i, ch) in text.char_indices() {
        if i >= clamped {
            break;
        }

        if ch == '\n' {
            line += 1;
            col_utf16 = 0;
            continue;
        }

        let mut buf = [0u16; 2];
        let n = ch.encode_utf16(&mut buf).len();
        col_utf16 += n as u32;
    }

    Position {
        line,
        character: col_utf16,
    }
}

fn offset_from_position_utf16(text: &str, position: Position) -> usize {
    // Convert an LSP position (UTF-16 line/col) into a byte offset in `text`.
    let target_line = position.line;
    let target_col = position.character;

    let mut line: u32 = 0;
    let mut col_utf16: u32 = 0;

    for (i, ch) in text.char_indices() {
        if line > target_line {
            return i;
        }

        if line == target_line && col_utf16 >= target_col {
            return i;
        }

        if ch == '\n' {
            line += 1;
            col_utf16 = 0;
            continue;
        }

        let mut buf = [0u16; 2];
        let n = ch.encode_utf16(&mut buf).len();
        col_utf16 += n as u32;
    }

    text.len()
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn ident_at_offset(text: &str, offset: usize) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut i = offset.min(bytes.len().saturating_sub(1));

    // If we're on whitespace/punctuation, try stepping left once.
    if !is_ident_char(bytes[i]) {
        if i == 0 || !is_ident_char(bytes[i - 1]) {
            return None;
        }
        i -= 1;
    }

    if !is_ident_char(bytes[i]) {
        return None;
    }

    let mut start = i;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = i + 1;
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }

    Some(text[start..end].to_string())
}

fn collect_top_level_defs(program: &moon_core::ast::Program) -> HashMap<String, Span> {
    use moon_core::ast::Stmt;

    let mut defs = HashMap::new();
    for stmt in &program.stmts {
        match stmt {
            Stmt::Fn { name, span, .. } => {
                defs.insert(name.clone(), *span);
            }
            Stmt::Let { name, span, .. } => {
                defs.insert(name.clone(), *span);
            }
            _ => {}
        }
    }
    defs
}

fn static_completions() -> Vec<CompletionItem> {
    use CompletionItemKind as K;

    let mut items = Vec::new();

    // Keywords.
    for kw in ["let", "fn", "if", "else", "true", "false"] {
        items.push(CompletionItem {
            label: kw.to_string(),
            kind: Some(K::KEYWORD),
            insert_text: Some(kw.to_string()),
            ..Default::default()
        });
    }

    // Types.
    for ty in ["Int", "Bool", "String", "Unit", "Array", "Object"] {
        items.push(CompletionItem {
            label: ty.to_string(),
            kind: Some(K::CLASS),
            insert_text: Some(ty.to_string()),
            ..Default::default()
        });
    }

    // Builtins.
    items.push(CompletionItem {
        label: "gc".to_string(),
        kind: Some(K::FUNCTION),
        insert_text: Some("gc".to_string()),
        ..Default::default()
    });

    items
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_position_roundtrip_ascii() {
        let text = "let x = 1;\nlet y = x;\n";
        for offset in 0..=text.len() {
            let pos = position_from_offset_utf16(text, offset);
            let back = offset_from_position_utf16(text, pos);
            assert_eq!(back, offset);
        }
    }

    #[test]
    fn utf16_position_handles_surrogate_pairs() {
        let mut text = String::from("let s = \"");
        let emoji = '\u{1F600}';
        text.push(emoji);
        text.push_str("\";\n");

        let emoji_offset = text.find(emoji).unwrap();
        let before = position_from_offset_utf16(&text, emoji_offset);
        assert_eq!(before.line, 0);
        assert_eq!(before.character, 9);

        let after_offset = emoji_offset + emoji.len_utf8();
        let after = position_from_offset_utf16(&text, after_offset);
        assert_eq!(after.line, 0);
        assert_eq!(after.character, 11);

        // Targeting the middle of the surrogate pair should land on the end of the char.
        let mid = offset_from_position_utf16(
            &text,
            Position {
                line: 0,
                character: 10,
            },
        );
        assert_eq!(mid, after_offset);
    }
}
