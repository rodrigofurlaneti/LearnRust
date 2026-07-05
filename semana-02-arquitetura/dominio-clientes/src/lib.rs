pub struct Email(String);

impl Email {
    pub fn new(endereco: &str) -> Result<Self, String> {
        if endereco.contains('@') {
            Ok(Email(endereco.to_string()))
        } else {
            Err(format!("'{endereco}' nao e um email valido"))
        }
    }

    pub fn valor(&self) -> &str {
        &self.0
    }
}

pub struct Cpf(String);

impl Cpf {
    pub fn new(numero: &str) -> Result<Self, String> {
        let apenas_digitos: String = numero.chars().filter(|c| c.is_ascii_digit()).collect();

        if apenas_digitos.len() != 11 {
            Err(format!("'{numero}' nao e um cpf valido"))
        } else {
            Ok(Cpf(apenas_digitos))
        }
    }

    pub fn valor(&self) -> &str {
        &self.0
    }
}

pub struct Cliente {
    nome: String,
    email: Email,
    cpf: Cpf,
}

impl Cliente {
    pub fn novo(nome: &str, email: &str, cpf: &str) -> Result<Self, String> {
        Ok(Cliente {
            nome: nome.to_string(),
            email: Email::new(email)?,
            cpf: Cpf::new(cpf)?,
        })
    }

    pub fn nome(&self) -> &str {
        &self.nome
    }

    pub fn email(&self) -> &str {
        self.email.valor()
    }

    pub fn cpf(&self) -> &str {
        self.cpf.valor()
    }
}

// Repository generico: funciona para Cliente, Pedido, ou qualquer T.
pub trait Repositorio<T> {
    fn salvar(&mut self, item: T);
    fn buscar_todos(&self) -> &Vec<T>;
}

pub struct RepositorioEmMemoria<T> {
    itens: Vec<T>,
}

impl<T> RepositorioEmMemoria<T> {
    pub fn novo() -> Self {
        RepositorioEmMemoria { itens: Vec::new() }
    }
}

impl<T> Repositorio<T> for RepositorioEmMemoria<T> {
    fn salvar(&mut self, item: T) {
        self.itens.push(item);
    }

    fn buscar_todos(&self) -> &Vec<T> {
        &self.itens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cliente_com_dados_invalidos_falha() {
        assert!(Cliente::novo("Rodrigo", "email-invalido", "12345678901").is_err());
        assert!(Cliente::novo("Rodrigo", "rodrigo@exemplo.com", "123").is_err());
    }

    #[test]
    fn cliente_valido_e_criado_com_sucesso() {
        let cliente = Cliente::novo("Rodrigo", "rodrigo@exemplo.com", "123.456.789-01").unwrap();
        assert_eq!(cliente.nome(), "Rodrigo");
        assert_eq!(cliente.cpf(), "12345678901");
    }

    #[test]
    fn repositorio_generico_funciona_com_cliente() {
        let mut repositorio: RepositorioEmMemoria<Cliente> = RepositorioEmMemoria::novo();
        let cliente = Cliente::novo("Rodrigo", "rodrigo@exemplo.com", "12345678901").unwrap();

        repositorio.salvar(cliente);

        assert_eq!(repositorio.buscar_todos().len(), 1);
        assert_eq!(repositorio.buscar_todos()[0].email(), "rodrigo@exemplo.com");
    }
}