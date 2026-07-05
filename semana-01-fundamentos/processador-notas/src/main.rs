fn media(notas: &[f64]) -> f64 {
    notas.iter().sum::<f64>() / notas.len() as f64
}

fn aprovados(notas: &[f64], nota_minima: f64) -> Vec<f64> {
    notas
        .iter()
        .filter(|&&nota| nota >= nota_minima)
        .copied()
        .collect()
}

fn notas_com_bonus(notas: &[f64], bonus: f64) -> Vec<f64> {
    notas.iter().map(|nota| nota + bonus).collect()
}

fn main() {
    let notas = [5.5, 7.0, 3.2, 9.8, 6.0, 4.5];

    println!("Notas originais: {notas:?}");
    println!("Media: {:.2}", media(&notas));

    let aprovados_lista = aprovados(&notas, 6.0);
    println!("Aprovados (>= 6.0): {aprovados_lista:?}");

    let com_bonus = notas_com_bonus(&notas, 0.5);
    println!("Notas com bonus de 0.5: {com_bonus:?}");
}