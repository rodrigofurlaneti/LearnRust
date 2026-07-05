fn calcular(a: f64, b: f64, operacao: impl Fn(f64, f64) -> f64) -> f64 {
    operacao(a, b)
}

fn main() {
    let soma = calcular(10.0, 5.0, |a, b| a + b);
    let subtracao = calcular(10.0, 5.0, |a, b| a - b);

    println!("Soma: {soma}");
    println!("Subtracao: {subtracao}");

    let taxa_desconto = 0.1;
    let aplicar_desconto = |valor: f64| valor - (valor * taxa_desconto);

    let precos = [100.0, 250.0, 80.0];

    for preco in precos {
        println!("Preco {preco:.2} com desconto: {:.2}", aplicar_desconto(preco));
    }
}