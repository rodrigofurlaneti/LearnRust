# auth-api — Autorização e Middleware (Rust + Axum + PostgreSQL)

Semana 9: parte inteira da Semana 8 (API de registro/login em DDD) foi
copiada para esta pasta e evoluída com autenticação por JWT em rotas
protegidas, RBAC (papéis de usuário) e rate limiting. Compare as duas
pastas (`../semana-08-api` vs esta) para ver exatamente o diff entre "só
emitir um token" e "usar esse token para proteger e restringir rotas".

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
  que conhece `axum`/`StatusCode`, e agora também os extractors de
  autenticação/autorização (seção 9).

Ninguém depende de infra. Infra depende de domain/application (via `mod.rs`
do binário), nunca o contrário — isso é o Dependency Inversion Principle
(o "D" do SOLID) na prática.

## 2. Passo a passo de como o projeto foi montado (herdado da Semana 8)

### Passo 1 — Domain: Value Objects e regra de negócio

- `domain/email.rs`, `domain/password.rs`, `domain/user_id.rs`: primitivos
  (`String`, `Uuid`) nunca circulam soltos pelo sistema — cada um vira um
  Value Object que só existe se for válido (`Email::parse`,
  `PlainPassword::parse`). Isso é a regra de Object Calisthenics "wrap all
  primitives" e também elimina uma classe inteira de bugs de validação.
- `domain/role.rs` **(novo na Semana 9)**: enum fechado `Role::User` /
  `Role::Admin` — mesma filosofia de "wrap primitives" aplicada a papéis.
- `domain/user.rs`: a entidade `User`, agora com 4 campos (`id`, `email`,
  `password`, `role`). A regra de negócio "esta senha pertence a este
  usuário?" mora dentro do próprio `User::matches_password` (Tell, Don't
  Ask) — ninguém de fora pega o hash e compara na mão. Regra de negócio
  nova: `User::register` (fluxo público) **nunca** aceita `Role` como
  parâmetro — só cria `Role::User`. Quem precisa de outro papel usa
  `User::register_with_role`, um construtor separado e explícito. Isso
  torna estruturalmente impossível alguém virar admin só chamando o
  endpoint público de cadastro.
- `domain/password_hasher.rs` e `domain/repository.rs`: as **portas**
  (`trait PasswordHasher`, `trait UserRepository`). O domínio declara *o
  que* precisa, nunca *como* é feito.
- `domain/errors.rs`: `DomainError` é o vocabulário único de erro de
  negócio (`InvalidEmail`, `WeakPassword`, `UserAlreadyExists`,
  `InvalidCredentials`, e agora `PermissionDenied` para RBAC).

### Passo 2 — Application: casos de uso

- `application/register_user.rs` e `application/authenticate_user.rs`: uma
  classe por caso de uso (SRP). Cada uma recebe as portas do domínio via
  `Arc<dyn Trait>` no construtor (Dependency Injection manual, sem
  framework de DI).
- `application/get_current_user.rs` **(novo)**: caso de uso por trás de
  `GET /me`. Recebe um `UserId` já validado pela presentation e busca os
  dados atuais no repositório — nunca confia só no conteúdo do token.
- Cada método `execute` tem no máximo um nível de indentação por bloco
  lógico — passos auxiliares viram métodos privados, outra regra de Object
  Calisthenics.
- `application/dto.rs`: os DTOs de entrada/saída dos casos de uso, agora
  incluindo `role` em `RegisterUserOutput` e o novo `CurrentUserOutput`.
- `application/token_service.rs`: a porta `trait TokenService` ganhou um
  segundo método, `verify`, além do `issue` que já existia — ver seção 9.

### Passo 3 — Infra: adapters concretos

- `infra/postgres_user_repository.rs` e `infra/sqlite_user_repository.rs`
  implementam `UserRepository` (Postgres via SQLx, e SQLite em memória
  para dev/testes) — ambos agora persistem/leem a coluna `role`.
- `infra/bcrypt_hasher.rs` implementa `PasswordHasher` com bcrypt,
  rodando o hashing em `spawn_blocking` (é CPU-bound e não pode travar o
  runtime assíncrono).
- `infra/jwt_token_service.rs` implementa `TokenService`: `issue` inclui a
  `role` como claim do JWT, e o novo `verify` decodifica e valida um token
  recebido, devolvendo `TokenClaims { user_id, role }`.
- `infra/db.rs`: criação dos pools de conexão. O schema do SQLite em
  memória ganhou a coluna `role TEXT NOT NULL DEFAULT 'user'`.
- `migrations/0002_add_role_to_users.sql` **(novo)**: migration separada
  (não editamos a `0001` original) adicionando a coluna `role` no Postgres
  — prática correta de evolução de schema em produção.

### Passo 4 — Presentation: HTTP

- `presentation/routes.rs`: mapeia `POST /auth/register`, `POST /auth/login`
  (com rate limiting, seção 11), `GET /me` e `GET /admin/ping`.
- `presentation/auth_extractor.rs` **(novo)**: `AuthenticatedUser` e
  `AdminUser` — ver seção 9.
- `presentation/handlers.rs`: handlers "burros" — só extraem o `Json`/
  extractor, chamam o caso de uso e devolvem o resultado. Zero regra de
  negócio aqui.
- `presentation/error_response.rs`: traduz `DomainError`/`ApplicationError`
  em `StatusCode` HTTP (400, 401, 403, 409, 429, 500).
- `presentation/state.rs`: `AppState` guardado pelo Axum, agora também
  expondo `tokens: Arc<dyn TokenService>` (usado pelo extractor).

### Passo 5 — main.rs: composition root

Único lugar do projeto que conhece **todas** as camadas ao mesmo tempo.
Aqui é onde os adapters concretos (Postgres/SQLite, bcrypt, JWT) são
criados e "encaixados" nas portas que application/domain esperam, e onde
(só no backend `memory`) é semeado um usuário admin de exemplo — seção 10.

## 3. SOLID aplicado

- **S**RP: cada caso de uso, cada Value Object e cada adapter tem uma única
  responsabilidade. `AuthenticatedUser` só autentica; `AdminUser` só
  acrescenta a checagem de papel por cima, sem duplicar a validação do
  token.
- **O**CP: novos métodos de hashing, emissão de token ou novos papéis
  entram como novos adapters/variantes, sem tocar em `application`/`domain`.
- **L**SP: qualquer implementação de `UserRepository`/`PasswordHasher`/
  `TokenService` pode substituir outra sem quebrar os casos de uso.
- **I**SP: as portas são pequenas e focadas (`UserRepository` só fala de
  persistência de usuário, `TokenService` só emite/verifica token).
- **D**IP: `application` e `domain` dependem de `trait`s, não de
  `sqlx`/`bcrypt`/`jsonwebtoken` diretamente — essas libs só aparecem em
  `infra`.

## 4. Object Calisthenics — o que foi seguido e o que foi flexibilizado

Seguidos à risca: um nível de indentação por método, não abreviar nomes,
encapsular primitivos em Value Objects (incluindo o novo `Role`), entidades
pequenas e focadas, evitar `else` (early return via `?` e `if ... return`).

Flexibilizado conscientemente: a regra "no máximo 2 variáveis de instância
por classe" não foi aplicada ao pé da letra em `User` (agora 4 campos:
id, email, password, role) nem em `AuthenticateUser` (3 dependências).
Essa é considerada a mais extrema das nove regras — aplicá-la
literalmente exigiria criar wrappers artificiais só para "enganar" a
contagem, o que pioraria a legibilidade sem ganho real de design.

## 5. Banco em memória (SQLite), criado quando a aplicação sobe

Por padrão o projeto **não precisa de Docker nem de Postgres** para rodar.
`main.rs` lê a variável `DATABASE_BACKEND` (padrão: `memory`) e decide qual
adapter de `UserRepository` instanciar. `infra/db.rs::create_in_memory_sqlite_pool`
cria o pool **e já executa o `CREATE TABLE IF NOT EXISTS users (...)`**
nesse momento — o schema nasce junto com o processo. Detalhe importante: o
pool usa `max_connections(1)` porque o SQLite em modo `:memory:` cria um
banco novo por conexão; com uma única conexão viva, garantimos que toda a
aplicação (ou teste) fale com o mesmo banco.

Rodando sem nada configurado:

```bash
cargo run
# DATABASE_BACKEND=memory (padrão) -> nasce um SQLite em memória,
# incluindo um usuario admin de exemplo (ver secao 10)
```

Rodando com Postgres de verdade:

```bash
docker compose up -d
psql postgres://postgres:postgres@localhost:5432/auth_api -f migrations/0001_create_users.sql
psql postgres://postgres:postgres@localhost:5432/auth_api -f migrations/0002_add_role_to_users.sql
cp .env.example .env   # ajuste DATABASE_BACKEND=postgres no .env
cargo run
```

## 6. Testes automatizados

`tests/auth_flow.rs` sobe os casos de uso reais na frente de um
`SqliteUserRepository` em memória — cada teste cria seu próprio banco,
isolado dos outros. Além dos testes herdados da Semana 8, esta pasta
ganhou:

- `usuario_registrado_publicamente_nunca_recebe_role_admin` — trava a
  regra de negócio central desta semana.
- `token_emitido_no_login_carrega_a_role_correta_e_bate_no_verify` — testa
  o roundtrip `issue`/`verify` do `TokenService`.
- `get_current_user_retorna_os_dados_de_quem_esta_logado` — testa o caso
  de uso por trás de `GET /me`.

```bash
cargo test
```

## 7. Documentação interativa (Swagger)

Com a API rodando, acesse `http://localhost:3000/swagger-ui/`. Além de
`/auth/register` e `/auth/login` (herdados da Semana 8), agora aparecem
`GET /me` e `GET /admin/ping`, ambos marcados como protegidos por
`security(("bearer_auth" = []))`. Clique em **Authorize** no topo da
página, cole o `access_token` recebido no login, e o Swagger passa a
mandar o header `Authorization: Bearer ...` automaticamente nas
requisições de teste.

O esquema de segurança é registrado em `presentation/openapi.rs` via um
`impl Modify for SecurityAddon`, que injeta o `SecurityScheme::Http` do
tipo Bearer/JWT nos `components` do documento OpenAPI.

## 8. Rodando o fluxo completo (exemplo)

```bash
# 1. cadastro (sempre cria Role::User)
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"rodrigo@example.com","password":"SenhaForte123"}'

# 2. login -> pega o access_token da resposta
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"rodrigo@example.com","password":"SenhaForte123"}'

# 3. rota protegida, com o token do passo 2
curl http://localhost:3000/me \
  -H "Authorization: Bearer <token-aqui>"

# 4. sem token -> 401
curl -i http://localhost:3000/me

# 5. usuario comum tentando rota de admin -> 403
curl -i http://localhost:3000/admin/ping \
  -H "Authorization: Bearer <token-de-usuario-comum>"
```

## 9. Autorização e middleware (o tema da semana)

Modelado como **extractor do Axum** em vez de um `tower::Layer` solto —
fica explícito na assinatura de cada handler quais rotas exigem
autenticação, sem precisar ler a configuração de rotas para descobrir:

```rust
pub async fn me(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,   // <- se chegou aqui, o token é válido
) -> Result<Json<CurrentUserOutput>, ApiError> { ... }
```

`AuthenticatedUser` implementa `FromRequestParts<AppState>`
(`presentation/auth_extractor.rs`): lê o header `Authorization`, extrai o
`Bearer <token>`, chama `state.tokens.verify(token)` (a mesma porta
`TokenService` usada para emitir tokens no login, agora também para
verificá-los) e, se tudo bater, injeta `user_id`/`role` direto no handler.
Se o token faltar, for inválido ou tiver expirado, o Axum nem chega a
executar o handler — responde 401 antes disso.

## 10. RBAC com `Role`

`AdminUser` (mesmo arquivo) compõe sobre `AuthenticatedUser`: primeiro
valida o token exatamente da mesma forma, depois checa
`role == Role::Admin`, devolvendo 403 (`DomainError::PermissionDenied`) se
não for. `GET /admin/ping` usa esse guard como exemplo.

Como não existe nenhuma forma pública de criar um usuário Admin (ver
`User::register` vs `User::register_with_role` na seção 2), o backend
`memory` semeia automaticamente, na subida da aplicação, um admin de
exemplo — **só nesse backend, nunca em Postgres/produção**:

```
email: admin@example.com
senha: AdminForte123
```

Veja `main.rs::seed_default_admin`. Isso reaproveita a ideia da Semana 8
("banco criado quando a aplicação sobe") para também resolver "como eu
testo uma rota de admin sem ferramenta externa nenhuma".

## 11. Rate limiting no login

`POST /auth/login` é a rota mais sensível a força bruta, então ganhou uma
camada extra que as outras não têm — `tower_governor`, configurado em
`presentation/routes.rs`:

```rust
GovernorConfigBuilder::default()
    .per_second(1)
    .burst_size(5)
    .finish()
```

Em média 1 tentativa por segundo, com rajada de até 5 — o suficiente para
um usuário real errar a senha algumas vezes, não para um ataque
automatizado. A 6ª requisição numa rajada responde `429 Too Many
Requests`. Para o `tower_governor` conseguir identificar "de onde" vem
cada requisição, `main.rs` roda o servidor com
`into_make_service_with_connect_info::<SocketAddr>()` em vez do
`into_make_service()` simples.

## 12. Próximos passos sugeridos

- Refresh token + revogação (blacklist em Redis) — closes o ciclo de vida
  completo do token, hoje ele só expira, nunca é revogado antes disso.
- Endpoint de gestão de usuários (promover/rebaixar `Role`), restrito a
  `AdminUser` — hoje a única forma de virar admin é o seed automático.
- Observabilidade: `tracing` estruturado por requisição (span com
  `user_id`/`role` quando autenticado), health checks, Dockerfile
  multi-stage e CI (candidato natural para a Semana 10).
- Migrar `sqlx::query` para `sqlx::query!` com verificação em tempo de
  compilação, usando `cargo sqlx prepare` para modo offline em CI.
