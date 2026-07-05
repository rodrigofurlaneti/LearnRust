struct Produto {
    nome: String,
    preco: f64,
}

struct Carrinho {
    itens: Vec<Produto>,
}

impl Carrinho {
    fn novo() -> Self {
        Carrinho { itens: Vec::new() }
    }

    fn adicionar(&mut self, nome: &str, preco: f64) {
        self.itens.push(Produto {
            nome: nome.to_string(),
            preco,
        });
    }

    fn buscar(&self, nome: &str) -> Option<&Produto> {
        self.itens.iter().find(|produto| produto.nome == nome)
    }

    fn total(&self) -> f64 {
        self.itens.iter().map(|produto| produto.preco).sum()
    }
}

fn main() {
    let mut carrinho = Carrinho::novo();

    carrinho.adicionar("Mouse", 50.0);
    carrinho.adicionar("Teclado", 150.0);

    match carrinho.buscar("Teclado") {
        Some(produto) => println!("Encontrado: {} custa {:.2}", produto.nome, produto.preco),
        None => println!("Produto nao encontrado"),
    }

    match carrinho.buscar("Monitor") {
        Some(produto) => println!("Encontrado: {} custa {:.2}", produto.nome, produto.preco),
        None => println!("Produto nao encontrado"),
    }

    println!("Total do carrinho: {:.2}", carrinho.total());
}