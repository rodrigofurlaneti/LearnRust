trait Forma {
    fn area(&self) -> f64;
    fn nome(&self) -> &'static str;
}

struct Circulo {
    raio: f64,
}

impl Forma for Circulo {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.raio * self.raio
    }

    fn nome(&self) -> &'static str {
        "Circulo"
    }
}

struct Retangulo {
    largura: f64,
    altura: f64,
}

impl Forma for Retangulo {
    fn area(&self) -> f64 {
        self.largura * self.altura
    }

    fn nome(&self) -> &'static str {
        "Retangulo"
    }
}

// Static dispatch: o compilador gera uma versao especializada desta funcao
// para cada tipo concreto usado (monomorphization). Zero custo em runtime,
// mas so aceita UM tipo concreto por chamada.
fn imprimir_area_estatico<T: Forma>(forma: &T) {
    println!("[estatico] {}: area = {:.2}", forma.nome(), forma.area());
}

// Dynamic dispatch: resolvido em runtime via vtable. Permite misturar
// tipos diferentes na mesma colecao, ao custo de uma indirecao.
fn imprimir_area_dinamico(forma: &dyn Forma) {
    println!("[dinamico] {}: area = {:.2}", forma.nome(), forma.area());
}

fn main() {
    let circulo = Circulo { raio: 3.0 };
    let retangulo = Retangulo { largura: 4.0, altura: 5.0 };

    imprimir_area_estatico(&circulo);
    imprimir_area_estatico(&retangulo);

    imprimir_area_dinamico(&circulo);
    imprimir_area_dinamico(&retangulo);

    // So e possivel misturar tipos diferentes na mesma Vec usando dyn Trait.
    // Com generics puros (static dispatch), isso nao compilaria.
    let formas: Vec<Box<dyn Forma>> = vec![
        Box::new(Circulo { raio: 1.0 }),
        Box::new(Retangulo { largura: 2.0, altura: 3.0 }),
    ];

    let area_total: f64 = formas.iter().map(|forma| forma.area()).sum();
    println!("Area total da colecao mista: {area_total:.2}");
}