use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Qual agregado sofreu a mudanca. Enum fechado (Object Calisthenics:
/// "wrap primitives") em vez de uma `String` solta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEntity {
    Usuario,
    Cliente,
    Produto,
    Pedido,
}

/// O que aconteceu com o agregado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Insert,
    Update,
    Delete,
}

/// Snapshot de item de pedido dentro do payload de auditoria - espelha
/// `domain::ItemPedido`, mas em campos primitivos serializaveis (o
/// dominio nunca depende de `serde` diretamente).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPedidoSnapshot {
    pub produto_id: String,
    pub nome_produto: String,
    pub quantidade: u32,
    pub valor_unitario_centavos: i64,
    pub valor_total_centavos: i64,
}

/// O ESTADO COMPLETO do agregado no momento do evento - nao um diff.
///
/// Decisao de design da Semana 11: cada evento de `Insert`/`Update` carrega
/// o snapshot inteiro (nao so o campo que mudou). Isso simplifica MUITO o
/// replay no boot (`infra::audit_log_replayer`): reconstruir o cache e so
/// percorrer o arquivo em ordem e "sobrescrever" cada agregado com o
/// ultimo estado lido, sem precisar aplicar diffs incrementais uns sobre
/// os outros. O custo e um arquivo um pouco maior (campos repetidos a
/// cada update); o beneficio e um replay trivial de auditar e testar.
///
/// Nota de seguranca (decisao explicita, ver README): o payload de
/// `Usuario` inclui `password_hash` (nunca a senha em texto puro). Sem
/// isso nao seria possivel reconstruir o login apos um restart - mas
/// significa que este arquivo passa a ser tao sensivel quanto um dump do
/// "banco" de usuarios e precisa ser tratado assim (fora do controle de
/// versao, acesso restrito).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo")]
pub enum AuditPayload {
    Usuario {
        email: String,
        password_hash: String,
        role: String,
    },
    Cliente {
        nome: String,
        documento: String,
        email: String,
    },
    Produto {
        nome: String,
        preco_centavos: i64,
    },
    Pedido {
        cliente_id: String,
        itens: Vec<ItemPedidoSnapshot>,
        valor_total_centavos: i64,
    },
    /// Usado em eventos `Delete`: nao ha estado para guardar, so o fato
    /// de que o agregado (`aggregate_id`) deixou de existir.
    Removido,
}

/// Um registro imutavel de auditoria: "isto aconteceu, com este agregado,
/// neste instante, e o estado dele ficou assim". Trafega do repositorio
/// decorado (`infra::audited_*_repository`) ate quem grava no arquivo
/// (`infra::file_audit_logger`) e, na volta, e o mesmo tipo que
/// `infra::audit_log_replayer` le de volta do arquivo no boot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entity: AuditEntity,
    pub action: AuditAction,
    pub aggregate_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: AuditPayload,
}

impl AuditEntry {
    pub fn new(entity: AuditEntity, action: AuditAction, aggregate_id: String, payload: AuditPayload) -> Self {
        Self {
            entity,
            action,
            aggregate_id,
            occurred_at: Utc::now(),
            payload,
        }
    }

    /// Uma linha do arquivo de auditoria: um objeto JSON por linha
    /// (formato "JSON lines"/NDJSON). Cada linha e independente e
    /// autocontida - por isso da pra ler o arquivo de tras pra frente,
    /// linha por linha, sem precisar carregar tudo de uma vez.
    pub fn to_log_line(&self) -> String {
        let json = serde_json::to_string(self).expect("AuditEntry sempre deveria serializar");
        format!("{json}\n")
    }

    /// Usado pelo replay no boot (`infra::audit_log_replayer`) para
    /// reconstruir uma linha ja escrita. Uma linha malformada (arquivo
    /// truncado por uma queda no meio de uma escrita, por exemplo) vira
    /// `None` em vez de derrubar o boot inteiro - quem chama decide
    /// so logar um aviso e seguir para a proxima linha.
    pub fn parse_log_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        serde_json::from_str(trimmed).ok()
    }
}

/// Porta de saida para auditoria. Contrato deliberadamente sincrono e sem
/// retorno (`()`, nao `Result`): quem chama `record` (os repositorios
/// decorados) nao deve travar esperando I/O de disco, nem precisa saber
/// se a gravacao teve sucesso ou falhou. A implementacao concreta
/// (`infra::file_audit_logger::FileAuditLogger`) resolve isso enfileirando
/// a entrada num canal e devolvendo o controle imediatamente.
pub trait AuditLogger: Send + Sync {
    fn record(&self, entry: AuditEntry);
}
