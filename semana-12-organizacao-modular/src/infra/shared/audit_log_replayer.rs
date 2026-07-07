use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::shared::audit_logger::{AuditAction, AuditEntity, AuditEntry, AuditPayload, ItemPedidoSnapshot};
use crate::domain::{
    Cliente, ClienteId, ClienteRepository, Cpf, Dinheiro, Email, HashedPassword, ItemPedido, Nome,
    Pedido, PedidoId, PedidoRepository, Produto, ProdutoId, ProdutoRepository, Quantidade, Role,
    User, UserId, UserRepository,
};

/// Reconstroi o cache em memoria a partir do log de auditoria no boot -
/// a "reidratacao" pedida na Semana 11 ("como um banco de dados mais
/// rapido"). So deve ser chamado UMA vez, no composition root
/// (`main.rs`), ANTES de o `FileAuditLogger` comecar a aceitar escritas
/// novas - senao corremos o risco de ler, no meio do replay, uma linha
/// que acabou de ser escrita pela propria inicializacao (ex.: o seed do
/// admin).
///
/// Estrategia: o arquivo e um log de eventos "full snapshot" (cada
/// `Insert`/`Update` carrega o ESTADO INTEIRO do agregado, nao um diff -
/// ver `application::audit_logger::AuditPayload`), entao reconstruir o
/// cache e simplesmente percorrer o arquivo em ordem cronologica (a mesma
/// ordem em que foi escrito, porque e append-only) e reaplicar cada
/// evento contra o repositorio de cache correspondente, exatamente como
/// aconteceu da primeira vez: um `Insert` volta a ser um `save`, um
/// `Update` volta a ser um `update`, um `Delete` volta a ser um `delete`.
///
/// Recebe os repositorios de CACHE PUROS (`Arc<dyn UserRepository>` etc,
/// nao decorados com auditoria) - reaplicar eventos que ja estao no
/// arquivo nao deveria gerar novas linhas de auditoria, senao o arquivo
/// cresceria dobrado a cada restart.
pub struct AuditLogReplayer;

impl AuditLogReplayer {
    pub async fn replay(
        path: &Path,
        usuarios: &Arc<dyn UserRepository>,
        clientes: &Arc<dyn ClienteRepository>,
        produtos: &Arc<dyn ProdutoRepository>,
        pedidos: &Arc<dyn PedidoRepository>,
    ) {
        let Some(conteudo) = Self::ler_arquivo_se_existir(path).await else {
            tracing::info!("nenhum arquivo de auditoria encontrado em {:?} - cache comeca vazio", path);
            return;
        };

        let mut eventos_aplicados = 0usize;
        let mut linhas_ignoradas = 0usize;

        for linha in conteudo.lines() {
            match AuditEntry::parse_log_line(linha) {
                Some(entry) => {
                    Self::aplicar(entry, usuarios, clientes, produtos, pedidos).await;
                    eventos_aplicados += 1;
                }
                None if linha.trim().is_empty() => {}
                None => linhas_ignoradas += 1,
            }
        }

        tracing::info!(
            "cache reidratado a partir de {:?}: {} evento(s) aplicado(s), {} linha(s) ignorada(s)",
            path,
            eventos_aplicados,
            linhas_ignoradas
        );
    }

    async fn ler_arquivo_se_existir(path: &Path) -> Option<String> {
        tokio::fs::read_to_string(path).await.ok()
    }

    async fn aplicar(
        entry: AuditEntry,
        usuarios: &Arc<dyn UserRepository>,
        clientes: &Arc<dyn ClienteRepository>,
        produtos: &Arc<dyn ProdutoRepository>,
        pedidos: &Arc<dyn PedidoRepository>,
    ) {
        let resultado = match entry.entity {
            AuditEntity::Usuario => Self::aplicar_usuario(&entry, usuarios).await,
            AuditEntity::Cliente => Self::aplicar_cliente(&entry, clientes).await,
            AuditEntity::Produto => Self::aplicar_produto(&entry, produtos).await,
            AuditEntity::Pedido => Self::aplicar_pedido(&entry, pedidos).await,
        };

        if let Err(motivo) = resultado {
            tracing::warn!(
                "evento de auditoria ignorado no replay (entidade={:?}, id={}): {}",
                entry.entity,
                entry.aggregate_id,
                motivo
            );
        }
    }

    // --- Usuario -----------------------------------------------------

    async fn aplicar_usuario(entry: &AuditEntry, usuarios: &Arc<dyn UserRepository>) -> Result<(), String> {
        let user_id = UserId::from_uuid(Self::parse_uuid(&entry.aggregate_id)?);

        if entry.action == AuditAction::Delete {
            return usuarios.delete(user_id).await.map_err(|erro| erro.to_string());
        }

        let user = Self::reconstruir_usuario(user_id, &entry.payload)?;
        let outcome = if entry.action == AuditAction::Insert {
            usuarios.save(&user).await
        } else {
            usuarios.update(&user).await
        };
        outcome.map_err(|erro| erro.to_string())
    }

    fn reconstruir_usuario(user_id: UserId, payload: &AuditPayload) -> Result<User, String> {
        let AuditPayload::Usuario { email, password_hash, role } = payload else {
            return Err("payload de usuario ausente ou invalido".to_string());
        };
        let email = Email::parse(email).map_err(|erro| erro.to_string())?;
        let hashed = HashedPassword::from_hash(password_hash.clone());
        let role = Role::parse(role).ok_or_else(|| "role desconhecida no replay".to_string())?;
        Ok(User::reconstitute(user_id, email, hashed, role))
    }

    // --- Cliente -------------------------------------------------------

    async fn aplicar_cliente(entry: &AuditEntry, clientes: &Arc<dyn ClienteRepository>) -> Result<(), String> {
        let cliente_id = ClienteId::from_uuid(Self::parse_uuid(&entry.aggregate_id)?);

        if entry.action == AuditAction::Delete {
            return clientes.delete(cliente_id).await.map_err(|erro| erro.to_string());
        }

        let cliente = Self::reconstruir_cliente(cliente_id, &entry.payload)?;
        let outcome = if entry.action == AuditAction::Insert {
            clientes.save(&cliente).await
        } else {
            clientes.update(&cliente).await
        };
        outcome.map_err(|erro| erro.to_string())
    }

    fn reconstruir_cliente(id: ClienteId, payload: &AuditPayload) -> Result<Cliente, String> {
        let AuditPayload::Cliente { nome, documento, email } = payload else {
            return Err("payload de cliente ausente ou invalido".to_string());
        };
        let nome = Nome::parse(nome).map_err(|erro| erro.to_string())?;
        let documento = Cpf::parse(documento).map_err(|erro| erro.to_string())?;
        let email = Email::parse(email).map_err(|erro| erro.to_string())?;
        Ok(Cliente::reconstitute(id, nome, documento, email))
    }

    // --- Produto ---------------------------------------------------------

    async fn aplicar_produto(entry: &AuditEntry, produtos: &Arc<dyn ProdutoRepository>) -> Result<(), String> {
        let produto_id = ProdutoId::from_uuid(Self::parse_uuid(&entry.aggregate_id)?);

        if entry.action == AuditAction::Delete {
            return produtos.delete(produto_id).await.map_err(|erro| erro.to_string());
        }

        let produto = Self::reconstruir_produto(produto_id, &entry.payload)?;
        let outcome = if entry.action == AuditAction::Insert {
            produtos.save(&produto).await
        } else {
            produtos.update(&produto).await
        };
        outcome.map_err(|erro| erro.to_string())
    }

    fn reconstruir_produto(id: ProdutoId, payload: &AuditPayload) -> Result<Produto, String> {
        let AuditPayload::Produto { nome, preco_centavos } = payload else {
            return Err("payload de produto ausente ou invalido".to_string());
        };
        let nome = Nome::parse(nome).map_err(|erro| erro.to_string())?;
        let preco = Dinheiro::from_centavos(*preco_centavos).map_err(|erro| erro.to_string())?;
        Ok(Produto::reconstitute(id, nome, preco))
    }

    // --- Pedido ----------------------------------------------------------

    async fn aplicar_pedido(entry: &AuditEntry, pedidos: &Arc<dyn PedidoRepository>) -> Result<(), String> {
        let pedido_id = PedidoId::from_uuid(Self::parse_uuid(&entry.aggregate_id)?);

        if entry.action == AuditAction::Delete {
            return pedidos.delete(pedido_id).await.map_err(|erro| erro.to_string());
        }

        // Pedido nunca tem evento de Update (a porta nao expoe esse
        // metodo - ver domain::pedido_repository), entao qualquer evento
        // que nao seja Delete e sempre um Insert.
        let pedido = Self::reconstruir_pedido(pedido_id, &entry.payload)?;
        pedidos.save(&pedido).await.map_err(|erro| erro.to_string())
    }

    fn reconstruir_pedido(id: PedidoId, payload: &AuditPayload) -> Result<Pedido, String> {
        let AuditPayload::Pedido { cliente_id, itens, valor_total_centavos } = payload else {
            return Err("payload de pedido ausente ou invalido".to_string());
        };
        let cliente_id = ClienteId::from_uuid(Self::parse_uuid(cliente_id)?);
        let itens = itens
            .iter()
            .map(Self::reconstruir_item)
            .collect::<Result<Vec<_>, _>>()?;
        let valor_total = Dinheiro::from_centavos(*valor_total_centavos).map_err(|erro| erro.to_string())?;
        Ok(Pedido::reconstitute(id, cliente_id, itens, valor_total))
    }

    fn reconstruir_item(snapshot: &ItemPedidoSnapshot) -> Result<ItemPedido, String> {
        let produto_id = ProdutoId::from_uuid(Self::parse_uuid(&snapshot.produto_id)?);
        let nome_produto = Nome::parse(&snapshot.nome_produto).map_err(|erro| erro.to_string())?;
        let quantidade = Quantidade::parse(snapshot.quantidade).map_err(|erro| erro.to_string())?;
        let valor_unitario =
            Dinheiro::from_centavos(snapshot.valor_unitario_centavos).map_err(|erro| erro.to_string())?;
        let valor_total =
            Dinheiro::from_centavos(snapshot.valor_total_centavos).map_err(|erro| erro.to_string())?;
        Ok(ItemPedido::reconstitute(produto_id, nome_produto, quantidade, valor_unitario, valor_total))
    }

    // --- Auxiliar comum ----------------------------------------------

    fn parse_uuid(raw: &str) -> Result<Uuid, String> {
        Uuid::parse_str(raw).map_err(|_| format!("id invalido no log de auditoria: {raw}"))
    }
}
