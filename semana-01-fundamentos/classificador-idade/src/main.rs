fn classificar(idade: u32) -> &'static str {
    match idade {
        0..=12 => "crianca",
        13..=17 => "adolescente",
        18..=59 => "adulto",
        idade if idade >= 60 => "idoso",
        _ => "invalido",
    }
}

fn main() {
    let idades = [5, 15, 30, 65, 0];

    for idade in idades {
        println!("{idade} anos -> {}", classificar(idade));
    }
}