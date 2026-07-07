//! Infraestrutura que atravessa todos os agregados: o logger de auditoria
//! que escreve o arquivo (`FileAuditLogger`) e quem le esse mesmo arquivo
//! de volta no boot para reidratar o cache (`AuditLogReplayer`).

pub mod audit_log_replayer;
pub mod file_audit_logger;
