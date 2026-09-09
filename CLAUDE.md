# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Visão Geral

`tpl` é uma **aplicação de linha de comandos escrita em Rust** que lê a estrutura (DDL / schema) de bases de dados **MariaDB** e aplica **templates MiniJinja** para produzir texto ou código a partir dessa estrutura.

O modelo de interacção é **inspirado no `git`**: um único executável, comandos *porcelain* para humanos, subcomandos, aliases curtos, flags longas e curtas, configuração hierárquica (global e local, com precedência bem definida) e comandos *plumbing* com saída estável, pensados para composição em pipelines.

## Os Três Braços

A aplicação organiza-se em três braços, e **todos são read-only**. Esta é a arrumação de topo do produto: todo o comando pertence a um braço, e uma funcionalidade que não caiba em nenhum deles não pertence ao `tpl`.

| # | Braço | Comando | O que faz |
|---|---|---|---|
| 1 | Exploração da base de dados | `tpl schema …` | Lê e apresenta a estrutura: características da BD, tabelas, colunas, índices, chaves, vistas, rotinas |
| 2 | Exploração dos templates | `tpl template …` | Lê e apresenta os templates disponíveis em `.tpl/templates/` |
| 3 | Renderização | `tpl render …` | Lê a base de dados → lê o template → renderiza → **imprime o output** |

O braço 3 é a composição dos dois primeiros e o único que os junta. O seu pipeline é fixo e não tem outras etapas: *ler BD → ler template → renderizar → imprimir*.

**Por defeito, o resultado de qualquer comando vai para a consola.** Isto vale para os três braços, não apenas para o `render`: `stdout` é o destino por omissão, sempre.

Guardar o resultado em disco é possível, mas **só por flag explícita** (`--output`, e no `render` também `--output-dir`). Sem essas flags o `tpl` não cria nem toca em nenhum ficheiro fora de `.tpl/`. O destino do output é, por isso, uma decisão sempre visível na linha de comando — nunca um efeito lateral implícito, o que importa quando quem invoca é um agente que não supervisiona a árvore de ficheiros.

Fora dos três braços existe apenas a gestão do projecto (`tpl init`, `tpl database …`, `tpl config …`): é auxiliar, não é um braço, e a sua escrita limita-se à pasta `.tpl/`.

## Consumidor Primário: Agentes de IA

O utilizador esperado do `tpl` **não é uma pessoa numa shell interactiva — é um agente de IA** (Claude Code, Codex e equivalentes) a invocar a ferramenta programaticamente e a ler o resultado. Isto é um requisito de design com o mesmo peso dos anteriores: molda o `--help`, os exit codes, as mensagens de erro e o que se escreve em `stdout`.

Um agente dispõe de três canais para perceber uma CLI, e só três: o **texto de ajuda**, o **exit code** e o **stdout/stderr**. Os três têm de bastar, sozinhos, para descobrir como usar a ferramenta, saber se correu bem, e saber o que fazer a seguir quando não correu.

### Ajuda legível por LLM

- **Auto-suficiência.** `tpl --help` e `tpl <comando> --help` descrevem completamente o nível em que estão e enumeram os filhos. Nunca remeter para manual externo, site ou README — o agente não os vai buscar.
- **Estrutura fixa e idêntica em todos os níveis**, sempre pela mesma ordem: `USAGE`, `DESCRIPTION`, `ARGUMENTS`, `OPTIONS`, `EXAMPLES`, `EXIT CODES`, `SEE ALSO`.
- **Exemplos em toda a ajuda.** Todo o `--help` de comando termina com pelo menos um exemplo completo, copiável e correcto. Os agentes copiam exemplos: um exemplo vale mais do que três parágrafos de prosa.
- **Explicitar o que habitualmente se subentende:** tipo de cada valor, valor por defeito, obrigatoriedade, valores admissíveis enumerados (`--tls <disabled|preferred|required>`), repetibilidade, e exclusões mútuas (`--dsn` é incompatível com `--host`).
- **Ajuda em JSON.** `tpl help --format json` emite a **árvore completa de comandos** num único documento — comandos, subcomandos, aliases, argumentos, flags, tipos, defaults, enums, obrigatoriedade e exit codes. É como um agente carrega toda a superfície da CLI numa só invocação, em vez de a descobrir por tentativa e erro. Este documento é contrato e tem teste próprio.
- **Concisão.** A ajuda ocupa janela de contexto. Frases curtas, sem prosa de marketing, sem repetir entre níveis o que já foi dito.

### Nunca interactivo

O `tpl` **nunca** pede input. Sem prompts de confirmação, sem pedidos de password, sem paginador, sem leitura de `stdin` que não seja um `--context` explicitamente pedido. Um prompt bloqueia um agente indefinidamente e o timeout que se segue não lhe diz o que falhou. Perante informação em falta, o comportamento correcto é falhar de imediato, com o exit code adequado e uma mensagem que diga exactamente o que falta.

### Sucesso é silencioso

Um comando bem sucedido escreve **apenas o resultado esperado** em `stdout` e termina com `0`. Nada de `OK`, `Done`, sumários, contagens, tempos decorridos, emoji, barras de progresso ou spinners. Um comando que não produza dados — `tpl init`, `tpl database add` — não escreve nada: o `0` é a mensagem.

Tudo o que não seja resultado vai para `stderr`, incluindo avisos e o output do `tracing`.

### Determinismo

A mesma invocação sobre o mesmo estado produz **output byte a byte idêntico**. Ordenações são sempre explícitas e estáveis — tabelas por nome, colunas por posição ordinal, índices por nome — nunca a ordem de chegada do servidor. Os agentes comparam outputs entre execuções, e instabilidade lê-se como mudança real.

A variável `now` do contexto de render é a única excepção conhecida e quebra a reprodutibilidade de quem a use. Documentá-la como tal no README.

## Âmbito de Trabalho

**O âmbito deste projecto restringe-se à pasta de raiz do repositório.**

Não é permitido ler, listar, pesquisar, inspeccionar ou referenciar nada fora de `tpl/`. Isto inclui directorias irmãs, outros projectos do utilizador, e qualquer caminho acima da raiz do repositório. Não há excepções para "procurar convenções", "ver como outro projecto resolveu isto" ou "confirmar um padrão".

Toda a informação necessária ao trabalho vive dentro deste repositório. Se algo não estiver aqui, pergunta-se ao utilizador — não se vai procurar lá fora.

## Invariantes do Projecto

Estas duas regras têm prioridade sobre qualquer outra consideração de design ou de implementação.

### 1. Read-only absoluto

O `tpl` **nunca** emite DDL, DML ou qualquer statement de escrita contra a base de dados a que se liga. Toda a ligação estabelecida deve impor o modo read-only ao nível do motor:

```sql
SET SESSION TRANSACTION READ ONLY;
```

Se a sessão read-only não puder ser estabelecida, a ligação é recusada e o processo termina com código `78`. Não existe flag para desligar este comportamento.

A leitura do catálogo é feita exclusivamente através de `INFORMATION_SCHEMA` (e, quando estritamente necessário, `SHOW`). Nunca através de `mysqldump` ou de qualquer processo externo.

### 2. Renderização em runtime, com MiniJinja

Os templates são **ficheiros em disco, carregados e compilados no momento do render** — nunca embebidos no binário em tempo de compilação. O motor é o **[MiniJinja](https://docs.rs/minijinja)** (`minijinja` + `minijinja-contrib`).

**Não usar Askama, Tera, Handlebars, `std::fmt`, nem qualquer outro motor.** O Askama em particular está explicitamente excluído por ser *compile-time*: obrigaria a recompilar o `tpl` sempre que um template mudasse, o que contraria a razão de ser desta ferramenta.

Configuração obrigatória do `Environment` do MiniJinja:

- **Loader de sistema de ficheiros** (`Environment::set_loader`) com raiz na directoria de templates resolvida, para que `{% include %}`, `{% import %}` e `{% extends %}` funcionem com caminhos relativos.
- **`UndefinedBehavior::Strict`** — uma variável indefinida é um erro de render, nunca uma string vazia silenciosa.
- **Auto-escape desligado por defeito** — o alvo primário é geração de código, não HTML. Activar auto-escape apenas por extensão (`.html`, `.xml`, `.htm`).
- Erros de render devem ser reportados com **nome do template, linha e coluna**, propagando a cadeia de `minijinja::Error`.

## Requisitos Não-Funcionais

Estes dois requisitos são de primeira ordem: pesam nas decisões de design tanto quanto a correcção funcional. Uma implementação correcta mas lenta, ou correcta mas gastadora, **não** satisfaz o requisito.

### 1. Desempenho extremo

O `tpl` tem de ser muito rápido. É uma ferramenta de linha de comandos, invocada repetidamente e frequentemente dentro de loops e pipelines de build — cada milissegundo é pago muitas vezes.

**Orçamentos iniciais.** São alvos de trabalho, a ratificar assim que houver medição real, e a partir daí tratados como limites:

| Operação | Alvo |
|---|---|
| `tpl --version`, `tpl --help` | < 5 ms de wall time |
| Arranque até ao primeiro byte de trabalho útil | < 10 ms |
| `tpl schema dump` sobre 200 tabelas | < 500 ms, dominado pelo tempo do servidor |
| `render --all-tables` sobre 200 tabelas | < 50 ms de CPU além da leitura do schema |

**Regras que decorrem disto:**

- **Round-trips à base de dados são `O(1)` por tipo de objecto, nunca `O(n)` por tabela.** Ler 200 tabelas faz um punhado de queries ao `INFORMATION_SCHEMA` com `WHERE table_schema = ?`, e junta os resultados em memória. Um `N+1` no leitor de catálogo é um bug de desempenho, não uma questão de estilo.
- **Cada template é parseado uma única vez por processo.** O `Environment` do MiniJinja guarda o template compilado; `--all-tables` reutiliza-o para todos os objectos. Reparsear por iteração é proibido.
- **Nada de trabalho no arranque que não seja necessário ao comando invocado.** Sem inicialização estática pesada, sem construir o `Environment` para um `--help`, sem ligar à base de dados para um comando que não a use. Inicialização preguiçosa por defeito.
- **I/O agregado.** Escrita através de writers com buffer; nunca um syscall por linha.
- **Sem paralelismo especulativo.** Concorrência só entra com benefício medido, em benchmark, sobre carga representativa. Paralelizar porque é possível é proibido — o custo de sincronização e de arranque de threads é real, e em cargas pequenas perde.

### 2. Eficiência de recursos

O `tpl` não deve consumir mais recursos do que os estritamente necessários à operação em curso — memória, CPU, ficheiros abertos, ligações, dependências e tamanho de binário.

**Orçamento inicial de memória:** RSS de pico < 32 MiB para uma base de dados de 200 tabelas.

**Regras que decorrem disto:**

- **Alocação mínima nos caminhos quentes.** Emprestar em vez de clonar; `&str` em vez de `String`; `Cow<'_, str>` quando evita mesmo uma cópia. Um `.clone()` num caminho quente exige justificação.
- **Pré-dimensionar colecções** cuja cardinalidade é conhecida a partir do catálogo (`with_capacity`).
- **Streaming sempre que o comando o permita.** A memória deve escalar com o maior objecto individual, não com a base de dados inteira, em tudo o que não exija o documento completo.
- **Uma só ligação à base de dados**, aberta o mais tarde possível e fechada assim que a leitura termina. Sem pool: o processo é efémero e faz um punhado de queries.
- **Orçamento de dependências.** Cada crate tem de justificar a sua presença. Preferir a `std`. Antes de acrescentar uma dependência, verificar o que ela arrasta (`cargo tree`) e o que custa (`cargo bloat`, tempo de arranque). Uma dependência que só se usa para uma função trivial não entra.
- **Perfil de release** afinado no `Cargo.toml`: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`.

> **Decisão em aberto — driver MariaDB.** O `sqlx` é assíncrono e arrasta o runtime `tokio`, o que penaliza arranque, tamanho de binário e memória residual num processo efémero que faz meia dúzia de queries. Um driver síncrono é provavelmente a escolha certa face a estes requisitos. Resolver por medição — arranque, RSS e tamanho de binário, lado a lado — antes de escrever o leitor de catálogo, e actualizar a tabela do Stack com o resultado.

### Disciplina de medição

**Nenhuma afirmação de desempenho sem números, e nenhuma optimização sem medição antes e depois.** Intuição sobre o que é rápido não é evidência.

| Ferramenta | Uso |
|---|---|
| `hyperfine` | Wall time end-to-end da CLI, incluindo arranque |
| `criterion` | Micro-benchmarks de funções (parsing, mapeamento de tipos, render) |
| `dhat-rs` | Perfil de alocações e pico de heap |
| `samply` / `cargo flamegraph` | Atribuição de CPU a call sites |
| `cargo bloat` | Contribuição de cada crate para o tamanho do binário |

Os benchmarks vivem em `benches/` e correm contra o dataset do container MariaDB, para serem reproduzíveis. As baselines são registadas em `BENCHMARKS.md`; **uma regressão face à baseline reprova a alteração** e tem de ser justificada ou corrigida antes de o trabalho ser dado por concluído.

Para trabalho de optimização, usar o agente `rust-perf-engineer`; para investigação de causa-efeito entre implementação e comportamento medido (RAM, CPU, código vácuo ou redundante), usar `extreme-code-profiler`.

## Stack

| Área | Escolha | Notas |
|---|---|---|
| Linguagem | Rust (edition 2024, MSRV a fixar no `Cargo.toml`) | |
| CLI | `clap` v4 (derive) | Árvore de comandos, aliases, `--help` por subcomando |
| Templates | `minijinja` + `minijinja-contrib` | Runtime, sempre |
| Acesso MariaDB | Por decidir por medição | Ver a decisão em aberto nos Requisitos Não-Funcionais |
| Serialização | `serde` + `serde_json` | O contexto de render é `serde`-serializável |
| Configuração | `toml` + `serde` | |
| Erros | `thiserror` na biblioteca, `anyhow` no binário | |
| Logging | `tracing` + `tracing-subscriber` | Controlado por `-v/--verbose` |

Qualquer alteração a esta tabela é uma decisão de arquitectura e deve ser registada antes de ser implementada.

## Estrutura do Projecto

```
tpl/
├── Cargo.toml
├── src/
│   ├── main.rs              # entrypoint: parse, dispatch, mapeamento de exit codes
│   ├── cli/                 # árvore clap — um módulo por comando porcelain
│   │   ├── init.rs
│   │   ├── database.rs
│   │   ├── schema.rs
│   │   ├── template.rs
│   │   ├── render.rs
│   │   └── config.rs
│   ├── project/             # descoberta de .tpl/, leitura do `.cfg`, store de bases de dados
│   ├── mariadb/             # leitor de INFORMATION_SCHEMA (read-only)
│   ├── model/               # Database, Table, Column, Index, ForeignKey, View, Routine
│   ├── render/              # Environment MiniJinja, loader, filtros, testes, funções
│   └── error.rs             # tipo de erro + exit codes
├── templates/               # templates de arranque, copiados para .tpl/templates/ pelo `tpl init`
├── tests/                   # testes de integração (CLI end-to-end)
├── scripts/mariadb/         # Dockerfile, setup.sql, seed.sql
├── examples/                # pipelines completos: schema → template → output
└── specification/           # especificação funcional
```

O `model/` é a fronteira do projecto: é simultaneamente o resultado da introspecção e o **contexto de render**. Tudo o que um template pode ver está definido aí, e as suas structs são a superfície pública documentada.

## O Projecto `.tpl`

O `tpl` opera sobre **projectos**. Um projecto é qualquer directoria que contenha uma pasta `.tpl/`, exactamente como o `git` usa `.git/`. Essa pasta é a raiz do projecto e a **única** fonte de configuração e de templates.

```
<raiz-do-projecto>/
└── .tpl/
    ├── .cfg              # TOML — bases de dados disponíveis; fora do versionamento
    ├── .gitignore        # ignora o .cfg
    └── templates/        # templates disponíveis; versionados com o projecto
        └── example.jinja # template de exemplo, criado pelo `tpl init`
```

**Descoberta.** A partir da directoria corrente, subir na árvore até encontrar uma pasta `.tpl/`. A primeira encontrada define a raiz do projecto; a subida pára aí. Se não existir nenhuma, o comando falha com código `78` e sugere `tpl init`. A variável `TPL_DIR` aponta explicitamente para uma pasta `.tpl/` e dispensa a descoberta.

**Não existe configuração global.** Nada em `~`, nada em XDG, nada em `/etc`. Um comando que não encontre `.tpl/` não tem fallback: falha. Esta regra é deliberada — torna o comportamento do `tpl` inteiramente determinado pelo conteúdo do projecto, e portanto reproduzível entre máquinas e em CI.

### O ficheiro `.tpl/.cfg`

TOML. O nome do ficheiro é `.cfg` — um dotfile, começado por ponto, sem extensão. Declara as bases de dados disponíveis e as opções do projecto.

O ponto é deliberado: o `.cfg` destina-se a **ficar fora do versionamento**. Contém a configuração de acesso de cada programador — endereços, utilizadores, credenciais — que é local à máquina e não deve viajar no repositório.

```toml
[core]
database = "shop"          # base de dados usada quando -d/--database é omitido

[database.shop]
dsn = "mysql://alice:${SHOP_DB_PASSWORD}@db.example.com:3306/shop"

[database.reporting]
host     = "10.0.1.5"
port     = 3306
user     = "reader"
database = "reporting"
password_command = "security find-generic-password -s tpl-reporting -w"
tls      = "required"
```

Uma entrada `[database.<nome>]` define-se **ou** por `dsn`, **ou** pelos campos discretos (`host`, `port`, `user`, `password`, `database`). Misturar as duas formas na mesma entrada é erro de configuração (`78`). O nome da entrada é uma etiqueta do projecto, não o nome da base de dados no servidor.

### O que o `tpl init` cria

O `tpl init` produz um projecto **imediatamente utilizável**, não uma pasta vazia. Cria os quatro artefactos acima, e nenhum deles fica por preencher:

| Artefacto | Conteúdo |
|---|---|
| `.tpl/.cfg` | Configuração padrão: secção `[core]` e uma entrada `[database.*]` de exemplo, comentada, com a forma exacta que uma entrada real deve ter |
| `.tpl/.gitignore` | Uma linha, `.cfg` |
| `.tpl/templates/` | A directoria de templates do projecto |
| `.tpl/templates/example.jinja` | Um template de exemplo funcional |

O `.cfg` gerado não tem nenhuma entrada activa: por omissão o projecto não conhece base de dados nenhuma, e o utilizador descomenta ou acrescenta a sua. O exemplo comentado existe para que a forma correcta esteja à vista, sem obrigar a consultar documentação.

O `example.jinja` **tem de renderizar sem erro** contra qualquer tabela de qualquer base de dados MariaDB, e serve de demonstração viva do contexto de render: percorre colunas, usa um filtro, usa um teste, e traz no cabeçalho o comando que o executa. É verificado por um teste de integração contra o container MariaDB — um exemplo que não corre é pior do que exemplo nenhum.

O `init` **recusa-se a sobrepor** uma pasta `.tpl/` existente, e falha com `73` se não conseguir criar o destino. Não fundir, não completar parcialmente, não sobrescrever.

### Segredos e versionamento

A pasta `.tpl/` divide-se em duas metades com destinos opostos, e essa divisão é intencional:

| Caminho | Destino | Porquê |
|---|---|---|
| `.tpl/templates/` | **Versionado** | É o trabalho partilhado da equipa; deve viajar no repositório e ser revisto como qualquer outro código |
| `.tpl/.cfg` | **Fora do versionamento** | É configuração de acesso local a cada máquina, e pode conter credenciais |

**Um dotfile não é ignorado pelo `git` por ser dotfile** — o `.git/` é ignorado por ser o próprio repositório, não por causa do ponto. Para que a intenção se cumpra na prática, o `tpl init` escreve `.tpl/.gitignore` com o conteúdo:

```gitignore
.cfg
```

Isto torna a pasta `.tpl/` autocontida: quem a copiar para outro projecto leva consigo a regra de exclusão, sem depender do `.gitignore` da raiz.

Mesmo com o `.cfg` fora do repositório, existem dois mecanismos para não escrever passwords em disco:

- **`${VAR}` em qualquer valor string** é substituído pela variável de ambiente correspondente no momento em que a configuração é lida. Uma variável indefinida é erro (`78`), nunca uma substituição vazia silenciosa.
- **`password_command`** executa o comando indicado e usa o `stdout` (trimmed) como password.

O ficheiro é criado com permissões `0600`. Qualquer comando que imprima configuração — `tpl database show`, `tpl config list` — **redige sempre** passwords e o corpo de credenciais dentro de um DSN. Nunca registar credenciais em logs, mensagens de erro ou traços do `tracing`, seja qual for o nível de verbosidade.

### `.tpl/templates/`

Os templates disponíveis ao projecto. O nome de um template é o seu caminho relativo a `.tpl/templates/`, pelo que `rust/struct.jinja` refere `.tpl/templates/rust/struct.jinja`. A raiz do loader do MiniJinja é esta pasta, e é também a fronteira: `{% include %}`, `{% import %}` e `{% extends %}` resolvem dentro dela e **nunca** acima dela. Um caminho que escape da raiz é erro de render (`65`).

## Superfície da CLI

```
tpl [flags globais] <comando> [<subcomando>] [<args>] [flags]
```

### Comandos porcelain

Agrupados pelos três braços:

```
# Braço 1 — exploração da base de dados
tpl schema info|tables|table|views|view|routines|routine|dump

# Braço 2 — exploração dos templates
tpl template list|show|check|path

# Braço 3 — renderização (output para stdout)
tpl render <template> [alvo] [flags]

# Auxiliar — gestão do projecto
tpl init                                          # cria .tpl/ com .cfg, .gitignore e um template de exemplo
tpl database add|list|show|update|remove|test     # alias: db
tpl config get|set|unset|list                     # opera sobre .tpl/.cfg
tpl help [--format json]                          # árvore completa de comandos
tpl version
```

Cada comando recebe em argumentos e flags apenas os parâmetros da operação; tudo o resto — credenciais, opções do projecto, corpo dos templates — é lido de `.tpl/`.

### Flags globais

| Flag | Curta | Descrição |
|---|---|---|
| `--database <nome>` | `-d` | Entrada `[database.<nome>]` a usar; sobrepõe-se a `TPL_DATABASE` e a `core.database` |
| `--tpl-dir <path>` | | Pasta `.tpl/` explícita; dispensa a descoberta |
| `--output <path>` | `-o` | Escreve o resultado neste ficheiro em vez de `stdout` |
| `--format <text\|json>` | | Formato de saída dos comandos de leitura |
| `--verbose` | `-v` | Repetível; controla o nível do `tracing` |
| `--quiet` | `-q` | Suprime tudo excepto erros |
| `--no-color` | | Desliga cor; implícito quando `stdout` não é TTY ou `NO_COLOR` está definido |

Variáveis de ambiente: `TPL_DIR`, `TPL_DATABASE`, `NO_COLOR`.

### Regras de design da CLI

- Todo o comando de leitura suporta `--format json` com **saída estável** — é o contrato *plumbing* e não pode mudar sem versionamento.
- `tpl schema dump` produz exactamente o mesmo JSON que o `render` recebe como contexto. `tpl render --context <ficheiro.json>` aceita-o de volta. Este round-trip tem de ser mantido: permite renderizar sem base de dados.
- Aliases seguem o padrão do `git`: curtos, previsíveis, documentados na tabela do README. Nunca ambíguos.
- Erros vão para `stderr`, resultado vai para `stdout`. Um comando cuja saída seja consumível por pipe nunca escreve ruído em `stdout`.
- **Consola por defeito, ficheiro só a pedido.** Todo o comando que produza resultado escreve em `stdout`, a menos que receba `--output` (ou, no `render`, `--output-dir`). Não existe forma de configurar um destino por omissão diferente da consola: seria um efeito lateral invisível na linha de comando.
- **Escrita atómica.** Havendo flag de destino, o conteúdo é escrito num ficheiro temporário na mesma directoria e renomeado por cima do alvo. Uma falha a meio nunca deixa um ficheiro truncado no lugar do bom.
- **Escrever em ficheiro não altera o resto do contrato.** O `stdout` fica vazio, o exit code continua a ser `0`, e não se imprime confirmação nenhuma.
- **Nenhum comando aceita password em `argv`.** As credenciais resolvem-se sempre a partir da entrada nomeada em `.tpl/.cfg`.
- `tpl init`, `tpl database …` e `tpl config …` são os únicos comandos que escrevem em `.tpl/`. Todos os outros tratam a pasta como só de leitura.

## Contexto de Render

Variáveis de topo disponíveis em qualquer template:

| Variável | Conteúdo |
|---|---|
| `database` | Metadados da base de dados e as colecções `tables`, `views`, `routines` |
| `table` / `view` / `routine` | Presente apenas quando o render está limitado a um objecto |
| `vars` | Mapa das flags `--set chave=valor` |
| `tpl` | `{ version }` |
| `now` | Timestamp UTC do render |

Filtros, testes e funções registados no `Environment` são **superfície pública**: acrescentar é permitido, renomear ou remover é uma quebra de compatibilidade e exige entrada no CHANGELOG.

- Filtros de nomenclatura: `pascal`, `camel`, `snake`, `upper_snake`, `kebab`, `plural`, `singular`
- Filtros de SQL/código: `quote` (identificador MariaDB com crases), `sql_type`, `rust_type`, `json`, `indent`, `comment`
- Testes: `nullable`, `primary_key`, `auto_increment`, `unique`, `numeric`, `temporal`, `textual`

Qualquer filtro que dependa do motor tem de ser correcto para MariaDB e validado contra uma instância real (ver secção seguinte).

## Exit Codes e Mensagens de Erro

O exit code é o canal mais fiável que um agente tem para saber o que aconteceu. Por isso **cada condição distinta tem um código distinto** e accionável: o código sozinho deve bastar para escolher o passo seguinte. Convenção `sysexits.h`.

| Código | Nome | Condição | O que o chamador deve fazer |
|---|---|---|---|
| `0` | `EX_OK` | Sucesso | Continuar |
| `64` | `EX_USAGE` | Comando ou flag desconhecida, argumento obrigatório em falta, flags mutuamente exclusivas | Corrigir a invocação; consultar `--help` |
| `65` | `EX_DATAERR` | Erro de sintaxe no template, falha de render, JSON de `--context` malformado | Corrigir o template ou o contexto |
| `66` | `EX_NOINPUT` | O objecto nomeado não existe: template, tabela, vista, rotina, entrada de base de dados | Listar o que existe e escolher outro nome |
| `69` | `EX_UNAVAILABLE` | Servidor inalcançável: DNS, ligação recusada, timeout, falha de TLS | Verificar host/rede; a operação é read-only, logo repetível |
| `70` | `EX_SOFTWARE` | Erro interno do `tpl` — um bug | Reportar; não é corrigível pelo chamador |
| `73` | `EX_CANTCREAT` | `tpl init` não consegue criar `.tpl/` | Verificar permissões do directório |
| `74` | `EX_IOERR` | Falha de I/O a ler `.tpl/` ou a escrever em `stdout` | Verificar permissões e espaço |
| `77` | `EX_NOPERM` | Autenticação recusada, ou privilégios insuficientes sobre `INFORMATION_SCHEMA` | Corrigir credenciais ou pedir `SELECT` |
| `78` | `EX_CONFIG` | Sem `.tpl/`; `.cfg` malformado; entrada inválida; `${VAR}` indefinida; sessão read-only não imposta | Corrigir `.tpl/.cfg` ou correr `tpl init` |

Estes códigos são contrato. **Cada um tem de ter um teste de integração que o exercite**, e o teste faz parte da definição de pronto da funcionalidade que o pode produzir.

`EPIPE` em `stdout` — o caso de `tpl … | head` — termina o processo em silêncio com `0`. Não é erro.

### Formato das mensagens de erro

Uma mensagem de erro tem de responder a três perguntas, por esta ordem, e não a mais nenhuma: **o que falhou**, **porquê**, e **o que fazer a seguir**. Vai toda para `stderr`.

```
error: table 'ordrs' does not exist in database 'shop'
cause: no row in INFORMATION_SCHEMA.TABLES matches table_schema='shop' and table_name='ordrs'
hint: did you mean 'orders'? list the available tables with: tpl -d shop schema tables
exit: 66 (EX_NOINPUT)
```

Regras:

- **`hint` contém, sempre que possível, um comando concreto e executável.** É a linha que um agente aproveita para se auto-corrigir, e é a que mais retorno dá.
- **Sugestão por proximidade** (distância de edição) para nomes errados de tabela, vista, rotina, template e entrada de base de dados. Um typo é a falha mais frequente de um agente e a mais barata de resolver.
- **`cause` é factual e específico** — a query que não devolveu linhas, o erro do servidor, a linha e coluna do template. Nunca uma repetição da linha `error`.
- **Localização exacta** em erros de template: nome, linha, coluna, e a cadeia de `minijinja::Error`.
- **Nunca conselhos vagos** do género "verifique a sua configuração". Dizer qual chave, em que ficheiro, com que valor esperado.
- **Nunca credenciais** em mensagens de erro, seja qual for o nível de verbosidade.
- Com `--format json`, o erro sai igualmente em JSON, com o mesmo conteúdo:

```json
{"error":{"exit":66,"code":"EX_NOINPUT","kind":"table_not_found","message":"table 'ordrs' does not exist in database 'shop'","cause":"no row in INFORMATION_SCHEMA.TABLES matches table_schema='shop' and table_name='ordrs'","hint":"tpl -d shop schema tables","did_you_mean":["orders"]}}
```

O campo `kind` é um identificador estável e enumerado, pensado para ser comparado por código; `message` é para ser lido. Acrescentar valores a `kind` é permitido, renomeá-los é quebra de compatibilidade.

## Desenvolvimento

```bash
cargo build
cargo run -- --help
cargo test
```

### Pipeline de validação obrigatório

Nenhum trabalho está concluído sem que todos estes comandos passem, por esta ordem:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --all-features
cargo audit
```

Quando a alteração toque num caminho quente — leitura de catálogo, construção do contexto, render, arranque do processo — acresce a este pipeline a execução dos benchmarks e a comparação com a baseline em `BENCHMARKS.md`, conforme a disciplina de medição definida nos Requisitos Não-Funcionais.

`clippy` com `-D warnings` não é negociável. Um lint que se justifique suprimir exige `#[allow(...)]` local acompanhado de um comentário com a razão — nunca uma supressão ao nível do crate.

### Desenvolvimento em Rust

Usar **sempre** o agente `rust-elite-developer` para tarefas de programação em Rust: criar, editar, refactorizar ou rever código. Para trabalho de performance em caminhos quentes, usar `rust-perf-engineer`. Para análise de superfície de ataque (parsing de input não confiável, credenciais, execução de `password_command`), usar `security-researcher`.

`unsafe` é proibido neste projecto. Manter `#![forbid(unsafe_code)]` no topo do crate.

## Testes contra MariaDB

Testes ou validações que necessitem de uma base de dados real **têm** de usar o container definido em `scripts/mariadb/Dockerfile`. Lançar o container antes, terminá-lo depois. **Nunca** usar instâncias externas, mocks ou stubs como substituto.

Sempre que for preciso confirmar o conteúdo, a estrutura ou os tipos devolvidos por uma query ao `INFORMATION_SCHEMA`:

1. Lançar o container MariaDB a partir do Dockerfile.
2. Executar a query real e observar a resposta efectiva.
3. Confirmar o comportamento na documentação oficial do MariaDB.
4. Documentar o código em conformidade com o que foi observado **e** confirmado.
5. Terminar o container.

**É proibido** assumir, inferir ou documentar o comportamento do catálogo sem executar estes passos. Conhecimento genérico sobre MySQL não é suficiente: as divergências entre MariaDB e MySQL no `INFORMATION_SCHEMA` são reais e relevantes.

Os scripts `scripts/mariadb/setup.sql` e `seed.sql` devem cobrir exaustivamente a superfície que o `tpl` lê: todos os tipos de dados nativos do MariaDB, chaves primárias simples e compostas, índices únicos e compostos, chaves estrangeiras com regras `ON UPDATE`/`ON DELETE` distintas, colunas geradas, vistas, procedimentos e funções, triggers e comentários. O domínio modelado deve ser realista, nunca `id=1, name='test'`.

## Documentação

- Toda a documentação do projecto (README, doc comments, especificação, CHANGELOG) é escrita em **inglês**, com ortografia, gramática e sintaxe impecáveis.
- A linguagem deve ser técnica, clara e sem ambiguidades, destinada a leitores humanos.
- A documentação tem de ser **fiel ao código**: nunca descrever comandos, flags, filtros ou estruturas que não estejam implementados exactamente como descritos. Enquanto a implementação não existir, o README tem de o dizer explicitamente.
- Todo o item público (`pub`) leva doc comment. `#![warn(missing_docs)]` no crate.

## Especificação Funcional

Usar sempre o agente `specification-manager` para qualquer trabalho sobre a pasta `/specification`: criação, actualização, revisão ou auditoria. É o responsável exclusivo por essa pasta e pelos identificadores estáveis de requisito.

## Fluxo de Trabalho

1. **Especificar** — formalizar o requisito na especificação funcional.
2. **Implementar** — escrever o código que o satisfaz.
3. **Testar** — validar contra MariaDB real quando o requisito toque no catálogo.
4. **Documentar** — actualizar README, doc comments e CHANGELOG.

## Execução de Tarefas por Subagentes

**Nenhuma tarefa é executada directamente.** Sempre que uma tarefa começa, é delegada a um **subagente especializado**, escolhido por ser o mais adequado ao propósito dessa tarefa concreta.

### Escolha do subagente

O conjunto de subagentes disponíveis **é avaliado no momento**, no computador onde o trabalho decorre. Não existe aqui uma lista fixa, e não deve passar a existir: os agentes instalados mudam, e uma lista escrita neste ficheiro ficaria desactualizada sem dar sinal disso. Antes de delegar, verificar quem está efectivamente disponível e escolher em função do propósito da tarefa.

Se nenhum subagente for claramente o mais adequado, escolher o de propósito geral. A ausência de uma escolha óbvia **não** é autorização para executar directamente.

As indicações de agente já escritas noutras secções deste ficheiro — `rust-elite-developer` para código Rust, `rust-perf-engineer` e `extreme-code-profiler` para desempenho, `security-researcher` para superfície de ataque, `specification-manager` para a especificação — são casos particulares desta regra, não excepções a ela.

### Âmbito fechado

Cada tarefa é executada sob um **âmbito fechado, objectivo e focado exclusivamente no seu propósito**. O briefing entregue ao subagente é delimitado pelo que a tarefa define — **título, descrição, requisitos e comentários** — e por mais nada.

O subagente não alarga o âmbito, não aproveita a passagem para corrigir o que encontra pelo caminho, e não antecipa a tarefa seguinte. O que descobrir fora do âmbito regista-se como comentário ou como nova tarefa, através da skill `roadmap-manager`; não se executa.

### Um de cada vez — nunca em paralelo

**NUNCA correr mais do que um subagente em simultâneo.** Podem usar-se tantos quantos a tarefa exigir, mas sempre **em série**: lançar um, esperar que termine, avaliar o resultado, e só então lançar o seguinte.

Esta regra sobrepõe-se a qualquer heurística por defeito que favoreça paralelismo, incluindo o hábito de agrupar várias invocações independentes na mesma mensagem para correrem em concorrência, e a orquestração por workflows, que faz fan-out de agentes. Neste projecto o padrão é execução em série, e é o padrão que prevalece na dúvida.

Só o utilizador pode pedir execução em paralelo. Mesmo nesse caso é **excepcional**: cumpre-se para o pedido em causa e retoma-se de imediato o padrão em série. Um pedido de paralelismo não abre precedente para os pedidos seguintes.

## Skills Obrigatórias

Três domínios deste projecto **não** são operados directamente: têm uma skill dedicada que é a única via de acesso. A regra é a mesma nos três casos — **nunca invocar a CLI subjacente a partir do Bash**; invocar a skill, que a opera.

| Domínio | Skill | CLI que a skill opera |
|---|---|---|
| Escrita no Git | `gitflow` | `git` (operações de escrita) |
| Sprints, tarefas e comentários | `roadmap-manager` | `rmp` |
| Conhecimento sobre o projecto | `knowledge-authority` | `rmp graph …` |

### 1. Git — skill `gitflow`

**Todas as operações de escrita no Git passam pela skill `gitflow`.** Isto inclui, sem excepção: `commit`, `add`, criação e remoção de branches, `merge`, `rebase`, `tag`, `push`, `stash` e qualquer reescrita de histórico.

O repositório segue o modelo **gitflow** (Vincent Driessen): abertura e fecho de sprints, registo de uma tarefa fechada como commit, criação de releases e de hotfixes — tudo isso é decisão da skill, não improviso.

Leitura é livre: `git status`, `git log`, `git diff`, `git show` e equivalentes podem ser executados directamente, porque não alteram estado.

Não fazer commits directamente em `main`.

### 2. Sprints, tarefas e comentários — skill `roadmap-manager`

**Toda a gestão e coordenação de sprints, tarefas e comentários passa pela skill `roadmap-manager`**, que é a única operadora da CLI `rmp` (Groadmap). Isto abrange criar, listar, inspeccionar e editar tarefas; o ciclo de vida dos sprints (planear, iniciar, fechar, reabrir); transições de estado; prioridades e severidades; dependências e subtarefas; reordenação; o log de auditoria e as estatísticas; e o log tipado de comentários (`FINDING`, `HYPOTHESIS`, `TEST`, `DECISION`, `PROGRESS`, `UPDATE`, `NOTE`).

O `rmp` é a **única fonte de verdade** para o planeamento e execução das tarefas deste projecto. Não usar ficheiros ad-hoc, listas no chat, TODOs no código ou qualquer outra ferramenta como substituto.

**Roadmap deste projecto:** `tpl`.

### 3. Conhecimento do projecto — skill `knowledge-authority`

**Toda a consulta e gestão de conhecimento sobre o projecto passa pela skill `knowledge-authority`**, que é a única operadora de `rmp graph …` e do grafo de conhecimento que lhe está associado.

É o **primeiro sítio a consultar** perante qualquer pergunta factual sobre o código deste repositório — que ficheiro ou função implementa uma funcionalidade, que módulos, tipos, testes ou dependências existem, o que depende de quê, como se articula a arquitectura, qual o alcance e o impacto de uma alteração. Consultar o grafo **antes** de ler ficheiros, mesmo quando a resposta pareça estar a um `grep` de distância.

Usar também para sincronizar, refrescar ou auditar o grafo após um commit ou quando os ficheiros do projecto mudarem, e para manter o `./knowledge-model.md` — ficheiro escrito exclusivamente por esta skill.

### Fronteiras entre as três

As três skills não se substituem umas às outras, e a confusão entre elas é o erro habitual:

- Gerir tarefas e sprints é `roadmap-manager` — **nunca** `knowledge-authority`, ainda que ambas usem o binário `rmp`.
- Consultar o grafo de conhecimento é `knowledge-authority` — **nunca** `roadmap-manager`.
- Registar trabalho em Git é `gitflow` — **nunca** um `git commit` avulso, mesmo que a alteração seja trivial.
