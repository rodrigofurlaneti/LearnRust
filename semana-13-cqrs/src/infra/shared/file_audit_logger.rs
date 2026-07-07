use std::path::PathBuf;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::application::shared::audit_logger::{AuditEntry, AuditLogger};

/// Adapter concreto do port `AuditLogger`: grava cada `AuditEntry` como
/// uma linha JSON num arquivo texto (`auditoria.txt` por padrao, formato
/// "JSON lines" desde a Semana 11), funcionando ao mesmo tempo como log de
/// auditoria (pedido na Semana 10) e como EVENT LOG de recuperacao: e este
/// mesmo arquivo que `infra::audit_log_replayer` le no boot para
/// reconstruir o cache (ver `main.rs` e o README, secao sobre
/// reidratacao). Toda vez que um insert, update ou delete acontece em
/// `User`/`Cliente`/`Produto`/`Pedido`, uma linha nova aparece aqui.
///
/// Design "fire-and-forget" (requisito explicito: nao travar a thread que
/// chama, sem retorno de sucesso/falha):
///
/// 1. `record()` e sincrono e so empurra a entrada num canal MPSC
///    (`tokio::sync::mpsc`, o mesmo mecanismo de canais visto na Semana 3
///    com `worker-pedidos`) - isso e uma operacao de memoria, nunca de
///    disco, entao nunca bloqueia quem chamou.
/// 2. Uma unica task de background (iniciada em `start`) e a UNICA dona
///    do arquivo: ela consome o canal em loop e escreve as linhas em
///    sequencia. Ter um unico escritor evita qualquer corrida de escrita
///    concorrente no arquivo, sem precisar de lock nenhum sobre o arquivo
///    em si.
/// 3. Se o canal falhar ao enviar (ex.: a task de background caiu) o erro
///    e silenciosamente ignorado - por design: auditoria nunca pode
///    derrubar ou atrasar um cadastro de usuario/cliente.
pub struct FileAuditLogger {
    sender: UnboundedSender<AuditEntry>,
}

impl FileAuditLogger {
    /// Inicia a task de background e devolve um logger pronto para uso.
    /// Deve ser chamado uma unica vez, no composition root (`main.rs`).
    pub fn start(path: PathBuf) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<AuditEntry>();
        tokio::spawn(Self::run_writer_loop(path, receiver));
        Self { sender }
    }

    async fn run_writer_loop(path: PathBuf, mut receiver: UnboundedReceiver<AuditEntry>) {
        while let Some(entry) = receiver.recv().await {
            Self::append_entry_or_log_error(&path, &entry).await;
        }
    }

    async fn append_entry_or_log_error(path: &PathBuf, entry: &AuditEntry) {
        let outcome = Self::append_entry(path, entry).await;
        if let Err(error) = outcome {
            tracing::error!("falha ao gravar auditoria em {:?}: {}", path, error);
        }
    }

    async fn append_entry(path: &PathBuf, entry: &AuditEntry) -> tokio::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path).await?;
        file.write_all(entry.to_log_line().as_bytes()).await
    }
}

impl AuditLogger for FileAuditLogger {
    fn record(&self, entry: AuditEntry) {
        // `send` de um canal unbounded nunca bloqueia. O `_` descarta de
        // proposito o `Result`: nao ha nada de util a fazer aqui se o
        // receiver ja tiver caido, e o contrato de `AuditLogger` (ver
        // application/audit_logger.rs) deixa explicito que quem registra
        // auditoria nao espera (nem recebe) confirmacao de sucesso/falha.
        let _ = self.sender.send(entry);
    }
}
