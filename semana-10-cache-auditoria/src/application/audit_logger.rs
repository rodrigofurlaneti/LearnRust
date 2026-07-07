use chrono::{DateTime, Utc};

/// Qual agregado sofreu a mudanca. Enum fechado (Object Calisthenics:
/// "wrap primitives") em vez de uma `String` solta tipo "usuario"/"cliente"
/// espalhada pelo codigo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEntity {
    Usuario,
    Cliente,
}

impl AuditEntity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEntity::Usuario => "USUARIO",
            AuditEntity::Cliente => "CLIENTE",
        }
    }
}

/// O que aconteceu com o agregado. Mapeia 1:1 com o pedido da Semana 10:
/// todo insert/update/delete das tabelas de usuario e cliente precisa
/// virar uma linha no arquivo de historico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Insert,
    Update,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Insert => "INSERT",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
        }
    }
}

/// Um registro imutavel de auditoria: "isto aconteceu, com este agregado,
/// neste instante". E a unidade que trafega do repositorio decorado
/// (`infra::audited_user_repository` / `infra::audited_cliente_repository`)
/// ate quem efetivamente grava no arquivo (`infra::file_audit_logger`).
#[derive(Debug, Clone)]
pub struct AuditEntry {
    entity: AuditEntity,
    action: AuditAction,
    aggregate_id: String,
    details: String,
    occurred_at: DateTime<Utc>,
}

impl AuditEntry {
    pub fn new(entity: AuditEntity, action: AuditAction, aggregate_id: String, details: String) -> Self {
        Self {
            entity,
            action,
            aggregate_id,
            details,
            occurred_at: Utc::now(),
        }
    }

    /// Formato de uma linha do arquivo de auditoria. Fica aqui (e nao em
    /// `infra`) porque e vocabulario da propria entrada de auditoria, nao
    /// um detalhe de "como o arquivo e aberto/escrito".
    pub fn to_log_line(&self) -> String {
        format!(
            "{} | {:<8} | {:<6} | id={} | {}\n",
            self.occurred_at.to_rfc3339(),
            self.entity.as_str(),
            self.action.as_str(),
            self.aggregate_id,
            self.details
        )
    }
}

/// Porta de saida para auditoria. Contrato deliberadamente sincrono e sem
/// retorno (`()`, nao `Result`): quem chama `record` (os repositorios
/// decorados) nao deve travar esperando I/O de disco, nem precisa saber
/// se a gravacao teve sucesso ou falhou - e um requisito explicito da
/// Semana 10 ("nao precisa ter retorno de sucesso ou falha"). A
/// implementacao concreta (`infra::file_audit_logger::FileAuditLogger`)
/// resolve isso enfileirando a entrada num canal e devolvendo o controle
/// imediatamente; quem efetivamente escreve no arquivo e uma task de
/// background separada.
pub trait AuditLogger: Send + Sync {
    fn record(&self, entry: AuditEntry);
}
