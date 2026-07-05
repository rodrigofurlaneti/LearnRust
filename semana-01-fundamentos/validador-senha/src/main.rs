#[derive(Debug)]
enum ErroSenha {
    MuitoCurta,
    SemNumero,
    SemLetraMaiuscula,
    SemCaractereEspecial,
}

fn validar_senha(senha: &str) -> Result<(), Vec<ErroSenha>> {
    let mut erros = Vec::new();

    if senha.len() < 8 {
        erros.push(ErroSenha::MuitoCurta);
    }

    if !senha.chars().any(|c| c.is_ascii_digit()) {
        erros.push(ErroSenha::SemNumero);
    }

    if !senha.chars().any(|c| c.is_ascii_uppercase()) {
        erros.push(ErroSenha::SemLetraMaiuscula);
    }

    if !senha.chars().any(|c| !c.is_alphanumeric()) {
        erros.push(ErroSenha::SemCaractereEspecial);
    }

    if erros.is_empty() {
        Ok(())
    } else {
        Err(erros)
    }
}

fn main() {
    let senhas = ["123", "senha123", "Senha123", "Senha123!"];

    for senha in senhas {
        match validar_senha(senha) {
            Ok(()) => println!("'{senha}' -> valida"),
            Err(erros) => println!("'{senha}' -> invalida: {erros:?}"),
        }
    }
}