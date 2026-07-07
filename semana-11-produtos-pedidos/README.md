# cadastro-api — Produtos, Pedidos e Reidratacao do Cache (Rust + Axum)

Semana 11: a estrutura de autenticacao/autorizacao/RBAC da Semana 9 e o
cache + auditoria da Semana 10 sao reaproveitados sem alteracao de
comportamento. O que e novo:

1. **Cadastro de produto** (`Produto`) e **cadastro de pedido**
   (`Pedido`), com um pedido podendo ter varios produtos e quantidades.
2. O pedido guarda o **valor unitario e o valor total no momento da
   compra** - nao uma referencia viva ao preco atual do produto.
3. O arquivo de auditoria da Semana 10 evolui de log de texto livre para
   **JSON lines** e passa a servir tambem como **fonte de reidratacao do
   cache**: se ele existir quando o processo sobe, o cache em memoria e
   reconstruido a partir dele antes da API comecar a atender requisicoes -
   "um banco de dados mais rapido", como descrito no pedido da semana.

Compare com `../semana-10-cache-auditoria` para ver exatamente o diff.

## 1. Regra de dependencia (Clean Architecture) - inalterada

```
presentation  --->  application  --->  domain
                                          ^
infra  -------------------------------- / (implementa portas do dominio)
```

## 2. Produto e Pedido: dois agregados novos

### 2.1 Value Objects novos

- `domain/dinheiro.rs` (`Dinheiro`): valor monetario guardado SEMPRE em
  **centavos** (`i64`), nunca em `f64`. Numero de ponto flutuante nao
  representa exatamente valores decimais como "19.90" - somas repetidas
  acumulam erro de arredondamento. Implementa `Add`/`Sum` (mesma tecnica
  de `impl Add for Vetor2D` da Semana 7, `operadores-vetor2d`), entao
  somar o valor de varios itens de um pedido e uma expressao natural.
  `parse_reais` recebe o preco como texto (`"349.90"`) - o DTO nunca usa
  `f64`, para nao reintroduzir na borda da API o problema que este VO
  existe para evitar.
- `domain/quantidade.rs` (`Quantidade`): inteiro positivo (`> 0`).
- `domain/nome.rs` (`Nome`): **generalizado** nesta semana - na Semana 10
  isto se chamava `NomeCliente` e so era usado por `Cliente`. Como
  `Produto` tambem precisa de "um nome que nao pode ser vazio", o VO foi
  promovido e os dois agregados passam a compartilha-lo (mesmo raciocinio
  ja aplicado ao VO `Email`, reaproveitado por `User` e `Cliente`).

### 2.2 `Produto` (`domain/produto.rs`)

Entidade simples: `id`, `nome`, `preco`. Sem controle de estoque de
proposito (fora do escopo desta semana - ver secao 8). Remover um produto
do cadastro **nunca** invalida pedidos ja feitos - o motivo esta em 2.3.

### 2.3 `ItemPedido` e o "snapshot" (`domain/item_pedido.rs`)

O coracao da regra de negocio pedida na semana. `ItemPedido` guarda uma
**copia** do nome e do preco do produto no exato momento da compra, nao
uma referencia viva a `Produto`:

```rust
pub fn snapshot_no_momento_do_pedido(
    produto_id: ProdutoId,
    nome_produto: Nome,
    quantidade: Quantidade,
    valor_unitario_atual: Dinheiro,
) -> Self {
    let valor_total = valor_unitario_atual.multiplicar_por(quantidade);
    // ...
}
```

Por que isso importa: se `ItemPedido` guardasse so `produto_id` e
calculasse o valor "na hora de exibir" via `ProdutoRepository`, um pedido
do mes passado mudaria de valor total sempre que o preco do produto
mudasse hoje - errado do ponto de vista de negocio (o mesmo motivo pelo
qual uma nota fiscal real imprime o preco praticado na venda, nao o preco
atual da prateleira). O teste
`pedido_guarda_o_preco_do_produto_no_momento_da_compra_mesmo_apos_reajuste`
(`tests/produtos_pedidos_flow.rs`) trava exatamente essa regra: cadastra
um produto a R$100, cria um pedido, reajusta o produto para R$150, e
confere que o pedido antigo continua valendo R$100.

### 2.4 `Pedido` (`domain/pedido.rs`)

Aggregate Root - `ItemPedido` nunca e acessado/persistido fora de um
`Pedido` (mesmo padrao visto na Semana 2, `dominio-pedidos`). Invariante
garantida no construtor: `Pedido::register` recusa uma lista de itens
vazia (`DomainError::PedidoSemItens`) - estruturalmente impossivel criar
um pedido sem produto.

`application/register_pedido.rs` e quem coordena os tres agregados
(`Cliente`, `Produto`, `Pedido`) - confere que o cliente existe, busca
cada produto pelo `ProdutoRepository`, monta o snapshot e so entao chama
`Pedido::register`. E o mesmo papel que `PoliticaCredito` (Domain Service
da Semana 2) ja cumpria coordenando `Cliente` e `Pedido`; aqui a
coordenacao acontece na camada de aplicacao por ser simples o bastante
para nao precisar de um objeto de dominio proprio.

De proposito **nao existe** `PUT /pedidos/:id`: editar as linhas de um
pedido ja feito e uma operacao bem mais delicada que "trocar um email" -
ficou fora do escopo (ver secao 8). Um pedido errado e cancelado
(`DELETE`, restrito a `AdminUser`) e refeito.

## 3. Reidratacao do cache - o tema central da semana

### 3.1 O problema

Cache em memoria (Semana 10) e rapido, mas os dados somem a cada
`cargo run`. A Semana 11 resolve isso reaproveitando o proprio arquivo de
auditoria como um **event log**: como cada evento (`Insert`/`Update`/
`Delete`) ja registra o que aconteceu com cada agregado, da pra
reconstruir o estado final so reaplicando o arquivo inteiro, em ordem, no
boot.

### 3.2 Mudanca de formato: texto livre -> JSON lines

Na Semana 10 uma linha do arquivo era texto formatado para leitura
humana. Nesta semana, `application/audit_logger.rs` reestrutura
`AuditEntry` para carregar um `AuditPayload` - o **estado completo** do
agregado (nao um diff) - e `to_log_line()` serializa a entrada inteira
como um objeto JSON por linha (formato NDJSON/JSON lines):

```
{"entity":"Produto","action":"Insert","aggregate_id":"1a3c...","occurred_at":"2026-07-07T12:17:56Z","payload":{"tipo":"Produto","nome":"Teclado mecanico","preco_centavos":34990}}
{"entity":"Pedido","action":"Insert","aggregate_id":"c01b...","occurred_at":"2026-07-07T12:17:57Z","payload":{"tipo":"Pedido","cliente_id":"8214...","itens":[{"produto_id":"1a3c...","nome_produto":"Teclado mecanico","quantidade":3,"valor_unitario_centavos":34990,"valor_total_centavos":104970}],"valor_total_centavos":104970}}
```

Cada `Insert`/`Update` carrega o snapshot inteiro (nao so o campo
alterado) - decisao deliberada para que o replay seja trivial: basta
"sobrescrever" o agregado com o ultimo estado lido, sem aplicar diffs
incrementais uns sobre os outros. `Delete` so precisa do id.

### 3.3 Decisao de seguranca (pergunta feita ao usuario antes de implementar)

Para reconstruir um `User` que consegue logar depois do restart, o
payload de `Usuario` precisa incluir o **hash bcrypt da senha** (nunca a
senha em texto puro). Isso muda a natureza do arquivo: de "log de leitura
humana" para algo com o mesmo nivel de sensibilidade de um backup de
banco de dados. A decisao (opcao escolhida explicitamente) foi manter um
UNICO arquivo (`auditoria.txt`) fazendo os dois papeis - auditoria e
recuperacao - em vez de separar em dois arquivos. Consequencias praticas:

- `auditoria.txt`/`AUDIT_LOG_PATH` continuam no `.gitignore` (nunca
  versionados).
- Em producao, o arquivo deveria ficar num volume com acesso restrito,
  como qualquer artefato que contenha hashes de senha.
- bcrypt foi desenhado para resistir a ataques offline mesmo se o hash
  vazar, mas "resistente" nao e "seguro publicar" - o cuidado continua
  sendo necessario.

### 3.4 Como o replay funciona (`infra/audit_log_replayer.rs`)

`AuditLogReplayer::replay` roda **uma vez**, no composition root
(`main.rs`), **antes** de o `FileAuditLogger` comecar a aceitar escritas
novas:

1. Le o arquivo inteiro (se existir - se nao, o cache so comeca vazio,
   igual a Semana 10).
2. Para cada linha, tenta decodificar um `AuditEntry` (`serde_json`).
   Uma linha malformada (ex.: arquivo truncado por uma queda no meio de
   uma escrita) vira um aviso no log e e pulada, em vez de derrubar o
   boot inteiro.
3. Reaplica o evento contra o repositorio de CACHE PURO correspondente
   (`CacheUserRepository` etc., NAO o decorado com auditoria) - um
   `Insert` volta a ser um `save`, um `Update` volta a ser um `update`,
   um `Delete` volta a ser um `delete`. Reaplicar historico nao deveria
   gerar historico novo, senao o arquivo cresceria em dobro a cada
   restart.
4. So DEPOIS do replay terminar e que `main.rs` cria o `FileAuditLogger`
   (que abre o arquivo em modo append) e envolve cada cache com seu
   decorator de auditoria - a partir dai, toda escrita nova volta a virar
   uma linha no arquivo.

Um efeito colateral util: como o admin de exemplo so e semeado se ainda
nao existir (`seed_default_admin`), e o replay ja o recria a partir do
arquivo, reiniciar o processo repetidas vezes nao duplica o admin nem
gera eventos de auditoria redundantes.

### 3.5 Prova em teste e em producao

- `tests/produtos_pedidos_flow.rs::cache_e_reidratado_a_partir_do_arquivo_de_auditoria_apos_reinicio_simulado`:
  cria usuario/cliente/produto/pedido, atualiza e remove produtos
  (cobrindo replay de `Insert`, `Update` e `Delete`), "derruba o
  processo" (os `Arc`s saem de escopo), monta um cache **novo e vazio**
  apontando para o mesmo arquivo, roda o replay e confere que tudo volta
  exatamente como estava.
- Testado manualmente com um processo real: `cargo run`, cadastro via
  HTTP, `kill -9` no processo (queda abrupta, nao um shutdown gracioso),
  novo `cargo run` apontando pro mesmo `auditoria.txt` - cliente, produto
  e pedido (com o valor congelado) voltaram, e o login do admin funcionou
  com o mesmo hash de senha.

## 4. SOLID e Object Calisthenics

Mesmos principios das Semanas 9/10, reforcados:

- **OCP**: a reidratacao foi adicionada sem tocar em nenhum caso de uso -
  eles continuam recebendo so um `Arc<dyn XRepository>` e nunca sabem se
  o dado veio de um cache vazio ou de um cache reidratado.
- **SRP**: `AuditLogReplayer` so sabe ler o arquivo e reaplicar eventos;
  `FileAuditLogger` so sabe escrever; nenhum dos dois sabe do outro.
- **Wrap primitives**: `Dinheiro`, `Quantidade`, `Nome` (generalizado) -
  nenhum `f64`/`u32`/`String` solto representa dinheiro, quantidade ou
  nome dentro do sistema.
- Duplicacao consciente e documentada: `AuditedProdutoRepository` e
  `AuditedPedidoRepository` repetem a estrutura de
  `AuditedClienteRepository` quase ao pe da letra. Nao foi extraida uma
  abstracao comum de proposito - a duplicacao e pequena e cada decorator
  le como um caso simples e independente; uma generalizacao prematura
  custaria mais legibilidade do que economizaria linhas.

## 5. Como rodar

```bash
cd semana-11-produtos-pedidos
cargo run
```

Variaveis opcionais (`.env`, a partir de `.env.example`): `JWT_SECRET` e
`AUDIT_LOG_PATH` (padrao `auditoria.txt`). Admin de exemplo semeado no
primeiro boot: `admin@example.com` / `AdminForte123`.

## 6. Endpoints novos desta semana

| Metodo | Rota | Protecao | Descricao |
| :---- | :---- | :---- | :---- |
| POST | `/produtos` | `AuthenticatedUser` | Cadastra produto |
| GET | `/produtos` | `AuthenticatedUser` | Lista produtos |
| GET | `/produtos/:id` | `AuthenticatedUser` | Consulta um produto |
| PUT | `/produtos/:id` | `AuthenticatedUser` | Atualiza um produto |
| DELETE | `/produtos/:id` | `AdminUser` | Remove um produto |
| POST | `/pedidos` | `AuthenticatedUser` | Cria pedido (congela preco/nome dos itens) |
| GET | `/pedidos` | `AuthenticatedUser` | Lista pedidos |
| GET | `/pedidos/:id` | `AuthenticatedUser` | Consulta um pedido |
| DELETE | `/pedidos/:id` | `AdminUser` | Cancela um pedido |

(Endpoints de `/auth`, `/me`, `/admin/*` e `/clientes/*` sao os mesmos
das Semanas 9/10.) Documentacao interativa em
`http://localhost:3000/swagger-ui/`.

## 7. Fluxo completo (exemplo)

```bash
TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"AdminForte123"}' | jq -r .access_token)

CLIENTE_ID=$(curl -s -X POST http://localhost:3000/clientes \
  -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" \
  -d '{"nome":"Rodrigo Furlaneti","documento":"111.444.777-35","email":"cliente@example.com"}' | jq -r .cliente_id)

PRODUTO_ID=$(curl -s -X POST http://localhost:3000/produtos \
  -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" \
  -d '{"nome":"Teclado mecanico","preco":"349.90"}' | jq -r .produto_id)

curl -s -X POST http://localhost:3000/pedidos \
  -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" \
  -d "{\"cliente_id\":\"$CLIENTE_ID\",\"itens\":[{\"produto_id\":\"$PRODUTO_ID\",\"quantidade\":3}]}"

# reinicie o processo (Ctrl+C, cargo run de novo) e confira que os dados
# voltaram, so olhando o log de boot ou consultando os mesmos ids acima.
cat auditoria.txt
```

## 8. Proximos passos sugeridos

- Controle de estoque em `Produto` (reservar/decrementar quantidade ao
  criar um pedido, com tratamento de concorrencia).
- Um `AuditLogger::flush()`/`wait_idle()` opcional (so para testes, atras
  de `#[cfg(test)]`) para eliminar os `sleep` usados hoje nos testes de
  auditoria - trade-off inerente ao design fire-and-forget.
- Compactacao/rotacao do arquivo de eventos: hoje ele cresce
  indefinidamente (inclusive com estados antigos de agregados
  atualizados varias vezes); um "snapshot + log incremental desde o
  snapshot" reduziria o tempo de replay em bases grandes.
- Endpoint de leitura do proprio historico (`GET /admin/auditoria`),
  restrito a `AdminUser`.
- Generalizar os decorators de auditoria (`Audited*Repository`) com um
  trait auxiliar comum, se um quinto agregado auditado aparecer numa
  semana futura.
