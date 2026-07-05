use std::fmt;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vetor2D {
    x: f64,
    y: f64,
}

// Implementar Add habilita o operador "+" para este tipo.
impl Add for Vetor2D {
    type Output = Vetor2D;

    fn add(self, outro: Vetor2D) -> Vetor2D {
        Vetor2D {
            x: self.x + outro.x,
            y: self.y + outro.y,
        }
    }
}

// Implementar Display habilita "{v}" na formatacao, equivalente
// ao override de ToString() em C# (mas sem herdar de nenhuma classe base).
impl fmt::Display for Vetor2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let v1 = Vetor2D { x: 1.0, y: 2.0 };
    let v2 = Vetor2D { x: 3.0, y: 4.0 };

    let soma = v1 + v2;

    println!("v1 = {v1}");
    println!("v2 = {v2}");
    println!("v1 + v2 = {soma}");

    println!("v1 == v2? {}", v1 == v2);
    println!("v1 == v1? {}", v1 == v1);
}