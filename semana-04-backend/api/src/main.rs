use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dominio_pedidos::{ItemPedido, Pedido, PedidoRepository};
use infraestrutura::RepositorioEmMemoria;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    repositorio: Arc<Mutex<RepositorioEmMemoria>>,
    pool: SqlitePool,
}

#[derive(Deserialize)]
struct ItemRequest {
    produto: String,
    quantidade: u32,
    preco_unitario: f64,
}

#[derive(Deserialize)]
struct CriarPedidoRequest {
    itens: Vec<ItemRequest>,
}

#[derive(Serialize)]
struct PedidoResponse {
    quantidade_itens: usize,
    total: f64,
}

#[derive(Serialize, sqlx::FromRow)]
struct PedidoHistorico {
    id: i64,
    total: f64,
    quantidade_itens: i64,
}

async fn criar_pedido(
    State(estado): State<AppState>,
    Json(payload): Json<CriarPedidoRequest>,
) -> Result<Json<PedidoResponse>, (StatusCode, String)> {
    let mut pedido = Pedido::novo();

    for item_req in payload.itens {
        let item = ItemPedido::new(&item_req.produto, item_req.quantidade, item_req.preco_unitario)
            .map_err(|erro| {
                tracing::warn!("falha ao criar item: {erro}");
                (StatusCode::BAD_REQUEST, erro)
            })?;
        pedido.adicionar_item(item);
    }

    let resposta = PedidoResponse {
        quantidade_itens: pedido.quantidade_itens(),
        total: pedido.total(),
    };

    sqlx::query("INSERT INTO pedidos (total, quantidade_itens) VALUES (?, ?)")
        .bind(resposta.total)
        .bind(resposta.quantidade_itens as i64)
        .execute(&estado.pool)
        .await
        .map_err(|erro| (StatusCode::INTERNAL_SERVER_ERROR, erro.to_string()))?;

    {
        let mut repositorio = estado.repositorio.lock().unwrap();
        repositorio.salvar(pedido);
    }

    tracing::info!(total = resposta.total, "pedido criado com sucesso");

    Ok(Json(resposta))
}

async fn listar_pedidos(State(estado): State<AppState>) -> Json<Vec<PedidoResponse>> {
    let repositorio = estado.repositorio.lock().unwrap();
    let respostas: Vec<PedidoResponse> = repositorio
        .buscar_todos()
        .iter()
        .map(|p| PedidoResponse {
            quantidade_itens: p.quantidade_itens(),
            total: p.total(),
        })
        .collect();

    tracing::info!(quantidade = respostas.len(), "pedidos listados");

    Json(respostas)
}

async fn historico_pedidos(State(estado): State<AppState>) -> Json<Vec<PedidoHistorico>> {
    let registros = sqlx::query_as::<_, PedidoHistorico>(
        "SELECT id, total, quantidade_itens FROM pedidos ORDER BY id DESC",
    )
    .fetch_all(&estado.pool)
    .await
    .unwrap();

    Json(registros)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = SqlitePoolOptions::new()
        .connect("sqlite://pedidos.db?mode=rwc")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pedidos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            total REAL NOT NULL,
            quantidade_itens INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let estado = AppState {
        repositorio: Arc::new(Mutex::new(RepositorioEmMemoria::novo())),
        pool,
    };

    let app = Router::new()
        .route("/pedidos", post(criar_pedido).get(listar_pedidos))
        .route("/pedidos/historico", get(historico_pedidos))
        .with_state(estado);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::info!("servidor rodando em http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}