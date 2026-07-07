use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cadastro_api::application::shared::audit_logger::AuditLogger;
use cadastro_api::application::produto::delete_produto::DeleteProduto;
use cadastro_api::application::shared::dto::{
    RegisterClienteInput, RegisterPedidoInput, RegisterProdutoInput, RegisterUserInput,
    UpdateProdutoInput,
};
use cadastro_api::application::cliente::register_cliente::RegisterCliente;
use cadastro_api::application::pedido::register_pedido::RegisterPedido;
use cadastro_api::application::produto::register_produto::RegisterProduto;
use cadastro_api::application::usuario::register_user::RegisterUser;
use cadastro_api::application::produto::update_produto::UpdateProduto;
use cadastro_api::domain::{ClienteRepository, PedidoRepository, ProdutoRepository, UserRepository};
use cadastro_api::infra::shared::audit_log_replayer::AuditLogReplayer;
use cadastro_api::infra::cliente::audited_cliente_repository::AuditedClienteRepository;
use cadastro_api::infra::pedido::audited_pedido_repository::AuditedPedidoRepository;
use cadastro_api::infra::produto::audited_produto_repository::AuditedProdutoRepository;
use cadastro_api::infra::usuario::audited_user_repository::AuditedUserRepository;
use cadastro_api::infra::usuario::bcrypt_hasher::BcryptPasswordHasher;
use cadastro_api::infra::cliente::cache_cliente_repository::CacheClienteRepository;
use cadastro_api::infra::pedido::cache_pedido_repository::CachePedidoRepository;
use cadastro_api::infra::produto::cache_produto_repository::CacheProdutoRepository;
use cadastro_api::infra::usuario::cache_user_repository::CacheUserRepository;
use cadastro_api::infra::shared::file_audit_logger::FileAuditLogger;

/// Ambiente de teste "com auditoria": cada teste usa seu proprio arquivo
/// de eventos (isolado dos demais), com os repositorios ja decorados -
/// exatamente a composicao usada em producao (`main.rs`), so trocando o
/// caminho do arquivo.
#[allow(dead_code)]
struct Ambiente {
    register_user: RegisterUser,
    register_cliente: RegisterCliente,
    register_produto: RegisterProduto,
    update_produto: UpdateProduto,
    delete_produto: DeleteProduto,
    register_pedido: RegisterPedido,
    produtos: Arc<dyn ProdutoRepository>,
    pedidos: Arc<dyn PedidoRepository>,
    audit_log_path: PathBuf,
}

fn caminho_unico(nome_teste: &str) -> PathBuf {
    let nome_arquivo = format!("auditoria_teste_{}_{}.txt", nome_teste, uuid::Uuid::new_v4());
    std::env::temp_dir().join(nome_arquivo)
}

fn montar_ambiente(nome_teste: &str) -> Ambiente {
    let audit_log_path = caminho_unico(nome_teste);
    let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(audit_log_path.clone()));

    let usuarios: Arc<dyn UserRepository> = Arc::new(AuditedUserRepository::new(
        Arc::new(CacheUserRepository::new()),
        audit.clone(),
    ));
    let clientes: Arc<dyn ClienteRepository> = Arc::new(AuditedClienteRepository::new(
        Arc::new(CacheClienteRepository::new()),
        audit.clone(),
    ));
    let produtos: Arc<dyn ProdutoRepository> = Arc::new(AuditedProdutoRepository::new(
        Arc::new(CacheProdutoRepository::new()),
        audit.clone(),
    ));
    let pedidos: Arc<dyn PedidoRepository> = Arc::new(AuditedPedidoRepository::new(
        Arc::new(CachePedidoRepository::new()),
        audit,
    ));

    let hasher = Arc::new(BcryptPasswordHasher::new(4)); // custo baixo: testes rapidos

    Ambiente {
        register_user: RegisterUser::new(usuarios, hasher),
        register_cliente: RegisterCliente::new(clientes.clone()),
        register_produto: RegisterProduto::new(produtos.clone()),
        update_produto: UpdateProduto::new(produtos.clone()),
        delete_produto: DeleteProduto::new(produtos.clone()),
        register_pedido: RegisterPedido::new(clientes, produtos.clone(), pedidos.clone()),
        produtos,
        pedidos,
        audit_log_path,
    }
}

async fn aguardar_flush_da_auditoria() {
    tokio::time::sleep(Duration::from_millis(150)).await;
}

fn produto_input(nome: &str, preco: &str) -> RegisterProdutoInput {
    RegisterProdutoInput {
        nome: nome.to_string(),
        preco: preco.to_string(),
    }
}

fn cliente_input() -> RegisterClienteInput {
    RegisterClienteInput {
        nome: "Rodrigo Furlaneti".to_string(),
        documento: "111.444.777-35".to_string(),
        email: "cliente@example.com".to_string(),
    }
}

// ---------------------------------------------------------------------
// Regra de negocio central da Semana 11: preco/nome congelados no pedido.
// ---------------------------------------------------------------------

#[tokio::test]
async fn pedido_guarda_o_preco_do_produto_no_momento_da_compra_mesmo_apos_reajuste() {
    let ambiente = montar_ambiente("snapshot_preco");

    let cliente = ambiente
        .register_cliente
        .execute(cliente_input())
        .await
        .expect("cadastro de cliente deveria funcionar");

    let produto = ambiente
        .register_produto
        .execute(produto_input("Teclado mecanico", "100.00"))
        .await
        .expect("cadastro de produto deveria funcionar");

    let pedido = ambiente
        .register_pedido
        .execute(RegisterPedidoInput {
            cliente_id: cliente.cliente_id.clone(),
            itens: vec![cadastro_api::application::shared::dto::ItemPedidoInput {
                produto_id: produto.produto_id.clone(),
                quantidade: 2,
            }],
        })
        .await
        .expect("pedido deveria ser criado");

    assert_eq!(pedido.itens.len(), 1);
    assert_eq!(pedido.itens[0].valor_unitario, "100.00");
    assert_eq!(pedido.itens[0].valor_total, "200.00");
    assert_eq!(pedido.valor_total, "200.00");

    // Reajuste de preco DEPOIS do pedido feito.
    ambiente
        .update_produto
        .execute(
            cadastro_api::domain::ProdutoId::from_uuid(uuid::Uuid::parse_str(&produto.produto_id).unwrap()),
            UpdateProdutoInput {
                nome: "Teclado mecanico".to_string(),
                preco: "150.00".to_string(),
            },
        )
        .await
        .expect("reajuste de preco deveria funcionar");

    // O pedido antigo NAO pode mudar de valor so porque o produto ficou
    // mais caro depois - essa e a regra de negocio pedida na Semana 11.
    let pedido_id = cadastro_api::domain::PedidoId::from_uuid(uuid::Uuid::parse_str(&pedido.pedido_id).unwrap());
    let pedido_relido = ambiente
        .pedidos
        .find_by_id(pedido_id)
        .await
        .expect("busca nao deveria falhar")
        .expect("pedido deveria continuar existindo");

    assert_eq!(pedido_relido.valor_total().as_reais_str(), "200.00");
    assert_eq!(pedido_relido.itens()[0].valor_unitario().as_reais_str(), "100.00");

    // O produto, por sua vez, reflete o preco novo normalmente.
    let produto_atual = ambiente
        .produtos
        .find_by_id(cadastro_api::domain::ProdutoId::from_uuid(
            uuid::Uuid::parse_str(&produto.produto_id).unwrap(),
        ))
        .await
        .expect("busca nao deveria falhar")
        .expect("produto deveria continuar existindo");
    assert_eq!(produto_atual.preco().as_reais_str(), "150.00");
}

#[tokio::test]
async fn rejeita_pedido_sem_itens() {
    let ambiente = montar_ambiente("pedido_sem_itens");
    let cliente = ambiente
        .register_cliente
        .execute(cliente_input())
        .await
        .expect("cadastro de cliente deveria funcionar");

    let resultado = ambiente
        .register_pedido
        .execute(RegisterPedidoInput {
            cliente_id: cliente.cliente_id,
            itens: vec![],
        })
        .await;

    assert!(resultado.is_err());
}

#[tokio::test]
async fn rejeita_pedido_com_produto_inexistente() {
    let ambiente = montar_ambiente("produto_inexistente");
    let cliente = ambiente
        .register_cliente
        .execute(cliente_input())
        .await
        .expect("cadastro de cliente deveria funcionar");

    let resultado = ambiente
        .register_pedido
        .execute(RegisterPedidoInput {
            cliente_id: cliente.cliente_id,
            itens: vec![cadastro_api::application::shared::dto::ItemPedidoInput {
                produto_id: uuid::Uuid::new_v4().to_string(),
                quantidade: 1,
            }],
        })
        .await;

    assert!(resultado.is_err());
}

#[tokio::test]
async fn rejeita_pedido_com_quantidade_zero() {
    let ambiente = montar_ambiente("quantidade_zero");
    let cliente = ambiente
        .register_cliente
        .execute(cliente_input())
        .await
        .expect("cadastro de cliente deveria funcionar");
    let produto = ambiente
        .register_produto
        .execute(produto_input("Mouse", "50.00"))
        .await
        .expect("cadastro de produto deveria funcionar");

    let resultado = ambiente
        .register_pedido
        .execute(RegisterPedidoInput {
            cliente_id: cliente.cliente_id,
            itens: vec![cadastro_api::application::shared::dto::ItemPedidoInput {
                produto_id: produto.produto_id,
                quantidade: 0,
            }],
        })
        .await;

    assert!(resultado.is_err());
}

// ---------------------------------------------------------------------
// A funcionalidade marquee da Semana 11: reidratacao do cache no boot.
// ---------------------------------------------------------------------

#[tokio::test]
async fn cache_e_reidratado_a_partir_do_arquivo_de_auditoria_apos_reinicio_simulado() {
    let audit_log_path = caminho_unico("reidratacao");

    // ---- "Processo 1": estado antes do restart -------------------------
    let email_usuario;
    let documento_cliente;
    let nome_produto_final;
    let preco_produto_final_centavos;
    let pedido_id_string;
    let cliente_id_string;
    let produto_removido_id_string;

    {
        let audit: Arc<dyn AuditLogger> = Arc::new(FileAuditLogger::start(audit_log_path.clone()));
        let usuarios: Arc<dyn UserRepository> = Arc::new(AuditedUserRepository::new(
            Arc::new(CacheUserRepository::new()),
            audit.clone(),
        ));
        let clientes: Arc<dyn ClienteRepository> = Arc::new(AuditedClienteRepository::new(
            Arc::new(CacheClienteRepository::new()),
            audit.clone(),
        ));
        let produtos: Arc<dyn ProdutoRepository> = Arc::new(AuditedProdutoRepository::new(
            Arc::new(CacheProdutoRepository::new()),
            audit.clone(),
        ));
        let pedidos: Arc<dyn PedidoRepository> = Arc::new(AuditedPedidoRepository::new(
            Arc::new(CachePedidoRepository::new()),
            audit,
        ));
        let hasher = Arc::new(BcryptPasswordHasher::new(4));

        let register_user = RegisterUser::new(usuarios, hasher);
        let register_cliente = RegisterCliente::new(clientes.clone());
        let register_produto = RegisterProduto::new(produtos.clone());
        let update_produto = UpdateProduto::new(produtos.clone());
        let delete_produto = DeleteProduto::new(produtos.clone());
        let register_pedido = RegisterPedido::new(clientes, produtos.clone(), pedidos);

        let usuario = register_user
            .execute(RegisterUserInput {
                email: "sobrevivente@example.com".to_string(),
                password: "SenhaForte123".to_string(),
            })
            .await
            .expect("cadastro de usuario deveria funcionar");
        email_usuario = usuario.email;

        let cliente = register_cliente
            .execute(cliente_input())
            .await
            .expect("cadastro de cliente deveria funcionar");
        documento_cliente = cliente.documento.clone();
        cliente_id_string = cliente.cliente_id.clone();

        // Produto que vai ser atualizado (para testar replay de Update).
        // O rename/reajuste acontece ANTES do pedido ser criado, entao o
        // snapshot do item do pedido deve capturar o estado JA atualizado.
        let produto = register_produto
            .execute(produto_input("Monitor 27pol", "1200.00"))
            .await
            .expect("cadastro de produto deveria funcionar");
        let produto_atualizado = update_produto
            .execute(
                cadastro_api::domain::ProdutoId::from_uuid(uuid::Uuid::parse_str(&produto.produto_id).unwrap()),
                UpdateProdutoInput {
                    nome: "Monitor 27pol UltraWide".to_string(),
                    preco: "1500.00".to_string(),
                },
            )
            .await
            .expect("atualizacao de produto deveria funcionar");
        nome_produto_final = produto_atualizado.nome.clone();
        preco_produto_final_centavos = 150_000i64;

        // Produto que vai ser removido (para testar replay de Delete).
        let produto_temporario = register_produto
            .execute(produto_input("Produto descontinuado", "10.00"))
            .await
            .expect("cadastro de produto deveria funcionar");
        produto_removido_id_string = produto_temporario.produto_id.clone();
        delete_produto
            .execute(cadastro_api::domain::ProdutoId::from_uuid(
                uuid::Uuid::parse_str(&produto_temporario.produto_id).unwrap(),
            ))
            .await
            .expect("remocao de produto deveria funcionar");

        let pedido = register_pedido
            .execute(RegisterPedidoInput {
                cliente_id: cliente.cliente_id,
                itens: vec![cadastro_api::application::shared::dto::ItemPedidoInput {
                    produto_id: produto.produto_id,
                    quantidade: 1,
                }],
            })
            .await
            .expect("pedido deveria ser criado");
        pedido_id_string = pedido.pedido_id;

        aguardar_flush_da_auditoria().await;
    } // "processo 1" cai aqui - todos os Arcs somem, ninguem mais escreve no arquivo.

    // ---- "Processo 2": novo boot, cache vazio, so o arquivo sobrevive --
    let usuarios_novos: Arc<dyn UserRepository> = Arc::new(CacheUserRepository::new());
    let clientes_novos: Arc<dyn ClienteRepository> = Arc::new(CacheClienteRepository::new());
    let produtos_novos: Arc<dyn ProdutoRepository> = Arc::new(CacheProdutoRepository::new());
    let pedidos_novos: Arc<dyn PedidoRepository> = Arc::new(CachePedidoRepository::new());

    AuditLogReplayer::replay(
        &audit_log_path,
        &usuarios_novos,
        &clientes_novos,
        &produtos_novos,
        &pedidos_novos,
    )
    .await;

    // Usuario reidratado, inclusive achavel por email (prova que o hash
    // de senha tambem voltou, nao so um registro vazio).
    let email = cadastro_api::domain::Email::parse(&email_usuario).unwrap();
    let usuario_reidratado = usuarios_novos
        .find_by_email(&email)
        .await
        .expect("busca nao deveria falhar")
        .expect("usuario deveria ter sido reidratado do arquivo de auditoria");
    assert_eq!(usuario_reidratado.email().as_str(), email_usuario);

    // Cliente reidratado.
    let cliente_id = cadastro_api::domain::ClienteId::from_uuid(uuid::Uuid::parse_str(&cliente_id_string).unwrap());
    let cliente_reidratado = clientes_novos
        .find_by_id(cliente_id)
        .await
        .expect("busca nao deveria falhar")
        .expect("cliente deveria ter sido reidratado");
    assert_eq!(cliente_reidratado.documento().as_str(), documento_cliente);

    // Pedido reidratado - o snapshot do item reflete o nome/preco do
    // produto NO MOMENTO DO PEDIDO (que aconteceu depois do rename).
    let pedido_id = cadastro_api::domain::PedidoId::from_uuid(uuid::Uuid::parse_str(&pedido_id_string).unwrap());
    let pedido_reidratado = pedidos_novos
        .find_by_id(pedido_id)
        .await
        .expect("busca nao deveria falhar")
        .expect("pedido deveria ter sido reidratado");
    assert_eq!(pedido_reidratado.itens()[0].nome_produto().as_str(), nome_produto_final);

    // Produto reidratado com o ESTADO FINAL (pos-update), nao o original.
    let produto_id = pedido_reidratado.itens()[0].produto_id();
    let produto_reidratado = produtos_novos
        .find_by_id(produto_id)
        .await
        .expect("busca nao deveria falhar")
        .expect("produto deveria ter sido reidratado com o ultimo estado (pos-update)");
    assert_eq!(produto_reidratado.nome().as_str(), nome_produto_final);
    assert_eq!(produto_reidratado.preco().as_centavos(), preco_produto_final_centavos);

    // Produto removido continua removido - o replay de Delete funcionou.
    let produto_removido_id = cadastro_api::domain::ProdutoId::from_uuid(
        uuid::Uuid::parse_str(&produto_removido_id_string).unwrap(),
    );
    let produto_removido = produtos_novos
        .find_by_id(produto_removido_id)
        .await
        .expect("busca nao deveria falhar");
    assert!(produto_removido.is_none());
}
