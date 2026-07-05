use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Parser)]
#[command(name = "cli", about = "Ferramenta administrativa do sistema de pedidos")]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Insere pedidos de exemplo no banco
    Seed {
        #[arg(short, long, default_value_t = 3)]
        quantidade: u32,
    },
    /// Mostra um relatorio com total de pedidos e soma geral
    Relatorio,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://../api/pedidos.db?mode=rwc")
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

    let cli = Cli::parse();

    match cli.comando {
        Comando::Seed { quantidade } => {
            for i in 1..=quantidade {
                let total = i as f64 * 100.0;
                sqlx::query("INSERT INTO pedidos (total, quantidade_itens) VALUES (?, ?)")
                    .bind(total)
                    .bind(1_i64)
                    .execute(&pool)
                    .await
                    .unwrap();
                println!("Pedido de exemplo criado: total = {total}");
            }
        }
        Comando::Relatorio => {
            let (quantidade, soma): (i64, f64) =
                sqlx::query_as("SELECT COUNT(*), COALESCE(SUM(total), 0.0) FROM pedidos")
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            println!("Total de pedidos: {quantidade}");
            println!("Soma geral: {soma}");
        }
    }
}