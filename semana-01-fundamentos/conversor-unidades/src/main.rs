enum Unidade {
    Celsius(f64),
    Fahrenheit(f64),
}

fn converter(u: Unidade) -> f64 {
    match u {
        Unidade::Celsius(c) => c * 9.0 / 5.0 + 32.0,
        Unidade::Fahrenheit(f) => (f - 32.0) * 5.0 / 9.0,
    }
}

fn imprimir_unidade(nome: &str) {
    println!("Unidade: {nome}");
}

fn unidade_mais_longa<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}

fn parse_temperatura(entrada: &str) -> Result<f64, String> {
    entrada
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("'{entrada}' nao e um numero valido"))
}

fn main() {
    println!("Hello, world!");

    let temp = Unidade::Celsius(100.0);
    let resultado = converter(temp);
    println!("Resultado: {resultado:.2}");

    let temp2 = Unidade::Fahrenheit(32.0);
    println!("Resultado 2: {:.2}", converter(temp2));

    let nome_unidade = String::from("Celsius");
    imprimir_unidade(&nome_unidade);
    imprimir_unidade(&nome_unidade);

    let maior = unidade_mais_longa("Celsius", "F");
    println!("Maior nome: {maior}");

    let entradas = ["36.5", "abc", "100"];
    for entrada in entradas {
        match parse_temperatura(entrada) {
            Ok(valor) => println!("Convertido com sucesso: {valor}"),
            Err(erro) => println!("Erro: {erro}"),
        }
    }
}