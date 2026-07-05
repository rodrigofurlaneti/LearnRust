# auth-api — API de autenticação em DDD (Rust + Axum + PostgreSQL)

Projeto de exemplo para a Semana 8: uma API de registro/login estruturada em
Domain-Driven Design, com Clean Architecture, princípios SOLID e Object
Calisthenics aplicados de forma pragmática.

## 1. Regra de dependência (Clean Architecture)

```
presentation  --->  application  --->  domain
                                          ^
infra  -------------------------------- / (implementa portas do domínio)
```

- **domain**: regras de negócio puras. Não importa `axum`, `sqlx`, `bcrypt`
  nem `jsonwebtoken`. Só depende de `std` + tipos utilitários (`uuid`,
  `async-trait`).
- **application**: orquestra casos de uso, falando só com abstrações
  (`traits`) do domínio.
- **infra**: implementações concretas dessas abstrações (Postgres, bcrypt,
  JWT). É a única camada que conhece bibliotecas externas de I/O.
- **presentation**: converte HTTP <-> DTOs de aplicação. É a única camada
  que conhece `axum`/`StatusCode`.

Ninguém depende de infra. Infra depende de domain/application (via `mod.rs`
do binário), nunca o contrário — isso é o Dependency Inversion Principle
(o "D" do SOLID) na prática.

## 2. Passo a passo de como o projeto foi montado

### Passo 1 — Domain: Value Objects e regra de negócio

- `domain/email.rs`, `domain/password.rs`, `domain/user_id.rs`: primitivos
  (`String`, `Uuid`) nunca circulam soltos pelo sistema — cada um vira um
  Value Object que só existe se for válido (`Email::parse`,
  `PlainPassword::parse`). Isso é a regra de Object Calisthenics "wrap all
  primitives" e também elimina uma classe inteira de bugs de validação.
- `domain/user.rs`: a entidade `User`. A regra de negócio "esta senha
  pertence a este usuário?" mora dentro do próprio `User::matches_password`
  (Tell, Don't Ask) — ninguém de fora pega o hash e compara na mão.
- `domain/password_hasher.rs` e `domain/repository.rs`: as **portas**
  (`trait PasswordHasher`, `trait UserRepository`). O domínio declara *o
  que* precisa, nunca *como* é feito.
- `domain/errors.rs`: `DomainError` é o vocabulário único de erro de
  negócio (`InvalidEmail`, `WeakPassword`, `UserAlreadyExists`,
  `InvalidCredentials`).

### Passo 2 — Application: casos de uso

- `application/register_user.rs` e `application/authenticate_user.rs`: uma
  classe por caso de uso (SRP). Cada uma recebe as portas do domínio via
  `Arc<dyn Trait>` no construtor (Dependency Injection manual, sem
  framework de DI).
- Cada método `execute` tem no máximo um nível de indentação por bloco
  lógico — passos auxiliares viram métodos privados
  (`ensure_email_is_available`, `find_registered_user`, etc.), outra regra
  de Object Calisthenics.
- `application/dto.rs`: os DTOs de entrada/saída dos casos de uso — a
  "linguagem" da aplicação, independente de JSON/HTTP.
- `application/token_service.rs`: mais uma porta (`trait TokenService`),
  porque emitir token também é um detalhe de infraestrutura (JWT hoje,
  poderia ser sessão opaca amanhã).

### Passo 3 — Infra: adapters concretos

- `infra/postgres_user_repository.rs` implementa `UserRepository` com
  SQLx (queries via `sqlx::query`, sem macros `query!` para não exigir
  banco vivo em tempo de compilação).
- `infra/bcrypt_hasher.rs` implementa `PasswordHasher` com bcrypt,
  rodando o hashing em `spawn_blocking` (é CPU-bound e não pode travar o
  runtime assíncrono).
- `infra/jwt_token_service.rs` implementa `TokenService` com
  `jsonwebtoken`, assinando um JWT HS256 com `sub` (id do usuário) e `exp`.
- `infra/sqlite_user_repository.rs` implementa o **mesmo** `UserRepository`,
  mas em cima de um SQLite totalmente em memória — ver seção 5.
- `infra/db.rs`: criação dos pools de conexão (Postgres e SQLite em
  memória).

### Passo 4 — Presentation: HTTP

- `presentation/routes.rs`: mapeia `POST /auth/register` e
  `POST /auth/login`.
- `presentation/handlers.rs`: handlers "burros" — só extraem o `Json`,
  chamam o caso de uso e devolvem o resultado. Zero regra de negócio aqui.
- `presentation/error_response.rs`: traduz `DomainError`/`ApplicationError`
  em `StatusCode` HTTP (400, 401, 409, 500).
- `presentation/state.rs`: `AppState` guardado pelo Axum, com os casos de
  uso já prontos (injetados no `main.rs`).

### Passo 5 — main.rs: composition root

Único lugar do projeto que conhece **todas** as camadas ao mesmo tempo.
Aqui é onde os adapters concretos (Postgres, bcrypt, JWT) são criados e
"encaixados" nas portas que application/domain esperam.

## 3. SOLID aplicado

- **S**RP: cada caso de uso, cada Value Object e cada adapter tem uma única
  responsabilidade.
- **O**CP: novos métodos de hashing ou de emissão de token entram como
  novos adapters, sem tocar em `application`/`domain`.
- **L**SP: qualquer implementação de `UserRepository`/`PasswordHasher`/
  `TokenService` pode substituir outra sem quebrar os casos de uso.
- **I**SP: as portas são pequenas e focadas (`UserRepository` só fala de
  persistência de usuário, `TokenService` só emite token).
- **D**IP: `application` e `domain` dependem de `trait`s, não de
  `sqlx`/`bcrypt`/`jsonwebtoken` diretamente — essas libs só aparecem em
  `infra`.

## 4. Object Calisthenics — o que foi seguido e o que foi flexibilizado

Seguidos à risca: um nível de indentação por método, não abreviar nomes,
encapsular primitivos em Value Objects, entidades pequenas e focadas,
evitar `else` (uso extensivo de early return via `?` e `if ... return`).

Flexibilizado conscientemente: a regra "no máximo 2 variáveis de instância
por classe" não foi aplicada ao pé da letra em `User` (3 campos: id, email,
password) nem em `AuthenticateUser` (3 dependências). Essa é considerada a
mais extrema das nove regras — aplicá-la literalmente exigiria criar
wrappers artificiais só para "enganar" a contagem, o que pioraria a
legibilidade sem ganho real de design.

## 5. Banco em memória (SQLite), criado quando a aplicação sobe

Por padrão o projeto **não precisa de Docker nem de Postgres** para rodar.
`main.rs` lê a variável `DATABASE_BACKEND` (padrão: `memory`) e decide qual
adapter de `UserRepository` instanciar:

```rust
// main.rs
let backend = std::env::var("DATABASE_BACKEND").unwrap_or_else(|_| "memory".to_string());

match backend.as_str() {
    "postgres" => /* PostgresUserRepository, exige DATABASE_URL */,
    "memory"   => /* SqliteUserRepository, sqlite::memory: */,
    other      => panic!("DATABASE_BACKEND inválido: {other}"),
}
```

`infra/db.rs::create_in_memory_sqlite_pool` cria o pool **e já executa o
`CREATE TABLE IF NOT EXISTS users (...)`** nesse momento — o schema nasce
junto com o processo, não existe passo de migration separado. Ao encerrar
a aplicação, o banco inteiro desaparece.

Detalhe importante: o pool é criado com `max_connections(1)`. O SQLite em
modo `:memory:` cria um banco novo e isolado por conexão; com uma única
conexão viva durante toda a vida do pool, garantimos que toda a aplicação
(ou um teste) sempre fale com o mesmo banco.

Isso é possível **sem tocar em domain nem em application** porque ambos só
conhecem o trait `UserRepository` — trocar o adapter concreto é só decisão
do composition root (`main.rs`). Esse é o Open/Closed Principle e o
Dependency Inversion Principle funcionando na prática, não só na teoria.

Rodando sem nada configurado:

```bash
cargo run
# DATABASE_BACKEND=memory (padrão) -> nasce um SQLite em memória
```

Rodando com Postgres de verdade:

```bash
docker compose up -d
psql postgres://postgres:postgres@localhost:5432/auth_api -f migrations/0001_create_users.sql
cp .env.example .env   # ajuste DATABASE_BACKEND=postgres no .env
cargo run
```

Testando (funciona com qualquer um dos dois backends):

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"rodrigo@example.com","password":"SenhaForte123"}'

curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"rodrigo@example.com","password":"SenhaForte123"}'
```

## 6. Testes automatizados

`tests/auth_flow.rs` é um teste de integração que sobe `RegisterUser` e
`AuthenticateUser` de verdade, na frente de um `SqliteUserRepository` em
memória — cada teste cria o seu próprio banco, isolado dos outros, sem
precisar de Docker nem de mocks de domínio:

```bash
cargo test
```

Para isso funcionar, o projeto virou lib + bin (`src/lib.rs` expõe os
módulos; `src/main.rs` só faz `use auth_api::...`), padrão comum em Rust
para permitir testes de integração de fora do crate.

## 7. Documentação interativa (Swagger)

O equivalente ao Swagger/Swashbuckle do .NET no ecossistema Axum é a dupla
[`utoipa`](https://docs.rs/utoipa) + `utoipa-swagger-ui`. Com a API
rodando (`cargo run`), acesse:

```
http://localhost:3000/swagger-ui/
```

para testar `/auth/register` e `/auth/login` direto do navegador, com
formulário, exemplos e respostas documentadas. O JSON puro do OpenAPI fica
em `http://localhost:3000/api-docs/openapi.json`.

Como foi ligado:

- `application/dto.rs`: os DTOs (`RegisterUserInput`, `AuthenticateUserOutput`
  etc.) ganharam `#[derive(utoipa::ToSchema)]`. Trade-off documentado: o
  "puro" em Clean Architecture seria a presentation ter suas próprias
  structs de schema, mas isso duplicaria os campos e arriscaria a
  documentação ficar dessincronizada da API real — preferimos aceitar essa
  dependência pequena (mesma categoria do `serde`, que a aplicação já usa).
- `presentation/handlers.rs`: cada handler ganhou um `#[utoipa::path(...)]`
  descrevendo método, request body e possíveis respostas (200/400/401/409).
  É só metadado lido em tempo de compilação — não muda o comportamento.
- `presentation/openapi.rs`: `ApiDoc`, o `#[derive(OpenApi)]` que agrega
  todos os paths e schemas num único documento.
- `presentation/routes.rs`: `SwaggerUi::new("/swagger-ui").url(...)` é
  mesclado ao router de produção.

Versões usadas (compatibilidade com axum 0.7): `utoipa = "5"` e
`utoipa-swagger-ui = "8.1"` — a partir da 8.1.1 o `utoipa-swagger-ui` passa
a exigir axum 0.8.

## 8. Próximos passos sugeridos (semanas seguintes)

- Middleware de autorização (extrator Axum que valida o JWT e injeta o
  `UserId` autenticado nas rotas protegidas).
- Refresh token e revogação (blacklist em Redis).
- Migrar `sqlx::query` para `sqlx::query!` com verificação em tempo de
  compilação, usando `cargo sqlx prepare` para modo offline em CI.
