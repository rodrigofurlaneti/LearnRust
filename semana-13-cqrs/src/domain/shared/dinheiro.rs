use std::iter::Sum;
use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::domain::shared::errors::DomainError;
use crate::domain::shared::quantidade::Quantidade;

/// Value Object monetario (novo na Semana 11): guarda o valor SEMPRE em
/// centavos (`i64`), nunca em `f64`. Ponto de fundamentos que vale
/// repetir: numero de ponto flutuante nao representa exatamente valores
/// decimais como "19.90" - somar `f64` repetidas vezes acumula erro de
/// arredondamento, o tipo de bug que passa despercebido em teste manual e
/// aparece so em producao, com dinheiro de verdade. Guardando centavos
/// como inteiro, toda soma/multiplicacao e exata.
///
/// Implementa `Add` (mesma tecnica de `impl Add for Vetor2D` na Semana 7,
/// `operadores-vetor2d`) para que somar o valor de varios itens de um
/// pedido seja uma expressao natural (`item1.valor_total() +
/// item2.valor_total()`), e `Sum` para poder usar `.sum()` num iterator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Dinheiro(i64);

impl Dinheiro {
    pub fn zero() -> Self {
        Self(0)
    }

    /// Usado pela camada de infraestrutura (replay do log de auditoria,
    /// reidratacao a partir do cache) - o valor ja foi validado da
    /// primeira vez que foi escrito, entao so revalidamos o invariante
    /// "nunca negativo".
    pub fn from_centavos(centavos: i64) -> Result<Self, DomainError> {
        if centavos < 0 {
            return Err(DomainError::ValorMonetarioInvalido);
        }
        Ok(Self(centavos))
    }

    /// Usado pela presentation ao receber o preco de um produto como
    /// texto (`"19.90"`, `"19,90"`) - de proposito o DTO nunca usa `f64`
    /// para nao reintroduzir o problema que este VO existe para evitar.
    pub fn parse_reais(raw: &str) -> Result<Self, DomainError> {
        let normalized = raw.trim().replace(',', ".");
        let valor: f64 = normalized
            .parse()
            .map_err(|_| DomainError::ValorMonetarioInvalido)?;
        Self::from_reais(valor)
    }

    fn from_reais(valor: f64) -> Result<Self, DomainError> {
        if !valor.is_finite() || valor < 0.0 {
            return Err(DomainError::ValorMonetarioInvalido);
        }
        Self::from_centavos((valor * 100.0).round() as i64)
    }

    pub fn as_centavos(&self) -> i64 {
        self.0
    }

    pub fn as_reais_str(&self) -> String {
        format!("{}.{:02}", self.0 / 100, self.0 % 100)
    }

    /// Regra de negocio central do pedido (Semana 11): o valor total de um
    /// item e o valor unitario multiplicado pela quantidade, calculado no
    /// exato momento do pedido. Vive aqui (Tell, Don't Ask) em vez de
    /// quem chama fazer a conta na mao.
    pub fn multiplicar_por(&self, quantidade: Quantidade) -> Self {
        Self(self.0 * i64::from(quantidade.valor()))
    }
}

impl Add for Dinheiro {
    type Output = Dinheiro;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sum for Dinheiro {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Dinheiro::zero(), Add::add)
    }
}
