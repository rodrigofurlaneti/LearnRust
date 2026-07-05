#[derive(Debug)]
enum ErroConta {
    ValorInvalido,
    SaldoInsuficiente,
}

struct Conta {
    saldo: f64,
}

impl Conta {
    fn nova() -> Self {
        Conta { saldo: 0.0 }
    }

    fn depositar(&mut self, valor: f64) -> Result<(), ErroConta> {
        if valor <= 0.0 {
            return Err(ErroConta::ValorInvalido);
        }

        self.saldo += valor;
        Ok(())
    }

    fn sacar(&mut self, valor: f64) -> Result<(), ErroConta> {
        if valor <= 0.0 {
            return Err(ErroConta::ValorInvalido);
        }

        if valor > self.saldo {
            return Err(ErroConta::SaldoInsuficiente);
        }

        self.saldo -= valor;
        Ok(())
    }

    fn saldo(&self) -> f64 {
        self.saldo
    }
}

fn main() {
    let mut conta = Conta::nova();

    let operacoes: Vec<(&str, f64)> = vec![
        ("deposito", 100.0),
        ("saque", 30.0),
        ("saque", 1000.0),
        ("deposito", -5.0),
    ];

    for (tipo, valor) in operacoes {
        let resultado = match tipo {
            "deposito" => conta.depositar(valor),
            "saque" => conta.sacar(valor),
            _ => unreachable!(),
        };

        match resultado {
            Ok(()) => println!("{tipo} de {valor:.2} realizado. Saldo atual: {:.2}", conta.saldo()),
            Err(erro) => println!("{tipo} de {valor:.2} falhou: {erro:?}"),
        }
    }
}