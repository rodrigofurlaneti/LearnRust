use crate::domain::shared::errors::DomainError;

const TAMANHO_CPF: usize = 11;

/// Value Object: CPF (documento do cliente brasileiro). So existe se tiver
/// 11 digitos, nao for uma sequencia de digitos repetidos (ex.:
/// "111.111.111-11", classicamente invalida apesar de "parecer" um numero)
/// e os dois digitos verificadores baterem com o algoritmo oficial da
/// Receita Federal. Mesma filosofia de `Email`/`Nome`: nenhuma
/// `String` solta representa um documento dentro do sistema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cpf(String);

impl Cpf {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let digits = Self::only_digits(raw);
        if Self::is_invalid(&digits) {
            return Err(DomainError::InvalidDocument);
        }
        Ok(Self(digits))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn only_digits(raw: &str) -> String {
        raw.chars().filter(|c| c.is_ascii_digit()).collect()
    }

    fn is_invalid(digits: &str) -> bool {
        !Self::has_correct_length(digits)
            || Self::has_all_repeated_digits(digits)
            || !Self::has_valid_check_digits(digits)
    }

    fn has_correct_length(digits: &str) -> bool {
        digits.chars().count() == TAMANHO_CPF
    }

    fn has_all_repeated_digits(digits: &str) -> bool {
        let first_digit = digits.chars().next();
        digits.chars().all(|digit| Some(digit) == first_digit)
    }

    fn has_valid_check_digits(digits: &str) -> bool {
        let numbers = Self::to_numbers(digits);
        let first_check_digit = Self::check_digit(&numbers[0..9], 10);
        let second_check_digit = Self::check_digit(&numbers[0..10], 11);

        numbers[9] == first_check_digit && numbers[10] == second_check_digit
    }

    fn to_numbers(digits: &str) -> Vec<u32> {
        digits.chars().filter_map(|c| c.to_digit(10)).collect()
    }

    /// Algoritmo oficial do digito verificador do CPF: soma ponderada dos
    /// digitos anteriores (pesos decrescentes a partir de `start_weight`),
    /// resto da divisao por 11, e o "resto 10" vira 0 por convencao.
    fn check_digit(base_digits: &[u32], start_weight: u32) -> u32 {
        let weighted_sum: u32 = base_digits
            .iter()
            .enumerate()
            .map(|(index, digit)| digit * (start_weight - index as u32))
            .sum();

        let remainder = (weighted_sum * 10) % 11;
        if remainder == 10 {
            0
        } else {
            remainder
        }
    }
}
