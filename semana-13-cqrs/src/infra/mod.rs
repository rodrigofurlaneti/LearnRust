//! Camada de infraestrutura, organizada por agregado (Semana 12): cada
//! repositorio de cache e seu decorator de auditoria moram no submodulo do
//! agregado correspondente; `shared` guarda o que atravessa todos eles.

pub mod cliente;
pub mod pedido;
pub mod produto;
pub mod shared;
pub mod usuario;
