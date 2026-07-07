# cadastro-api — Organizacao Modular por Agregado (Rust + Axum)

Semana 12: **nenhuma regra de negocio muda**. O codigo e identico em
comportamento ao da `../semana-11-produtos-pedidos` (mesmos endpoints,
mesmo cache, mesma auditoria/reidratacao) - o unico objetivo desta semana
e reorganizar os arquivos dentro de cada camada em subpastas por
**agregado**, um padrao comum em projetos DDD que crescem e comecam a
ficar dificeis de navegar quando cada camada e uma pasta unica e achatada
com dezenas de arquivos misturados (`user.rs`, `cliente.rs`, `produto.rs`,
`pedido.rs`, `dto.rs`... todos lado a lado).

## 1. Regra de dependencia (Clean Architecture) - inalterada

```
presentation  --->  application  --->  domain
                                          ^
infra  -------------------------------- / (implementa portas do dominio)
```

Isso nao muda: a reorganizacao acontece **dentro** de cada camada, nunca
entre elas. `domain` continua sem importar nada de `infra`/`presentation`.

## 2. Antes x depois

Ate a Semana 11, cada camada era uma pasta unica:

```
src/domain/
├── user.rs, user_id.rs, role.rs, password.rs, password_hasher.rs, repository.rs
├── cliente.rs, cliente_id.rs, cliente_repository.rs, cpf.rs
├── produto.rs, produto_id.rs, produto_repository.rs
├── pedido.rs, pedido_id.rs, pedido_repository.rs, item_pedido.rs
└── dinheiro.rs, email.rs, nome.rs, quantidade.rs, errors.rs
```

Com 4 agregados (Usuario, Cliente, Produto, Pedido) x 4 camadas, achar
"tudo que pertence a Pedido" exigia varrer visualmente cada uma das 4
pastas. A partir desta semana, cada camada ganha uma subpasta por
agregado, mais uma pasta `shared/` para o que atravessa mais de um:

```
src/
├── domain/
│   ├── usuario/   {user.rs, user_id.rs, role.rs, password.rs, password_hasher.rs, user_repository.rs}
│   ├── cliente/   {entidade.rs, cliente_id.rs, cliente_repository.rs, cpf.rs}
│   ├── produto/   {entidade.rs, produto_id.rs, produto_repository.rs}
│   ├── pedido/    {entidade.rs, pedido_id.rs, pedido_repository.rs, item_pedido.rs}
│   └── shared/    {dinheiro.rs, email.rs, nome.rs, quantidade.rs, errors.rs}
├── application/
│   ├── usuario/   {register_user.rs, update_user.rs, delete_user.rs, authenticate_user.rs, get_current_user.rs}
│   ├── cliente/   {register_cliente.rs, get_cliente.rs, list_clientes.rs, update_cliente.rs, delete_cliente.rs}
│   ├── produto/   {register_produto.rs, get_produto.rs, list_produtos.rs, update_produto.rs, delete_produto.rs}
│   ├── pedido/    {register_pedido.rs, get_pedido.rs, list_pedidos.rs, delete_pedido.rs}
│   └── shared/    {audit_logger.rs, dto.rs, errors.rs, token_service.rs}
├── infra/
│   ├── usuario/   {cache_user_repository.rs, audited_user_repository.rs, bcrypt_hasher.rs, jwt_token_service.rs}
│   ├── cliente/   {cache_cliente_repository.rs, audited_cliente_repository.rs}
│   ├── produto/   {cache_produto_repository.rs, audited_produto_repository.rs}
│   ├── pedido/    {cache_pedido_repository.rs, audited_pedido_repository.rs}
│   └── shared/    {file_audit_logger.rs, audit_log_replayer.rs}
└── presentation/
    ├── usuario/   {auth_handlers.rs, user_handlers.rs}
    ├── cliente/   {cliente_handlers.rs}
    ├── produto/   {produto_handlers.rs}
    ├── pedido/    {pedido_handlers.rs}
    └── shared/    {auth_extractor.rs, error_response.rs, openapi.rs, path_id.rs, routes.rs, state.rs}
```

Cada subpasta de agregado tem seu proprio `mod.rs` declarando os arquivos
que contem e reexportando os tipos publicos; cada camada tambem tem um
`mod.rs` de topo que declara as 5 subpastas e reexporta tudo achatado -
por isso `crate::domain::User`, `crate::domain::ClienteRepository`,
`crate::presentation::create_cliente` etc. continuam funcionando
exatamente como antes para quem consome a camada de fora. So os imports
internos (`use crate::domain::produto::produto_id::ProdutoId;` etc.)
passaram a incluir o segmento do agregado.

### 2.1 Por que "camada + agregado" e nao o inverso

Havia duas opcoes razoaveis:

1. **Camada primeiro, agregado dentro** (a escolhida): `domain/pedido/`,
   `application/pedido/`, `infra/pedido/`, `presentation/pedido/`. Mantem
   a Dependency Rule visualmente obvia (a raiz de `src/` ainda mostra as
   4 camadas) e e uma mudanca de baixo risco - move arquivos, nao
   redesenha a arquitetura.
2. **Agregado primeiro, camada dentro** (modulo vertical):
   `pedidos/{domain,application,infra,presentation}`. Mais proximo de um
   "modular monolith" pronto para virar crates/servicos separados no
   futuro, mas exige uma mudanca estrutural maior e deixa a regra de
   dependencia menos visivel a primeira vista (cada modulo passa a ter
   sua propria mini-arquitetura).

Para uma API deste tamanho (4 agregados, ainda um unico servico), a
opcao 1 da a maior parte do beneficio de navegacao com o menor risco.

### 2.2 Um ajuste feito durante a reorganizacao

`domain/cliente/`, `domain/produto/` e `domain/pedido/` tinham cada um um
arquivo com o mesmo nome da propria pasta (`cliente/cliente.rs`,
`produto/produto.rs`, `pedido/pedido.rs`) - o Clippy aponta isso como
`module_inception` (modulo com o mesmo nome do modulo que o contem, um
cheiro de codigo). Esses tres arquivos foram renomeados para
`entidade.rs` (`domain/cliente/entidade.rs` etc.), mantendo o tipo
publico com o mesmo nome (`Cliente`, `Produto`, `Pedido`) - o warning some
sem mudar nenhum comportamento. `domain/usuario/user.rs` nao teve esse
problema porque o agregado se chama `usuario` e o arquivo `user.rs`
(nomes diferentes de proposito, ja que "usuario" e o agregado enquanto
`User` e so uma das entidades que ele guarda).

## 3. O que NAO mudou

Tudo documentado nos READMEs anteriores continua valendo sem alteracao de
comportamento - vale a pena ler `../semana-11-produtos-pedidos/README.md`
para o racional de negocio completo:

- Cadastro de Produto e Pedido, com o snapshot de preco/nome no momento
  da compra (`ItemPedido::snapshot_no_momento_do_pedido`, agora em
  `domain/pedido/item_pedido.rs`).
- Auditoria em JSON lines + reidratacao do cache no boot
  (`AuditLogReplayer::replay`, agora em `infra/shared/audit_log_replayer.rs`),
  incluindo a decisao de manter um unico `auditoria.txt` guardando o hash
  bcrypt da senha do usuario.
- Autenticacao/RBAC/rate limiting (Semana 9), cache em memoria (Semana
  10) - inalterados.
- Os 7 testes de integracao (`tests/cadastro_flow.rs`,
  `tests/produtos_pedidos_flow.rs`) passam sem nenhuma mudanca de
  asserção, so os `use cadastro_api::...` no topo dos arquivos apontam
  para os novos caminhos.

## 4. Como rodar

```bash
cd semana-12-organizacao-modular
cargo run
```

Variaveis opcionais (`.env`, a partir de `.env.example`): `JWT_SECRET` e
`AUDIT_LOG_PATH` (padrao `auditoria.txt`). Admin de exemplo semeado no
primeiro boot: `admin@example.com` / `AdminForte123`. Endpoints e fluxo de
exemplo: identicos aos da Semana 11 (ver README de la), so a base da URL
continua `http://localhost:3000`.

## 5. Verificacao feita nesta semana

- `cargo build` e `cargo test` limpos (7 testes passando).
- `cargo clippy --all-targets` sem warnings novos - so o
  `derivable_impls` de `Role::default()`, ja presente e aceito desde a
  Semana 9, e nenhum `module_inception` (ver 2.2).
- Checagem de integridade dos ~96 arquivos `.rs` (chaves/parenteses
  balanceados, ausencia de bytes nulos) apos as movimentacoes e os
  ajustes de import em massa.

## 6. Proximos passos sugeridos

- Se um quinto agregado aparecer, o padrao `camada/agregado/` ja deixa
  claro onde os arquivos novos devem morar.
- Com os `Audited*Repository` agora agrupados por agregado
  (`infra/cliente/`, `infra/produto/`, `infra/pedido/`), fica mais facil
  enxergar a duplicacao entre eles e decidir se vale a pena extrair um
  trait auxiliar comum (ideia ja levantada na Semana 11).
- Se o projeto crescer a ponto de justificar times/deploys separados por
  agregado, a opcao 2 da secao 2.1 (modulo vertical, um passo a caminho
  de multi-crate) fica mais facil de migrar a partir desta estrutura do
  que a partir da estrutura achatada original.
