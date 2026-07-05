use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Error, Debug)]
enum ErroProcessamento {
    #[error("item {0} e invalido para processamento")]
    ItemInvalido(i32),
}

async fn processar_item(item: i32) -> Result<(), ErroProcessamento> {
    println!("Processando item {item}...");
    sleep(Duration::from_millis(200)).await;

    if item == 3 {
        return Err(ErroProcessamento::ItemInvalido(item));
    }

    println!("Item {item} processado.");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<i32>(10);

    let produtor = tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(i).await.unwrap();
        }
    });

    let consumidor = tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            match processar_item(item).await {
                Ok(()) => {}
                Err(erro) => println!("Falha ao processar: {erro}"),
            }
        }
    });

    produtor.await?;
    consumidor.await?;

    println!("Processamento finalizado.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn processar_item_valido_funciona() {
        let resultado = processar_item(1).await;
        assert!(resultado.is_ok());
    }

    #[tokio::test]
    async fn processar_item_3_falha() {
        let resultado = processar_item(3).await;
        assert!(resultado.is_err());
    }
}