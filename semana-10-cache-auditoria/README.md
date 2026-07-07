# cadastro-api — Cache e Auditoria (Rust + Axum)

Semana 10: a estrutura de autenticacao/autorizacao inteira da Semana 9
(JWT, RBAC via extractors, rate limiting no login, Swagger) foi copiada
para esta pasta e mantida sem alteracoes de comportamento. O que muda:

1. A persistencia deixa de ser Postgres/SQLite e passa a ser um **cache em
   memoria de processo** (`HashMap` + `RwLock`, mesmo padrao de estado
   compartilhado da Semana 3).
2. Ganhamos um segundo agregado, **`Cliente`** (cadastro de cliente), com
   CRUD completo, seguindo exatamente os mesmos padroes de `User`.
3. Toda escrita (insert/update/delete) em `User` ou `Cliente` gera uma
   linha num **arquivo de auditoria** (`auditoria.txt`), gravada de forma
   assincrona e "fire-and-forget" - quem faz o cadastro nunca espera o
   disco responder.

Compare esta pasta com `../semana-09-autorizacao` para ver exatamente o
diff entre "persistir em banco" e "persistir em cache com trilha de
auditoria".

## 1. Regra de dependencia (Clean Architecture) - inalterada

```
presentation  --->  application  --->  domain
                                          ^
infra  -------------------------------- / (implementa portas do dominio)
```

`domain` continua sem conhecer `axum`, `tokio::sync`, `bcrypt` ou
`jsonwebtoken`. `application` continua so falando com `trait`s
(`UserRepository`, `ClienteRepository`, `TokenService`, e o novo
`AuditLogger`). `infra` e a unica camada que sabe que existe um
`HashMap` em memoria ou um arquivo `.txt` em disco.

## 2. O que e novo nesta semana

### 2.1 Cliente: um segundo agregado, mesma filosofia de `User`

- `domain/cliente_id.rs`, `domain/nome_cliente.rs`, `domain/cpf.rs`:
  Value Objects novos - nenhuma `String`/`Uuid` solta representa um
  cliente. `Cpf::parse` implementa o algoritmo oficial do digito
  verificador (nao so checa o tamanho), rejeitando tambem sequencias
  repetidas ("111.111.111-11").
- `domain/cliente.rs`: entidade `Cliente` com `id/nome/documento/email`
  (reaproveita o VO `Email` de `User` - o conceito "email valido" e
  identico nos dois contextos). Imutavel de fora: `with_updated_data`
  devolve um novo `Cliente`, quem chama decide persistir.
- `domain/cliente_repository.rs`: porta `ClienteRepository`, com
  `save`/`update`/`delete`/`find_by_id`/`find_by_documento`/`list_all`.
- `application/register_cliente.rs`, `update_cliente.rs`,
  `delete_cliente.rs`, `get_cliente.rs`, `list_clientes.rs`: um caso de
  uso por operacao (SRP), simetricos aos casos de uso de `User`.

### 2.2 Usuario ganhou update/delete administrativos

A Semana 9 so tinha "insert" de usuario (`POST /auth/register`). Para o
log de auditoria fazer sentido ("insert, update ou delete"), a
Semana 10 acrescenta:

- `domain/user.rs::with_email` - regra de negocio de troca de email,
  mantendo id/senha/role.
- `domain/repository.rs::UserRepository` ganhou `update`/`delete`.
- `application/update_user.rs`, `delete_user.rs` - casos de uso novos,
  expostos em `PUT`/`DELETE /admin/usuarios/:id`, restritos a
  `AdminUser` (mesmo guard de RBAC da Semana 9).

### 2.3 Cache em memoria (`infra/cache_user_repository.rs`, `cache_cliente_repository.rs`)

Cada "tabela" e um `Arc<RwLock<HashMap<Uuid, T>>>`: leituras concorrentes
(`RwLock::read`), escritas exclusivas (`RwLock::write`) - o mesmo
raciocinio de `Arc<Mutex<T>>` da Semana 3, so trocando `Mutex` por
`RwLock` porque ha muito mais leitura (`GET`) do que escrita aqui. Cada
leitura devolve uma **copia** (`User`/`Cliente` agora derivam `Clone`),
nunca uma referencia viva presa dentro do lock.

De proposito estas duas classes nao sabem nada sobre auditoria - so
cuidam do cache. Isso e Open/Closed Principle na pratica: acrescentamos
"grave um historico" sem tocar em uma linha delas.

### 2.4 Auditoria em arquivo, sem travar a requisicao

- `application/audit_logger.rs`: porta `AuditLogger` com um unico metodo,
  `fn record(&self, entry: AuditEntry)` - **sincrono, sem `Result`**. O
  contrato deixa explicito que quem registra auditoria nunca espera (nem
  recebe) confirmacao de sucesso ou falha - requisito direto da semana.
  `AuditEntry`/`AuditAction`/`AuditEntity` sao o vocabulario fechado
  ("wrap primitives") de "o que mudou, em qual agregado, quando".
- `infra/file_audit_logger.rs` (`FileAuditLogger`): a implementacao.
  `record()` so empurra a entrada num canal `tokio::sync::mpsc`
  *unbounded* (mesmo mecanismo de canais da Semana 3,
  `worker-pedidos`) - isso e uma operacao de memoria, nunca bloqueia.
  Uma **unica task de background**, iniciada uma vez em `main.rs`, e a
  dona exclusiva do arquivo: ela consome o canal em loop e escreve as
  linhas em sequencia, sem nenhuma corrida de escrita concorrente.
- `infra/audited_user_repository.rs` e `infra/audited_cliente_repository.rs`:
  **Decorator** (padrao GoF) sobre `UserRepository`/`ClienteRepository`.
  Implementam o mesmo `trait` do cache, delegam a escrita para o cache
  interno e, se der certo, chamam `audit.record(...)`. Leituras so
  delegam, sem gerar linha de auditoria. Nem `RegisterUser` nem
  `RegisterCliente` (nem nenhum outro caso de uso) sabe que auditoria
  existe - a decisao de "quem e auditado" mora inteira no composition
  root (`main.rs::build_audited_user_repository`).

Formato de uma linha do arquivo (`AuditEntry::to_log_line`):

```
2026-07-07T14:32:01.123456+00:00 | USUARIO  | INSERT | id=6f1b... | email=rodrigo@example.com role=user
2026-07-07T14:33:10.987654+00:00 | CLIENTE  | UPDATE | id=9ac2... | nome=Rodrigo Furlaneti documento=11144477735 email=cliente@example.com
2026-07-07T14:35:44.555555+00:00 | USUARIO  | DELETE | id=6f1b... | usuario removido do cache
```

## 3. SOLID e Object Calisthenics - mesmos principios da Semana 9

- **SRP**: cada caso de uso, cada Value Object, cada adapter de cache e o
  `FileAuditLogger` tem uma unica razao para mudar.
- **OCP**: auditoria foi adicionada via Decorator, sem tocar em
  `CacheUserRepository`/`CacheClienteRepository` nem nos casos de uso.
- **LSP**: `AuditedUserRepository` e `CacheUserRepository` sao ambas
  `UserRepository` validas e intercambiaveis (o mesmo vale para o par de
  `Cliente`) - nenhum caso de uso percebe diferenca.
- **ISP**: `AuditLogger` e uma porta de um metodo so.
- **DIP**: `application`/`domain` dependem so de `trait`s; `tokio::sync::mpsc`,
  `tokio::fs` e o `HashMap` do cache ficam inteiramente em `infra`.

Object Calisthenics: `Cpf`, `NomeCliente`, `ClienteId`, `AuditAction` e
`AuditEntity` sao mais um capitulo de "wrap all primitives". Metodos
continuam com um nivel de indentacao por bloco logico (ver
`Cpf::is_invalid`, que delega cada verificacao a um metodo privado em vez
de encadear `&&`/`||` com logica inline).

## 4. Como rodar

```bash
cd semana-10-cache-auditoria
cargo run
```

Sem nenhuma variavel de ambiente configurada, a API sobe em
`http://0.0.0.0:3000`, com um cache vazio (exceto o admin de exemplo) e
comeca a gravar em `./auditoria.txt`. Variaveis opcionais (`.env`, a
partir de `.env.example`): `JWT_SECRET` e `AUDIT_LOG_PATH`.

Como o cache e 100% em memoria, **reiniciar o processo apaga os dados**
(usuarios e clientes) - o arquivo de auditoria, por outro lado,
**persiste em disco** e acumula o historico completo entre reinicios
(modo append).

## 5. Endpoints

| Metodo | Rota | Protecao | Descricao |
| :---- | :---- | :---- | :---- |
| POST | `/auth/register` | publica | Cadastra usuario (sempre `Role::User`) |
| POST | `/auth/login` | publica, rate limited | Login, devolve JWT |
| GET | `/me` | `AuthenticatedUser` | Dados do usuario logado |
| GET | `/admin/ping` | `AdminUser` | Exemplo de rota restrita a admin |
| PUT | `/admin/usuarios/:id` | `AdminUser` | Atualiza email de um usuario |
| DELETE | `/admin/usuarios/:id` | `AdminUser` | Remove um usuario |
| POST | `/clientes` | `AuthenticatedUser` | Cadastra cliente |
| GET | `/clientes` | `AuthenticatedUser` | Lista clientes |
| GET | `/clientes/:id` | `AuthenticatedUser` | Consulta um cliente |
| PUT | `/clientes/:id` | `AuthenticatedUser` | Atualiza um cliente |
| DELETE | `/clientes/:id` | `AdminUser` | Remove um cliente |

Documentacao interativa em `http://localhost:3000/swagger-ui/` (clique em
**Authorize** e cole o `access_token` do login).

## 6. Fluxo completo (exemplo)

```bash
# login com o admin semeado no boot
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"AdminForte123"}'
# -> guarde o access_token da resposta em $TOKEN

# cadastra um cliente
curl -X POST http://localhost:3000/clientes \
  -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" \
  -d '{"nome":"Rodrigo Furlaneti","documento":"111.444.777-35","email":"cliente@example.com"}'

# lista clientes
curl http://localhost:3000/clientes -H "Authorization: Bearer $TOKEN"

# remove o cliente (id devolvido no cadastro)
curl -X DELETE http://localhost:3000/clientes/<cliente-id> \
  -H "Authorization: Bearer $TOKEN"

# confira o historico gerado
cat auditoria.txt
```

## 7. Testes automatizados

`tests/cadastro_flow.rs` sobe os casos de uso reais na frente de um
`CacheUserRepository`/`CacheClienteRepository` decorados com um
`FileAuditLogger` apontando para um arquivo temporario por teste (cada
teste usa seu proprio arquivo, isolado dos demais). Cobrem:

- Ciclo completo de usuario: cadastro, login, `GET /me`, atualizacao de
  email e remocao - conferindo que cada operacao aparece no arquivo de
  auditoria com a acao certa (`INSERT`/`UPDATE`/`DELETE`).
- Ciclo completo de cliente: cadastro, consulta, listagem, atualizacao e
  remocao, com a mesma checagem de auditoria.
- Regras de negocio: `Cpf` invalido e rejeitado, documento duplicado e
  rejeitado, usuario/cliente inexistente devolve erro ao tentar
  atualizar/remover.

```bash
cargo test
```

Nota sobre o teste da auditoria: como `record()` e "fire-and-forget", os
testes dao um pequeno `tokio::time::sleep` apos a operacao antes de ler o
arquivo - um trade-off inerente ao design assincrono pedido pela semana
(nao ha como "esperar a escrita terminar" sem reintroduzir o acoplamento
que o requisito pediu para evitar). Ver secao 8 para uma alternativa.

## 8. Proximos passos sugeridos

- Expor um `AuditLogger::flush()`/`wait_idle()` opcional (so para testes,
  atras de `#[cfg(test)]` ou feature flag) para eliminar o `sleep` acima
  sem reintroduzir espera sincrona no caminho de producao.
- Persistir o cache em disco periodicamente (snapshot), para sobreviver a
  reinicios sem perder os dados - hoje so o arquivo de auditoria e
  duravel.
- Endpoint de leitura do proprio historico (`GET /admin/auditoria`),
  paginando o arquivo, restrito a `AdminUser`.
- Generalizar os dois decorators quase identicos
  (`AuditedUserRepository`/`AuditedClienteRepository`) com um trait
  auxiliar comum, se um terceiro agregado auditado aparecer numa semana
  futura (ver comentario em `audited_cliente_repository.rs` sobre por que
  isso nao foi feito ainda - abstracao prematura).
- Rotacao/particionamento do arquivo de auditoria por data, para nao
  crescer indefinidamente em producao.
