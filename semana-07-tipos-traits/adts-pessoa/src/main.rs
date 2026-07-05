struct DadosComuns {
    nome: String,
    email: String,
}

enum Pessoa {
    Fisica { dados: DadosComuns, cpf: String },
    Juridica { dados: DadosComuns, cnpj: String },
}

impl Pessoa {
    fn identificador(&self) -> &str {
        match self {
            Pessoa::Fisica { cpf, .. } => cpf,
            Pessoa::Juridica { cnpj, .. } => cnpj,
        }
    }

    fn nome(&self) -> &str {
        match self {
            Pessoa::Fisica { dados, .. } => &dados.nome,
            Pessoa::Juridica { dados, .. } => &dados.nome,
        }
    }

    fn email(&self) -> &str {
        match self {
            Pessoa::Fisica { dados, .. } => &dados.email,
            Pessoa::Juridica { dados, .. } => &dados.email,
        }
    }

    fn tipo(&self) -> &'static str {
        match self {
            Pessoa::Fisica { .. } => "Pessoa Fisica",
            Pessoa::Juridica { .. } => "Pessoa Juridica",
        }
    }
}

fn main() {
    let pessoas = vec![
        Pessoa::Fisica {
            dados: DadosComuns {
                nome: "Rodrigo".to_string(),
                email: "rodrigo@exemplo.com".to_string(),
            },
            cpf: "12345678901".to_string(),
        },
        Pessoa::Juridica {
            dados: DadosComuns {
                nome: "Empresa XYZ".to_string(),
                email: "contato@xyz.com".to_string(),
            },
            cnpj: "12345678000199".to_string(),
        },
    ];

    for pessoa in &pessoas {
        println!(
            "{} | {} | {} | identificador: {}",
            pessoa.tipo(),
            pessoa.nome(),
            pessoa.email(),
            pessoa.identificador()
        );
    }
}