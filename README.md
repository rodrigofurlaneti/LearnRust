# LearnRust

Repositorio de estudos de Rust, criado a partir de um plano intensivo voltado para um desenvolvedor senior com background em C\#/.NET, DDD, Clean Architecture e SOLID. O objetivo e aprender Rust aplicando, desde o inicio, os mesmos padroes de arquitetura ja dominados em .NET, comparando cada conceito novo com seu equivalente (ou ausencia de equivalente) no mundo C\#.

Cada semana tem sua propria pasta, com um ou mais projetos Cargo independentes. Os projetos evoluem de forma incremental: o dominio criado na Semana 2 e reaproveitado nas Semanas 4, 5 e 6\. A partir da Semana 1 e da Semana 2, foram adicionados exemplos extras para reforcar conceitos especificos.

## Pre-requisitos e ferramentas instaladas

- **Rust toolchain**: instalado via `rustup` (perfil padrao, toolchain `stable`, target `x86_64-pc-windows-msvc`). Inclui `rustc`, `cargo`, `clippy` e `rustfmt`.  
- **Target WebAssembly**: `wasm32-unknown-unknown` (via `rustup target add`), usado na Semana 5\.  
- **wasm-pack**: ferramenta para compilar e empacotar modulos WASM (`cargo install wasm-pack`).  
- **basic-http-server**: servidor HTTP estatico simples para testar paginas WASM localmente, sem depender de Python/Node (`cargo install basic-http-server`).  
- **IDE**: Visual Studio 2022/2026, com a extensao **rust-analyzer.vs** (Kitam Studios), que da suporte a intellisense, build, debug e test explorer para projetos Rust abertos via "Abrir Pasta".

### Observacoes de ambiente (Windows)

- O **Developer PowerShell do Visual Studio** pode nao refletir imediatamente o PATH atualizado apos instalar o Rust. Se `cargo`/`rustc` nao forem reconhecidos, feche e reabra o Visual Studio (ou ajuste o PATH manualmente na sessao com `$env:Path += ";$env:USERPROFILE\.cargo\bin"`).  
- Arquivos `.rs` com acentuacao (ex.: "nao", "e") devem ser salvos como **UTF-8 sem assinatura** no Visual Studio (`Salvar Como` \> `Salvar com Codificacao`). Por simplicidade, o codigo deste repositorio evita acentos em comentarios e mensagens de erro para nao depender desse ajuste.  
- Servidores (Axum, `basic-http-server`) bloqueiam o terminal enquanto rodam — use uma segunda janela do PowerShell para testar endpoints com `Invoke-RestMethod` ou o navegador.

## Estrutura do repositorio

LearnRust/

├── semana-01-fundamentos/

│   ├── conversor-unidades/        \# binario — ownership, borrowing, lifetimes, Result/Option

│   ├── validador-senha/           \# binario — enums, Vec, acumulo de erros

│   ├── carrinho-compras/          \# binario — \&mut self, Option\<\&T\>

│   ├── conta-bancaria/            \# binario — encapsulamento, Result custom

│   ├── processador-notas/         \# binario — slices, iterators (map/filter/fold)

│   ├── classificador-idade/       \# binario — match com ranges e guards

│   └── calculadora-closures/      \# binario — closures, impl Fn

├── semana-02-arquitetura/

│   ├── Cargo.toml                 \# workspace

│   ├── dominio-pedidos/           \# lib — dominio puro (Pedido, ItemPedido, trait Repository)

│   ├── infraestrutura/            \# lib — RepositorioEmMemoria (implementacao concreta)

│   ├── dominio-clientes/          \# lib — Cliente, Value Objects, Repository generico\<T\>

│   ├── dominio-vendas/            \# lib — Domain Service PoliticaCredito

│   └── aplicacao-pedidos/         \# lib — casos de uso (Application layer)

├── semana-03-concorrencia/

│   └── worker-pedidos/            \# binario — threads, Tokio, thiserror/anyhow

├── semana-04-backend/

│   ├── api/                       \# binario — Axum \+ SQLx \+ tracing

│   └── cli/                       \# binario — clap (administrativo)

├── semana-05-wasm/

│   └── pedido-wasm/               \# lib (cdylib) — compilado para WASM

├── semana-06-ia-agentes/

│   └── agente-pedidos/            \# binario — agente de IA com Rig (em andamento)

└── semana-07-tipos-traits/

    ├── adts-pessoa/               \# binario — Algebraic Data Types (enum \+ composicao)

    ├── polimorfismo-formas/       \# binario — static dispatch vs dyn Trait

    ├── operadores-vetor2d/        \# binario — std::ops::Add, PartialEq, Display

    └── smart-pointers-arvore/     \# binario — Box\<T\> e Rc\<T\>

## Semana 1 — Fundamentos e Ownership Model

Conceitos centrais: sintaxe basica, structs, enums com dados associados, pattern matching exaustivo (`match`), ownership e move semantics, borrowing (`&`, `&mut`), lifetimes explicitos (`'a`), e tratamento de erros com `Option`/`Result` (sem `null`, sem excecoes).

Paralelo com C\#: ownership substitui o Garbage Collector por verificacao em tempo de compilacao; `Result<T, E>` substitui excecoes para erros esperados; `enum` em Rust e muito mais proximo de union types do que o `enum` do C\#.

| Projeto | Foco especifico | Como rodar |
| :---- | :---- | :---- |
| `conversor-unidades` | ownership, borrowing, lifetimes (`'a`), `Result<f64, String>` | `cd semana-01-fundamentos\conversor-unidades && cargo run` |
| `validador-senha` | enums de erro, `Vec`, acumulo de multiplos erros em vez de parar no primeiro | `cd semana-01-fundamentos\validador-senha && cargo run` |
| `carrinho-compras` | metodos `&mut self` (escrita) vs `&self` (leitura), `Option<&T>` emprestado de dentro de uma struct | `cd semana-01-fundamentos\carrinho-compras && cargo run` |
| `conta-bancaria` | encapsulamento (campo privado \+ metodos), `Result` customizado para regras de negocio | `cd semana-01-fundamentos\conta-bancaria && cargo run` |
| `processador-notas` | slices (`&[f64]`), cadeia de iterators (`filter`/`map`/`collect`) — equivalente ao LINQ | `cd semana-01-fundamentos\processador-notas && cargo run` |
| `classificador-idade` | `match` com ranges (`0..=12`) e guards (`if idade >= 60`) | `cd semana-01-fundamentos\classificador-idade && cargo run` |
| `calculadora-closures` | closures como parametro (`impl Fn(f64, f64) -> f64`), captura de variavel do ambiente — equivalente a `Func<T,TResult>` | `cd semana-01-fundamentos\calculadora-closures && cargo run` |

## Semana 2 — Traits, Generics e Arquitetura Idiomatica (DDD)

Workspace `semana-02-arquitetura` com cinco crates, representando as camadas de um Clean Architecture:

- `dominio-pedidos` (lib): dominio puro — `ItemPedido` e `Pedido` (Value Objects e Aggregate Root com invariantes garantidas por smart constructors), e o trait `PedidoRepository` (a porta/interface).  
- `infraestrutura` (lib): implementacao concreta `RepositorioEmMemoria`, que depende de `dominio-pedidos` via path dependency.  
- `dominio-clientes` (lib): `Cliente`, Value Objects `Email`/`Cpf` (newtype com smart constructor), e um **Repository generico** `Repositorio<T>`/`RepositorioEmMemoria<T>` reutilizavel para qualquer entidade, sem duplicar codigo (generics resolvidos por monomorphization).  
- `dominio-vendas` (lib): `PoliticaCredito`, um **Domain Service** que coordena `Cliente` e `Pedido` sem pertencer a nenhum dos dois — depende das outras duas crates de dominio, mas nenhuma delas depende de volta.  
- `aplicacao-pedidos` (lib): a **camada Application** — `CriarPedidoCasoDeUso`/`ListarPedidosCasoDeUso`, que dependem apenas do trait `PedidoRepository` (nunca da `infraestrutura` concreta), permitindo testar com um repositorio falso (`RepositorioFalso`) sem mock library.

Conceitos aplicados: traits (contratos sem heranca de implementacao), generics vs trait objects (`dyn Trait`), padrao newtype para Value Objects, Aggregate Root com invariantes, Repository como trait (generico e especifico), Domain Service, camada Application desacoplada da infraestrutura, organizacao em workspace/crates por camada — a mesma separacao de projetos `.csproj` do Clean Architecture, mas com a direcao da dependencia garantida pelo compilador, nao so por convencao.

Testes (`#[test]`, equivalente ao `[Fact]` do xUnit) cobrem validacao de Value Objects, calculo de totais do Aggregate, a politica de credito e os casos de uso da Application.

Como rodar (testa todas as crates do workspace de uma vez):

cd semana-02-arquitetura

cargo test

## Semana 3 — Concorrencia, Async e Padroes de Sistema

**Projeto:** `semana-03-concorrencia/worker-pedidos` (executavel)

Conceitos aplicados: threads nativas com `Arc<Mutex<T>>` para estado compartilhado seguro (verificado em compile-time via `Send`/`Sync`); programacao assincrona com Tokio (`#[tokio::main]`, `tokio::spawn`, canais `mpsc`); tratamento de erros em escala com `thiserror` (erros tipados) e `anyhow` (propagacao na camada de aplicacao); testes assincronos com `#[tokio::test]`.

Paralelo com C\#: `Arc<Mutex<T>>` \~ `lock`/`Monitor`, mas com a exclusividade garantida pelo compilador; `async`/`await` tem sintaxe quase identica, mas uma `Future` em Rust e "preguicosa" e so executa dentro de um runtime (Tokio), diferente de uma `Task` do C\# que ja inicia "quente".

Como rodar:

cd semana-03-concorrencia\\worker-pedidos

cargo test

## Semana 4 — Backend/APIs REST e CLI

**Projetos:** `semana-04-backend/api` (Axum) e `semana-04-backend/cli` (clap), ambos consumindo `dominio-pedidos` e `infraestrutura` da Semana 2 via path dependency.

`api`: servidor REST com Axum, rotas `POST /pedidos` (cria pedido, valida via smart constructors do dominio) e `GET /pedidos` (lista em memoria via `RepositorioEmMemoria`); persistencia adicional em SQLite via SQLx (`GET /pedidos/historico`); observabilidade estruturada com `tracing`/`tracing-subscriber`. Estado compartilhado via `Arc<Mutex<RepositorioEmMemoria>>` (mesmo padrao da Semana 3).

`cli`: ferramenta administrativa com `clap` (subcomandos `seed` e `relatorio`), que le/escreve no **mesmo arquivo SQLite** usado pela API (`pedidos.db`), demonstrando dois processos independentes compartilhando persistencia.

Paralelo com C\#: Axum \~ Minimal APIs do ASP.NET Core; SQLx (modo sem macro) \~ Dapper; `tracing` \~ Serilog; `criar_pedido`/`listar_pedidos` ja seguem CQRS na pratica (commands vs queries), sem precisar de um dispatcher tipo MediatR.

Como rodar a API:

cd semana-04-backend\\api

cargo run

Em outro terminal, testar:

Invoke-RestMethod \-Uri http://127.0.0.1:3000/pedidos \-Method Post \-Body '{"itens":\[{"produto":"Mouse","quantidade":2,"preco\_unitario":50.0}\]}' \-ContentType "application/json"

Invoke-RestMethod \-Uri http://127.0.0.1:3000/pedidos/historico \-Method Get

Como rodar a CLI (com a API parada ou rodando, ambas compartilham o banco):

cd semana-04-backend\\cli

cargo run \-- seed \--quantidade 2

cargo run \-- relatorio

## Semana 5 — WebAssembly (WASM)

**Projeto:** `semana-05-wasm/pedido-wasm` (lib `cdylib`), tambem dependente de `dominio-pedidos`.

Uma crate adaptadora fina expoe `calcular_total_pedido` via `wasm-bindgen`, sem acoplar o dominio puro a tecnologia de WASM (mesmo principio de Anti-Corruption Layer usado nos DTOs da API). Compilado com `wasm-pack build --target web`, gerando a pasta `pkg/` (ignorada no git, e artefato de build). Consumido por uma pagina `index.html` simples via modulo ES, servida localmente com `basic-http-server`.

Como rodar:

cd semana-05-wasm\\pedido-wasm

wasm-pack build \--target web

basic-http-server .

Depois, abrir `http://127.0.0.1:4000` no navegador.

## Semana 6 — IA Generativa e Agentes em Rust (Capstone)

**Projeto:** `semana-06-ia-agentes/agente-pedidos` (executavel, em andamento)

Uso do framework **Rig** (`rig-core`) para orquestracao de LLM, com o provedor Anthropic (Claude). Requer a variavel de ambiente `ANTHROPIC_API_KEY` (chave gerada em console.anthropic.com; contas novas recebem cerca de US$ 5 em creditos gratuitos, sem necessidade de cartao, apenas verificacao de celular por SMS).

Objetivo final do capstone: um agente que expoe uma tool customizada (usando o dominio `Pedido` da Semana 2), integrando os quatro focos do curso (Backend, Sistemas, IA e, opcionalmente, WASM).

Como rodar:

$env:ANTHROPIC\_API\_KEY \= "sk-ant-sua-chave-aqui"

cd semana-06-ia-agentes\\agente-pedidos

cargo run

## Semana 7 — Sistema de Tipos, Traits e Smart Pointers

Modulo criado para aprofundar, com nomenclatura propria de Rust (sem forcar analogia com Orientacao a Objetos classica), os mecanismos que substituem heranca e os demais recursos do sistema de tipos.

| Projeto | Foco especifico |
| :---- | :---- |
| `adts-pessoa` | **Algebraic Data Types**: `struct` como product type, `enum` como sum type. Modela `Pessoa` (Fisica/Juridica) como variantes de enum com `DadosComuns` compartilhado via composicao — sem heranca de struct, que nao existe em Rust. |
| `polimorfismo-formas` | **Static dispatch vs dynamic dispatch**: generics com trait bound (`fn f<T: Forma>`, monomorphization, zero custo em runtime) comparado a trait objects (`&dyn Forma`, `Vec<Box<dyn Forma>>`, que permite misturar tipos diferentes na mesma colecao). |
| `operadores-vetor2d` | **Traits de operador da std**: `impl Add for Vetor2D` habilita o operador `+`; `#[derive(PartialEq)]` gera `==`; `impl fmt::Display` habilita `{v}` na formatacao — tudo via trait, sem herdar de nenhuma classe base. |
| `smart-pointers-arvore` | **Box e Rc**: `Box<T>` resolve o problema de tipos recursivos (tamanho infinito sem indirecao); `Rc<T>` permite multiplos "donos" do mesmo dado via contagem de referencias (`Rc::strong_count`), sem duplicar os dados. |

Como rodar qualquer um dos quatro:

cd semana-07-tipos-traits\\\<nome-do-projeto\>

cargo run

Mapeamento de conceito (evitando o vocabulario de OO classica): o que seria "classe" em C\# vira `struct` (dados) \+ `impl` (comportamento, em bloco separado); o que seria "herenca" vira `enum` (conjunto fechado de variantes) ou composicao (tipo contendo outro tipo); o que seria "interface"/"polimorfismo" vira `trait`, com escolha explicita entre dispatch estatico e dinamico; "sobrecarga de operador" e "override de ToString/Equals" viram implementacoes dos traits `std::ops`/`Display`/`PartialEq`.

## Fluxo geral de dependencias entre crates (Semana 2\)

dominio-pedidos        dominio-clientes

      │                        │

      │                        │

      ▼                        ▼

infraestrutura          dominio-vendas (Domain Service, depende dos dois dominios)

      │

      ▼

aplicacao-pedidos (depende so do TRAIT PedidoRepository do dominio, nunca da infraestrutura)

O dominio nunca depende de nenhuma camada externa (Application, Infrastructure, API, WASM, IA) — a dependencia sempre aponta para dentro, replicando a regra de dependencia do Clean Architecture. Isso e reforcado nas Semanas 4 e 5, onde `api`, `cli` e `pedido-wasm` dependem de `dominio-pedidos`/`infraestrutura`, mas nunca o contrario.

## Proximos passos

- Concluir o agente de IA da Semana 6 com tool customizada (validacao/calculo do dominio `Pedido` exposto como function calling para o Claude), apos validar a chave `ANTHROPIC_API_KEY`.  
- Explorar RAG simples e integracao MCP (Model Context Protocol) como extensao opcional do capstone.  
- Opcional: refatorar o `api` da Semana 4 para consumir `aplicacao-pedidos` (Semana 2\) em vez de ter a logica de caso de uso direto nos handlers HTTP.

