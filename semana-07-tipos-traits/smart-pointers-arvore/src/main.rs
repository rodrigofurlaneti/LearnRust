use std::rc::Rc;

// Uma struct recursiva SEM Box nao compilaria: o compilador precisa saber
// o tamanho exato de NoArvore em compile-time, e "conter a si mesma"
// diretamente geraria tamanho infinito. Box<T> resolve isso porque um
// ponteiro tem tamanho fixo, independente do que ele aponta (o filho vai
// para o heap).
struct NoArvore {
    valor: i32,
    esquerda: Option<Box<NoArvore>>,
    direita: Option<Box<NoArvore>>,
}

impl NoArvore {
    fn folha(valor: i32) -> Self {
        NoArvore { valor, esquerda: None, direita: None }
    }

    fn soma(&self) -> i32 {
        let soma_esquerda = self.esquerda.as_ref().map_or(0, |n| n.soma());
        let soma_direita = self.direita.as_ref().map_or(0, |n| n.soma());
        self.valor + soma_esquerda + soma_direita
    }
}

fn main() {
    let arvore = NoArvore {
        valor: 10,
        esquerda: Some(Box::new(NoArvore::folha(5))),
        direita: Some(Box::new(NoArvore {
            valor: 20,
            esquerda: Some(Box::new(NoArvore::folha(3))),
            direita: None,
        })),
    };

    println!("Soma da arvore: {}", arvore.soma());

    // Rc<T>: multiplos "donos" do mesmo dado, com contagem de referencias.
    // So permite leitura compartilhada (para mutacao, precisaria de
    // Rc<RefCell<T>>, um passo alem do que vemos aqui).
    let categoria = Rc::new(String::from("Eletronicos"));

    println!("Contagem de referencias: {}", Rc::strong_count(&categoria));

    let categoria_produto1 = Rc::clone(&categoria);
    let categoria_produto2 = Rc::clone(&categoria);

    println!("Contagem apos 2 clones: {}", Rc::strong_count(&categoria));
    println!("Produto 1 na categoria: {categoria_produto1}");
    println!("Produto 2 na categoria: {categoria_produto2}");
}