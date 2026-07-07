# cadastro-api — CQRS leve (Command/Query + Bus) (Rust + Axum)

Semana 13: **nenhuma regra de negocio muda**. Mesmo comportamento da
`../semana-12-organizacao-modular` (mesmos endpoints, mesmo cache, mesma
auditoria/reidratacao) - o objetivo desta semana e formalizar uma
separacao que o codigo ja tinha implicitamente desde a Semana 9: casos de
uso que **mudam** estado (`register_*`, `update_*`, `delete_*`,
`authenticate_user`) de um lado, casos de uso que so **leem** estado
(`get_*`, `list_*`) do outro. Isso e CQRS (Command Query Responsibility
Segregation) na sua forma mais simples: sem bancos de leitura/escrita
separados, sem eventual consistency - so vocabulario explicito e um ponto
de entrada unico para cada lado.

## 1. Regra de dependencia (Clean Architecture) - inalterada

```
presentation  --->  application  --->  domain
                                          ^
infra  -------------------------------- / (implementa portas do dominio)
```

## 2. O vocabulario novo (`application/shared/cqrs.rs`)

```rust
pub trait Command { type Output; }
pub trait Query   { type Output; }

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Output, ApplicationError>;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Output, ApplicationError>;
}
```

- **Command**: uma intencao de mudar o estado do sistema (`RegisterClienteCommand`,
  `DeleteProdutoCommand` etc). So carrega dados, nao sabe se executar.
- **Query**: uma pergunta sobre o estado atual (`GetPedidoQuery`,
  `ListClientesQuery`). Nunca muda nada - rodar duas vezes seguidas nunca
  tem efeito colateral.
- **CommandHandler/QueryHandler**: quem sabe executar um Command/Query
  especifico. Cada caso de uso que ja existia (`RegisterCliente`,
  `GetPedido` etc.) ganhou um `impl CommandHandler<X>`/`impl
  QueryHandler<X>` que so delega para o `execute` que ja existia -
  **nenhuma logica de negocio foi reescrita**, so ganhou um nome.

## 3. Onde cada coisa mora

Cada agregado ganhou dois arquivos novos dentro de `application/<agregado>/`:

```
application/cliente/
├── commands.rs   RegisterClienteCommand, UpdateClienteCommand, DeleteClienteCommand
├── queries.rs    GetClienteQuery, ListClientesQuery
├── register_cliente.rs, update_cliente.rs, ...   (inalterados desde a Semana 9/10)
```

O mesmo padrao se repete em `usuario/`, `produto/` e `pedido/`. Exemplo
completo (`application/cliente/commands.rs`):

```rust
pub struct RegisterClienteCommand(pub RegisterClienteInput);

impl Command for RegisterClienteCommand {
    type Output = ClienteOutput;
}

#[async_trait]
impl CommandHandler<RegisterClienteCommand> for RegisterCliente {
    async fn handle(&self, command: RegisterClienteCommand) -> Result<ClienteOutput, ApplicationError> {
        self.execute(command.0).await   // delega pro que ja existia
    }
}
```

## 4. `CommandBus` e `QueryBus` (`application/shared/`)

O barramento e so uma colecao de `Arc<UseCase>` (as mesmas 12 dependencias
de escrita e 7 de leitura que o `AppState` ja carregava antes) com um
metodo generico `dispatch`:

```rust
pub async fn dispatch<C>(&self, command: C) -> Result<C::Output, ApplicationError>
where
    C: Command,
    Self: CommandHandler<C>,
{
    CommandHandler::handle(self, command).await
}
```

Quem chama nunca escreve `if command is RegisterCliente { ... } else if
...` - o compilador escolhe o `impl CommandHandler<C> for CommandBus`
certo a partir do tipo concreto de `C`, em tempo de compilacao. Ver o
comentario no topo de `command_bus.rs` para o porque disso (e nao um
registro dinamico via `TypeId`/`dyn Any`, como um Mediator faria em C#/Java).

## 5. O que a presentation ganhou

`AppState` foi de 19 campos para 3:

```rust
pub struct AppState {
    pub command_bus: Arc<CommandBus>,
    pub query_bus: Arc<QueryBus>,
    pub tokens: Arc<dyn TokenService>,   // extractor de JWT, fora dos barramentos
}
```

E os handlers HTTP trocaram "chame o caso de uso" por "despache o
Command/Query":

```rust
// Antes (Semana 12)
let output = state.register_cliente.execute(input).await?;

// Agora (Semana 13)
let output = state.command_bus.dispatch(RegisterClienteCommand(input)).await?;
```

## 6. Uma decisao de nomenclatura: login e Command, nao Query

`AuthenticateUserCommand` (login) nao muda o estado de nenhum agregado
`User`, mas foi modelado como Command, nao Query: e uma acao imperativa
("autentique-se") com efeitos que vao alem de so ler dados - emissao de
token, e o alvo do rate limiting da Semana 9. A regra pratica usada aqui:
se a operacao e "faca algo" (mesmo que o "algo" nao persista uma
mudanca), e Command; se e "me diga algo", e Query.

## 7. Por que nao foi ate o fim (CQRS "completo")

O CQRS classico costuma incluir Read Models proprios - estruturas
desnormalizadas, otimizadas pra consulta, atualizadas por um projector que
consome eventos (a auditoria em JSON lines desta API ja seria uma fonte
perfeita pra isso). Essa versao **nao foi implementada** nesta semana de
proposito: o objetivo era o vocabulario (Command/Query/Handler/Bus), nao
introduzir eventual consistency numa API que ainda serve tudo de um cache
em memoria unico. Fica como proximo passo natural (secao 9).

## 8. Como rodar

```bash
cd semana-13-cqrs
cargo run
```

Variaveis, endpoints e fluxo de exemplo: identicos as Semanas 11/12 (ver
README de la). Admin de exemplo: `admin@example.com` / `AdminForte123`.

## 9. Verificacao feita nesta semana

- `cargo build`/`cargo test` limpos - os mesmos 7 testes de integracao
  das Semanas 11/12 passam **sem nenhuma mudanca de asserção**, prova de
  que o `execute()` de cada caso de uso continua fazendo exatamente o que
  fazia antes (os testes chamam os casos de uso diretamente, sem passar
  pelo bus).
- `cargo clippy --all-targets` sem warnings novos - so o
  `derivable_impls` de `Role::default()`, ja aceito desde a Semana 9.

## 10. Proximos passos sugeridos

- Read Models desnormalizados projetados a partir do event log (ver
  secao 7) - o CQRS "completo".
- Validacao de Command/Query num middleware generico (hoje cada DTO valida
  a si mesmo via Value Objects no construtor do dominio - o padrao
  Command/Handler abriria espaco pra um pipeline de validacao/logging
  cross-cutting, tipo os `IPipelineBehavior` do MediatR em .NET).
- Paginacao/filtros nas Queries de listagem (`ListClientesQuery`,
  `ListProdutosQuery`, `ListPedidosQuery` sao unit structs hoje - o campo
  de filtro entraria ali, sem tocar no `CommandBus`/`QueryBus`).
