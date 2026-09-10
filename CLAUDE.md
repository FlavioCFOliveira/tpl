# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

É **coordenação de agentes**: diz o que o projecto é, onde vive a verdade sobre ele, e como o trabalho é executado aqui. **Não contém requisitos funcionais.**

## Visão Geral

`tpl` é uma **aplicação de linha de comandos escrita em Rust** que lê a estrutura (DDL / schema) de bases de dados **MariaDB** e aplica **templates MiniJinja** para produzir texto ou código a partir dessa estrutura.

O modelo de interacção é **inspirado no `git`**: um único executável, subcomandos, aliases curtos, e comandos de leitura com saída estável, pensados para composição em pipelines.

**Âmbito do produto.** O `tpl` explora a base de dados, explora os templates do projecto, e renderiza um a partir do outro. É **read-only** sobre a base de dados, e a única escrita que faz em disco é a que lhe for explicitamente pedida na linha de comando, além da gestão da própria pasta de projecto. Uma funcionalidade que não caiba nisto não pertence ao `tpl`.

**Consumidor primário: agentes de IA.** O utilizador esperado não é uma pessoa numa shell interactiva — é um agente (Claude Code, Codex e equivalentes) a invocar a ferramenta programaticamente e a ler o resultado. Um agente dispõe de três canais para perceber uma CLI, e só três: o **texto de ajuda**, o **exit code** e o **stdout/stderr**. Isto é um requisito de design de primeira ordem e é a razão de a ajuda, os exit codes, os formatos de saída e o determinismo serem tratados como contrato. **O conteúdo desse contrato é da especificação, não deste ficheiro.**

## Fontes de Verdade

O projecto tem três fontes de verdade, com âmbitos que não se sobrepõem. Confundi-las é o erro previsível:

| Fonte | Responde a | Dono |
|---|---|---|
| `/specification` | **O que** o `tpl` faz — requisitos, regras, casos de uso | subagente `specification-manager` |
| `rmp` | **Quando e por quem** — sprints, tarefas, estado, decisões | skill `roadmap-manager` |
| Knowledge Graph | **Onde e como** — que código existe e como se articula | skill `knowledge-authority` |

Uma tarefa no `rmp` implementa um requisito da especificação; não o substitui. Um facto no grafo descreve o código que existe; não legitima código que a especificação não pediu.

**Este ficheiro não é fonte de verdade funcional.** Não descreve comandos, flags, formatos de saída, exit codes, variáveis de contexto, filtros, nem a estrutura da configuração. Perante qualquer pergunta sobre *o que* o `tpl` faz, ler a especificação — nunca responder a partir deste ficheiro, e nunca reintroduzir aqui o que ela já diz. Uma segunda cópia deixa de ser verdadeira sem dar sinal disso.

### Onde procurar cada assunto

O índice completo e actual está em `specification/README.md`, e é por aí que se começa. Os assuntos que este ficheiro deixou de descrever estão distribuídos assim:

| Assunto | Ficheiro |
|---|---|
| Gramática de invocação, árvore de comandos, aliases, argumentos | `specification/cli-contract.md` |
| Flags globais, variáveis de ambiente, precedência de configuração | `specification/global-flags.md` |
| Formas de ajuda, estrutura do texto de ajuda, árvore de comandos em JSON, versão | `specification/help-and-version.md` |
| Exploração da base de dados | `specification/schema-commands.md` |
| Exploração dos templates | `specification/template-commands.md` |
| Renderização | `specification/render-command.md` |
| Cache do catálogo | `specification/cache-commands.md`, `specification/cache-documents.md` |
| Gestão da configuração e das entradas de base de dados | `specification/cfg-commands.md` |
| Ficheiro de configuração do projecto: chaves, tipos, expansão, definições de ligação | `specification/configuration-model.md` |
| A pasta do projecto, a sua descoberta e a inicialização | `specification/project-and-discovery.md` |
| Saída em texto e em JSON, envelope, codificação, destino do output | `specification/output-formats.md` |
| Exit codes, ordem de validação, formato das mensagens de erro, sugestões | `specification/errors-and-exit-codes.md` |
| Regras transversais de segurança e tratamento de credenciais | `specification/security.md` |
| O que entra no modelo a partir do catálogo, e o que fica de fora | `specification/catalogue-coverage.md` |
| Estrutura do documento que transporta o modelo — o contexto de render | `specification/context-document.md` |
| Filtros, testes e funções globais disponíveis ao template | `specification/template-environment.md` |
| Semântica de render | `specification/render-semantics.md` |
| Séries de MariaDB suportadas, diferenças entre séries, promessa de read-only | `specification/server-contract.md` |
| Leituras incompletas por privilégios insuficientes | `specification/privileges-and-completeness.md` |
| Propriedades de desempenho exigidas, cargas de referência, protocolo de medição | `specification/performance-requirements.md` |
| Termos, casos de uso ponta a ponta, questões em aberto | `specification/glossary.md`, `specification/use-cases.md`, `specification/open-questions.md` |

## Regras Inegociáveis

Estas regras não se ponderam caso a caso. Cada uma tem uma secção que a desenvolve; a tabela existe para que nenhuma se perca por estar longe.

| Regra | Se estiver prestes a quebrá-la |
|---|---|
| Só se trabalha sobre **tarefa aberta**: no sprint `OPEN` e em `DOING` com `--commit-open` | **PARAR** e abri-la pela skill `roadmap-manager` |
| **Nenhuma tarefa é executada directamente** — delega-se a um subagente, e a cada peça de trabalho o seu | **PARAR** e escolher o subagente |
| **Um subagente de cada vez**, nunca em paralelo | **PARAR** e serializar. Só o utilizador autoriza paralelo, e só para o pedido em causa |
| Escrita no Git **só** pela skill `gitflow` | **PARAR**. Nunca um `git commit` avulso, por trivial que seja |
| Tarefas, sprints e comentários **só** pela skill `roadmap-manager` | **PARAR**. Nunca `rmp` invocado do Bash |
| Conhecimento sobre o código **só** pela skill `knowledge-authority` | **PARAR**. Nunca `rmp graph …` directamente |
| A **especificação precede a implementação**, e `/specification` só é escrita pelo subagente `specification-manager` | **PARAR** e formalizar o requisito primeiro |
| **Nada se lê, lista ou referencia fora da raiz do repositório** | **PARAR**. O que faltar pergunta-se ao utilizador |
| **`unsafe` é proibido**; `#![forbid(unsafe_code)]` mantém-se no topo do crate | **PARAR** e resolver em Rust seguro |
| O **pipeline de validação obrigatório** passa antes de o trabalho estar concluído | O trabalho **não está concluído**. Corrigir e repetir |
| O `tpl` **nunca escreve na base de dados** | **PARAR**. A leitura do catálogo é só por `INFORMATION_SCHEMA` |
| Templates carregam-se **em runtime**, nunca embebidos em tempo de compilação | **PARAR**. Motores compile-time estão excluídos |
| Validação que precise de base de dados usa os **containers do projecto** | **PARAR** e lançar o container. Nunca mocks nem instâncias externas |

## Antes de Começar

Correr esta verificação antes de qualquer trabalho. Um "não" em qualquer ponto interrompe o trabalho — não o autoriza a prosseguir com uma nota.

1. **O requisito está na especificação?** Se não estiver, ou estiver ambíguo, incompleto ou contraditório — **PARAR**. A primeira tarefa é levá-lo ao subagente `specification-manager`.
2. **Existe tarefa no `rmp` para este trabalho?** Se não — **PARAR** e criá-la pela skill `roadmap-manager`. Não se executa trabalho sem tarefa.
3. **A tarefa está no sprint `OPEN`?** Se está em `BACKLOG`, ou num sprint `PENDING` ou `CLOSED` — **PARAR**. Trazê-la para o sprint aberto é acção de planeamento e **exige confirmação do utilizador**.
4. **A tarefa está em `DOING`, aberta com `--commit-open <hash>`?** Se não, abrir agora, com o hash real de `git rev-parse HEAD`.
5. **Que subagente executa cada peça do trabalho?** Decompor a tarefa e escolher por peça, avaliando os agentes efectivamente instalados. Sem especialista óbvio, o de propósito geral — **nunca** execução directa.
6. **O âmbito está fechado?** O briefing é o que a tarefa define — título, descrição, requisitos, comentários — e mais nada. O que se descobrir fora dele **regista-se; não se executa**.
7. **Durante o trabalho**, escrever o log à medida: `DECISION` com as opções rejeitadas, `FINDING` com o que se descobriu, `TEST` com a verificação e o resultado.
8. **Ao fechar**, commit primeiro pela skill `gitflow`, depois `--commit-close <hash>` com o hash real desse commit.

## Execução de Tarefas por Subagentes

**Nenhuma tarefa é executada directamente.** Sempre que uma tarefa começa, é delegada a um **subagente especializado**, escolhido por ser o mais adequado ao propósito dessa tarefa concreta. Dar por si a executar directamente é motivo para **PARAR** e delegar.

### Escolha do subagente

O conjunto de subagentes disponíveis **é avaliado no momento**, no computador onde o trabalho decorre. Não existe aqui uma lista fixa, e **NUNCA** passará a existir: os agentes instalados mudam, e uma lista escrita neste ficheiro ficaria desactualizada sem dar sinal disso. Antes de delegar, verificar **SEMPRE** quem está efectivamente disponível e escolher em função do propósito da tarefa.

Se nenhum subagente for claramente o mais adequado, escolher o de propósito geral. A ausência de uma escolha óbvia **NUNCA** é autorização para executar directamente.

As indicações de agente já escritas noutras secções deste ficheiro — `rust-elite-developer` para código Rust, `rust-perf-engineer` e `extreme-code-profiler` para desempenho, `security-researcher` para superfície de ataque, `specification-manager` para a especificação — são casos particulares desta regra, não excepções a ela.

### A regra vale para cada peça de trabalho

**A delegação não é só por tarefa — é por cada peça de trabalho dentro dela.** Aberta a tarefa, o trabalho que ela contém é **decomposto**, e para **cada peça** escolhe-se o subagente mais adequado, avaliado contra os agentes efectivamente instalados na máquina nesse momento, nos termos da secção anterior.

Uma tarefa que atravesse vários tipos de trabalho — especificação, código, testes, desempenho, segurança, documentação — usa **vários subagentes**, um por tipo, e **SEMPRE em série**, nos termos de **Um de cada vez — nunca em paralelo**, mais abaixo.

**A escolha do subagente de cada peça é registada na tarefa**, através da skill `roadmap-manager`, para que a execução fique rastreável a quem a fez.

Por peça valem, sem alteração, as duas regras de **Escolha do subagente**: avaliar os agentes efectivamente instalados no momento, e recorrer ao de propósito geral quando não houver especialista óbvio.

### Âmbito fechado

Cada tarefa é executada sob um **âmbito fechado, objectivo e focado exclusivamente no seu propósito**. O briefing entregue ao subagente é delimitado pelo que a tarefa define — **título, descrição, requisitos e comentários** — e por mais nada.

O subagente **NUNCA** alarga o âmbito, **NUNCA** aproveita a passagem para corrigir o que encontra pelo caminho, e **NUNCA** antecipa a tarefa seguinte. O que descobrir fora do âmbito regista-se como comentário ou como nova tarefa, através da skill `roadmap-manager`; não se executa. Perante a tentação de o corrigir já, **PARAR** e registar.

### Um de cada vez — nunca em paralelo

**NUNCA correr mais do que um subagente em simultâneo.** Podem usar-se tantos quantos a tarefa exigir, mas SEMPRE **em série**: lançar um, esperar que termine, avaliar o resultado, e só então lançar o seguinte. Dar por si prestes a lançar dois — **PARAR** e serializar.

Esta regra sobrepõe-se a qualquer heurística por defeito que favoreça paralelismo, incluindo o hábito de agrupar várias invocações independentes na mesma mensagem para correrem em concorrência, e a orquestração por workflows, que faz fan-out de agentes. Neste projecto o padrão é execução em série, e é o padrão que prevalece na dúvida.

Só o utilizador pode pedir execução em paralelo. Mesmo nesse caso é **excepcional**: cumpre-se para o pedido em causa e retoma-se de imediato o padrão em série. Um pedido de paralelismo não abre precedente para os pedidos seguintes.

## Skills Obrigatórias

Três domínios deste projecto **NUNCA** são operados directamente: têm uma skill dedicada que é a única via de acesso. A regra é a mesma nos três casos — **NUNCA invocar a CLI subjacente a partir do Bash**; invocar a skill, que a opera. Estar prestes a escrever uma destas CLI no Bash é motivo para **PARAR** e chamar a skill.

| Domínio | Skill | CLI que a skill opera |
|---|---|---|
| Escrita no Git | `gitflow` | `git` (operações de escrita) |
| Sprints, tarefas e comentários | `roadmap-manager` | `rmp` |
| Conhecimento sobre o projecto | `knowledge-authority` | `rmp graph …` |

### 1. Git — skill `gitflow`

**Todas as operações de escrita no Git passam pela skill `gitflow`.** Isto inclui, sem excepção: `commit`, `add`, criação e remoção de branches, `merge`, `rebase`, `tag`, `push`, `stash` e qualquer reescrita de histórico.

O repositório segue o modelo **gitflow** (Vincent Driessen): abertura e fecho de sprints, registo de uma tarefa fechada como commit, criação de releases e de hotfixes — tudo isso é decisão da skill, não improviso.

Leitura é livre: `git status`, `git log`, `git diff`, `git show` e equivalentes podem ser executados directamente, porque não alteram estado.

**NUNCA** fazer commits directamente em `main`.

### 2. Sprints, tarefas e comentários — skill `roadmap-manager`

**Toda a gestão e coordenação de sprints, tarefas e comentários passa pela skill `roadmap-manager`**, que é a única operadora da CLI `rmp` (Groadmap). Isto abrange criar, listar, inspeccionar e editar tarefas; o ciclo de vida dos sprints (planear, iniciar, fechar, reabrir); transições de estado; prioridades e severidades; dependências e subtarefas; reordenação; o log de auditoria e as estatísticas; e o log tipado de comentários (`FINDING`, `HYPOTHESIS`, `TEST`, `DECISION`, `PROGRESS`, `UPDATE`, `NOTE`).

O `rmp` é a **única fonte de verdade** para o planeamento e execução das tarefas deste projecto. **NUNCA** usar ficheiros ad-hoc, listas no chat, TODOs no código ou qualquer outra ferramenta como substituto.

**Roadmap deste projecto:** `tpl`.

#### Nenhum trabalho fora de uma tarefa aberta

**Todo o trabalho é executado sobre uma tarefa aberta no `rmp`, e só sobre ela.** Começar trabalho numa tarefa que não foi aberta é uma **violação de processo**, não um atalho — e não se justifica por a alteração ser pequena, óbvia ou urgente.

**Os dois conjuntos de estados não se confundem.** `OPEN` é estado de *sprint*, nunca de tarefa:

| Entidade | Estados |
|---|---|
| Tarefa | `BACKLOG`, `SPRINT`, `DOING`, `TESTING`, `COMPLETED` |
| Sprint | `PENDING`, `OPEN`, `CLOSED` |

Uma tarefa é executável quando **ambas** as condições se verificam:

1. Pertence ao sprint cujo estado é `OPEN`.
2. Foi ela própria transitada para `DOING` com `--commit-open <hash>`.

Falhar uma das duas basta para a tarefa **não** ser executável. Em concreto:

- Uma tarefa em `BACKLOG` **NUNCA** se executa.
- Uma tarefa num sprint `PENDING` ou `CLOSED` **NUNCA** se executa.
- Trazer uma tarefa para o sprint aberto é uma **acção de planeamento** e **exige confirmação do utilizador**. **NUNCA** se faz por iniciativa própria para desbloquear trabalho.

Falhando qualquer uma das duas condições, **PARAR** e regularizar a tarefa pela skill `roadmap-manager` antes de escrever a primeira linha.

**Trabalho descoberto a meio que não tenha tarefa própria regista-se como tarefa ou como comentário — nunca se executa.** É **Âmbito fechado**, em *Execução de Tarefas por Subagentes*, e vale aqui sem alteração.

O ciclo de vida da tarefa **NUNCA** é excepção à regra do topo desta secção: toda a transição de estado e todo o comentário passam pela skill `roadmap-manager`.

#### Fecho: primeiro o commit, depois a tarefa

O gate de commit impõe a ordem, e a ordem não se inverte:

1. O trabalho é **registado em commit**, através da skill `gitflow`.
2. A tarefa é **fechada** com `--commit-close <hash>`, com o hash real desse commit.

O hash vem do commit efectivamente criado. **NUNCA inventar, adivinhar ou reaproveitar um hash** para satisfazer o gate: sem commit criado, **PARAR** e criá-lo pela skill `gitflow`.

#### O log escreve-se durante o trabalho

O log tipado de comentários é escrito **à medida que o trabalho acontece**, não reconstruído no fim:

| Tipo | Quando se escreve |
|---|---|
| `DECISION` | Toda a escolha não óbvia, com as opções rejeitadas e a razão da rejeição |
| `FINDING` | O que foi descoberto |
| `TEST` | A verificação feita e o resultado que deu |

Uma tarefa cujo trabalho envolveu escolhas e cujo log está vazio **não está pronta para fechar**: **PARAR** e escrever o log antes do fecho.

### 3. Conhecimento do projecto — skill `knowledge-authority`

**Toda a consulta e gestão de conhecimento sobre o projecto passa pela skill `knowledge-authority`**, que é a única operadora de `rmp graph …` e do grafo de conhecimento que lhe está associado.

É o **primeiro sítio a consultar** perante qualquer pergunta factual sobre o código deste repositório — que ficheiro ou função implementa uma funcionalidade, que módulos, tipos, testes ou dependências existem, o que depende de quê, como se articula a arquitectura, qual o alcance e o impacto de uma alteração. Consultar **SEMPRE** o grafo **antes** de ler ficheiros, mesmo quando a resposta pareça estar a um `grep` de distância. Estar prestes a abrir um ficheiro para responder a uma pergunta factual sem ter consultado o grafo é motivo para **PARAR** e consultá-lo.

Usar também para sincronizar, refrescar ou auditar o grafo após um commit ou quando os ficheiros do projecto mudarem, e para manter o `./knowledge-model.md` — ficheiro escrito exclusivamente por esta skill.

### Fronteiras entre as três

As três skills não se substituem umas às outras, e a confusão entre elas é o erro habitual:

- Gerir tarefas e sprints é `roadmap-manager` — **nunca** `knowledge-authority`, ainda que ambas usem o binário `rmp`.
- Consultar o grafo de conhecimento é `knowledge-authority` — **nunca** `roadmap-manager`.
- Registar trabalho em Git é `gitflow` — **nunca** um `git commit` avulso, mesmo que a alteração seja trivial.

## Especificação Funcional

**Todo o trabalho assenta numa especificação.** Nenhuma funcionalidade é implementada, alterada ou removida sem estar primeiro descrita na especificação funcional do projecto, em `/specification`.

A especificação é a **única fonte de verdade funcional** do `tpl`: o que lá não está não é requisito, e **NUNCA** se implementa por iniciativa própria, por analogia com outro projecto, por parecer óbvio ou por estar mencionado de passagem numa conversa. Perante um pedido que a especificação não cobre, **PARAR** e formalizá-lo primeiro.

### Coordenação

A coordenação da especificação é responsabilidade **exclusiva** do subagente `specification-manager`. É o único que escreve em `/specification`, e todo o trabalho sobre essa pasta passa por ele:

- bootstrap da estrutura e manutenção do `README.md` de navegação;
- criação e alteração de módulos funcionais, regras de negócio, actores, casos de uso e glossário;
- atribuição e manutenção dos identificadores estáveis de requisito;
- detecção de inconsistências, ambiguidades, contradições e lacunas entre ficheiros;
- manutenção das referências cruzadas;
- depreciação e arquivo de conteúdo obsoleto;
- auditorias estruturadas.

**Nenhum outro agente edita `/specification`** — nem sequer para corrigir uma gralha.

### A especificação precede a implementação

Sem excepção:

1. Chega o pedido. Verificar se está **coberto pela especificação**.
2. Se não estiver — ou se estiver de forma ambígua, incompleta ou contraditória — **PARAR**: a **primeira** tarefa é levar o subagente `specification-manager` a formalizá-lo. Só depois se implementa.
3. Implementar exactamente o que a especificação define: nem menos, nem mais.
4. Se durante a implementação se descobrir que a especificação está errada ou incompleta, **PARAR**. Corrigi-la através do subagente `specification-manager` e só então retomar. **NUNCA** implementar contra a especificação, e **NUNCA** deixar código e especificação a divergir.

Esta é a etapa 1 do Fluxo de Trabalho, e a razão de ser a primeira.

## Fluxo de Trabalho

1. **Especificar** — formalizar o requisito na especificação funcional, através do subagente `specification-manager`. Nenhuma etapa seguinte começa sem esta.
2. **Implementar** — escrever o código que o satisfaz.
3. **Testar** — validar contra MariaDB real quando o requisito toque no catálogo.
4. **Documentar** — actualizar README, doc comments e CHANGELOG.

As etapas correm por esta ordem e **NUNCA** se salta nenhuma. Dar por si a implementar sem a etapa 1 cumprida — **PARAR** e voltar a ela.

## Âmbito de Trabalho

**O âmbito deste projecto restringe-se à pasta de raiz do repositório.**

**NUNCA** ler, listar, pesquisar, inspeccionar ou referenciar nada fora de `tpl/`. Isto inclui directorias irmãs, outros projectos do utilizador, e qualquer caminho acima da raiz do repositório. **NUNCA** há excepções para "procurar convenções", "ver como outro projecto resolveu isto" ou "confirmar um padrão".

Toda a informação necessária ao trabalho vive dentro deste repositório. Se algo não estiver aqui, **PARAR** e perguntar ao utilizador — **NUNCA** se vai procurar lá fora.

## Invariantes de Implementação

Duas restrições permanentes sobre as escolhas de implementação. Têm prioridade sobre qualquer outra consideração de design, e nenhuma se negoceia caso a caso.

### 1. Read-only absoluto

O `tpl` **nunca** emite DDL, DML ou qualquer statement de escrita contra a base de dados a que se liga, e não existe flag que desligue este comportamento. A leitura do catálogo é feita através do `INFORMATION_SCHEMA` — nunca através de `mysqldump` ou de qualquer processo externo.

O detalhe da garantia — como o modo read-only é imposto na sessão, o que acontece quando não pode ser imposto, e que statements são admissíveis — pertence a `specification/server-contract.md` e lê-se lá.

### 2. Renderização em runtime, com MiniJinja

Os templates são **ficheiros em disco, carregados e compilados no momento do render** — nunca embebidos no binário em tempo de compilação. O motor é o **[MiniJinja](https://docs.rs/minijinja)** (`minijinja` + `minijinja-contrib`).

**Não usar Askama, Tera, Handlebars, `std::fmt`, nem qualquer outro motor.** O Askama em particular está explicitamente excluído por ser *compile-time*: obrigaria a recompilar o `tpl` sempre que um template mudasse, o que contraria a razão de ser desta ferramenta.

A configuração exigida ao `Environment` — loader, comportamento perante variáveis indefinidas, auto-escape, fronteira de resolução de caminhos e conteúdo de um erro de render — pertence a `specification/template-environment.md` e `specification/render-semantics.md`.

## Stack

Registo das escolhas tecnológicas vinculativas. Qualquer alteração a esta tabela é uma decisão de arquitectura e deve ser registada antes de ser implementada.

| Área | Escolha | Notas |
|---|---|---|
| Linguagem | Rust (edition 2024, MSRV a fixar no `Cargo.toml`) | |
| CLI | `clap` v4 (derive) | Árvore de comandos, aliases, `--help` por subcomando |
| Templates | `minijinja` + `minijinja-contrib` | Runtime, sempre |
| Acesso MariaDB | **Decisão em aberto** | Por resolver por medição |
| Serialização | `serde` + `serde_json` | O contexto de render é `serde`-serializável |
| Configuração | `toml` + `serde` | |
| Erros | `thiserror` na biblioteca, `anyhow` no binário | |
| Logging | `tracing` + `tracing-subscriber` | Controlado pela flag de verbosidade |

> **Decisão em aberto — driver MariaDB.** Um driver assíncrono arrasta um runtime que penaliza arranque, tamanho de binário e memória residual num processo efémero que faz um punhado de queries; um driver síncrono é provavelmente a escolha certa. Resolver **por medição** — arranque, RSS e tamanho de binário, lado a lado — antes de escrever o leitor de catálogo, e actualizar esta tabela com o resultado.

## Plataformas Suportadas

O `tpl` é uma ferramenta **Unix**. É essa a família de sistemas para que se escreve, e é a fronteira que delimita o que o código pode assumir.

Os sistemas suportados e verificados são o **Linux** e o **macOS**, nas arquitecturas **arm64** (`aarch64`, incluindo Apple Silicon) e **amd64** (`x86_64`). Outros Unix — os BSD, por exemplo — devem funcionar, porque nada no `tpl` depende de um sistema em concreto para além do que a `std` já abstrai, mas **não são testados nem garantidos**: não entram na matriz, não correm em validação, e um problema que só neles se manifeste não reprova uma alteração.

**O Windows está fora do âmbito.** Não é alvo, não se escreve código para o acomodar, e não se aceita uma dependência por causa dele.

A matriz concreta de alvos — target triples, escolha de libc, linkagem e forma de empacotar o binário — é decisão de arquitectura em curso e **não se fixa aqui**.

### Regras que decorrem disto

- **Nenhum alvo é de segunda classe.** O que passa no pipeline de validação obrigatório definido em **Desenvolvimento** tem de passar em todos os alvos suportados; uma falha num deles reprova a alteração, seja qual for o alvo.
- **Código específico de plataforma é a excepção**, e fica isolado atrás de `#[cfg(unix)]`. `#[cfg(windows)]` não tem lugar no crate. As permissões restritivas exigidas ao ficheiro de configuração dependem de `std::os::unix::fs::PermissionsExt` e são, por si só, razão bastante para a fronteira ser o Unix.
- **Caminhos sempre por `Path` e `PathBuf`**, nunca por concatenação de strings com o separador escrito à mão.
- **Sem assumir características do CPU.** Nada de `target-cpu=native` no perfil de release: quebraria a portabilidade e a reprodutibilidade do binário distribuído.
- **Dependências têm de compilar e passar testes em todos os alvos.** Um crate que não suporte `aarch64` não entra — critério que acresce ao orçamento de dependências e não o substitui.
- **Uma baseline de desempenho identifica sempre o alvo em que foi medida.** Números medidos em alvos diferentes não se comparam entre si.

## Estrutura do Projecto

```
tpl/
├── Cargo.toml
├── src/
│   ├── main.rs              # entrypoint: parse, dispatch, mapeamento de exit codes
│   ├── cli/                 # árvore clap — um módulo por comando porcelain
│   ├── project/             # descoberta da pasta de projecto e leitura da configuração
│   ├── mariadb/             # leitor de INFORMATION_SCHEMA (read-only)
│   ├── model/               # o modelo lido do catálogo
│   ├── render/              # Environment MiniJinja, loader, filtros, testes, funções
│   └── error.rs             # tipo de erro + exit codes
├── templates/               # templates de arranque
├── tests/                   # testes de integração (CLI end-to-end)
├── benches/                 # benchmarks
├── scripts/mariadb/         # Dockerfile, setup.sql, seed.sql
├── examples/                # pipelines completos: schema → template → output
└── specification/           # especificação funcional
```

O `model/` é a fronteira do projecto: é simultaneamente o resultado da introspecção e o que um template vê. A sua forma não se decide no código — é a especificação que a fixa (`specification/catalogue-coverage.md` e `specification/context-document.md`); o `model/` implementa-a e as suas structs são a superfície pública documentada.

## Convenções de Código Rust

O código deste projecto — e a forma como está organizado — segue as **boas práticas e as convenções idiomáticas da linguagem Rust**. Não é uma preferência de estilo: é regra do projecto, e vale tanto para o que se escreve de novo como para o que se refactoriza.

### Organização

- **Módulos.** Nomes em `snake_case`, sem abreviaturas obscuras e sem repetir o nome do pai (`mariadb::reader`, nunca `mariadb::mariadb_reader`). Um módulo por conceito, alinhado com a árvore da secção anterior.
- **Um só estilo de ficheiro-módulo.** Usar sempre a forma `foo.rs` acompanhada da directoria `foo/`; **não** usar `mod.rs`. Misturar os dois estilos na mesma árvore é proibido.
- **Visibilidade mínima.** Por omissão tudo é privado. `pub(crate)` para o que atravessa módulos, `pub(super)` para o que só o pai precisa, e `pub` reservado ao que é genuinamente superfície pública — no essencial o `model/` e o tipo de erro.
- **Re-exports deliberados.** O `lib.rs` re-exporta uma API coerente com `pub use`; não se re-exporta um módulo inteiro só para poupar um caminho de `use`.
- **Biblioteca e binário separados.** A lógica vive na biblioteca e é testável sem lançar processo; o `main.rs` limita-se a fazer parse, despachar e mapear o erro para exit code.

### Nomenclatura

Conformidade com as [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) e a RFC 430:

- `snake_case` para funções, variáveis, campos e módulos; `UpperCamelCase` para tipos, traits e variantes; `SCREAMING_SNAKE_CASE` para constantes e estáticos.
- Convenções de conversão, escolhidas pelo custo: `as_` (empréstimo barato), `to_` (custa, aloca), `into_` (consome o receptor).
- Getters sem prefixo `get_`: `table.name()`, nunca `table.get_name()`.
- Iteradores pelo trio convencional: `iter`, `iter_mut`, `into_iter`.

### Tipos e erros

- **`Result` e `Option` com o operador `?`.** `unwrap`/`expect` fora de testes só com a invariante documentada na própria mensagem. O tipo de erro segue a tabela da Stack: `thiserror` na biblioteca, `anyhow` no binário.
- **Newtypes para invariantes** — um nome de tabela já validado não é uma `String` qualquer.
- **Genéricos ou `impl Trait`** em vez de indirecção desnecessária; `Box<dyn Trait>` só quando a heterogeneidade for real.
- **Conversões por `From`/`TryFrom`**, e não por funções avulsas quando o trait serve.
- **Traits da `std` implementados quando fazem sentido**: `Debug` sempre, e `Display`, `Default`, `FromStr`, `AsRef` onde o tipo o justifique.
- **Assinaturas em tipos emprestados** — `&str`, `&[T]`, `&Path` em parâmetros, nunca `String`, `Vec<T>` ou `PathBuf`. É também o que a regra de alocação mínima exige.

### Expressão

- **Iteradores e combinadores** em vez de loops indexados com acumulador mutável, quando não custem clareza nem desempenho.
- **Pattern matching exaustivo**, sem um `_ =>` que engula silenciosamente variantes futuras de um `enum` do próprio crate.
- **`#[non_exhaustive]`** nos tipos públicos que se prevê virem a crescer — tipicamente o `enum` de erro e as structs do `model/`.
- **Derives em vez de implementações manuais** sempre que sejam equivalentes.

### Ferramentas como árbitro

O `rustfmt` com a configuração por defeito é a autoridade de formatação, e o `clippy` com `-D warnings` é a autoridade de idiomática. Ambos correm no pipeline de validação obrigatório definido em **Desenvolvimento**, e a sua decisão não se discute caso a caso.

Na dúvida, a referência é a convenção da linguagem e as Rust API Guidelines — nunca o hábito trazido de outra linguagem.

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

Quando a alteração toque num caminho quente — leitura de catálogo, construção do contexto, render, arranque do processo — acresce a este pipeline a execução dos benchmarks e a comparação com a baseline em `BENCHMARKS.md`, conforme a **Disciplina de medição**.

`clippy` com `-D warnings` não é negociável. Um lint que se justifique suprimir exige `#[allow(...)]` local acompanhado de um comentário com a razão — nunca uma supressão ao nível do crate.

### Desenvolvimento em Rust

Usar **sempre** o agente `rust-elite-developer` para tarefas de programação em Rust: criar, editar, refactorizar ou rever código. Para trabalho de performance em caminhos quentes, usar `rust-perf-engineer`. Para análise de superfície de ataque (parsing de input não confiável, credenciais, execução de comandos externos), usar `security-researcher`.

`unsafe` é proibido neste projecto. Manter `#![forbid(unsafe_code)]` no topo do crate.

## Desempenho e Eficiência

O desempenho e a economia de recursos são requisitos de **primeira ordem**: pesam nas decisões de design tanto quanto a correcção funcional. Uma implementação correcta mas lenta, ou correcta mas gastadora, **não** satisfaz o requisito. O `tpl` é uma ferramenta de linha de comandos, invocada repetidamente e frequentemente dentro de loops e pipelines de build — cada milissegundo é pago muitas vezes.

**Os alvos numéricos não vivem aqui.** As propriedades exigidas, as cargas de referência e o protocolo de medição pertencem a `specification/performance-requirements.md`; as baselines efectivamente medidas vivem em `BENCHMARKS.md`.

### Regras de implementação

- **Os round-trips ao catálogo não escalam com o número de objectos.** Um `N+1` no leitor de catálogo é um bug de desempenho, não uma questão de estilo.
- **Cada template é parseado uma única vez por processo.** O `Environment` do MiniJinja guarda o template compilado e reutiliza-o; reparsear por iteração é proibido.
- **Nada de trabalho no arranque que não seja necessário ao comando invocado.** Sem inicialização estática pesada, sem construir o `Environment` para uma invocação que não renderiza, sem ligar à base de dados para um comando que não a use. Inicialização preguiçosa por defeito.
- **I/O agregado.** Escrita através de writers com buffer; nunca um syscall por linha.
- **Alocação mínima nos caminhos quentes.** Emprestar em vez de clonar; `&str` em vez de `String`; `Cow<'_, str>` quando evita mesmo uma cópia. Um `.clone()` num caminho quente exige justificação.
- **Pré-dimensionar colecções** cuja cardinalidade já é conhecida (`with_capacity`).
- **Streaming sempre que a operação o permita.** A memória deve escalar com o maior objecto individual, não com a base de dados inteira, em tudo o que não exija o documento completo.
- **Uma só ligação à base de dados**, aberta o mais tarde possível e fechada assim que a leitura termina. Sem pool: o processo é efémero e faz um punhado de queries.
- **Orçamento de dependências.** Cada crate tem de justificar a sua presença. Preferir a `std`. Antes de acrescentar uma dependência, verificar o que ela arrasta (`cargo tree`) e o que custa (`cargo bloat`, tempo de arranque). Uma dependência que só se usa para uma função trivial não entra.
- **Perfil de release** afinado no `Cargo.toml`: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`.
- **Sem paralelismo especulativo.** Concorrência só entra com benefício medido, em benchmark, sobre carga representativa. Paralelizar porque é possível é proibido — o custo de sincronização e de arranque de threads é real, e em cargas pequenas perde.

### Disciplina de medição

**Nenhuma afirmação de desempenho sem números, e nenhuma optimização sem medição antes e depois.** Intuição sobre o que é rápido não é evidência.

| Ferramenta | Uso |
|---|---|
| `hyperfine` | Wall time end-to-end da CLI, incluindo arranque |
| `criterion` | Micro-benchmarks de funções (parsing, mapeamento de tipos, render) |
| `dhat-rs` | Perfil de alocações e pico de heap |
| `samply` / `cargo flamegraph` | Atribuição de CPU a call sites |
| `cargo bloat` | Contribuição de cada crate para o tamanho do binário |

Os benchmarks vivem em `benches/` e correm contra o dataset dos containers MariaDB, para serem reproduzíveis. As baselines são registadas em `BENCHMARKS.md`, identificando o alvo em que foram medidas; **uma regressão face à baseline reprova a alteração** e tem de ser justificada ou corrigida antes de o trabalho ser dado por concluído.

Para trabalho de optimização, usar o agente `rust-perf-engineer`; para investigação de causa-efeito entre implementação e comportamento medido (RAM, CPU, código vácuo ou redundante), usar `extreme-code-profiler`.

## Testes contra MariaDB

Testes ou validações que necessitem de uma base de dados real **têm** de usar os containers definidos em `scripts/mariadb/`. Lançar os containers antes, terminá-los depois. **Nunca** usar instâncias externas, mocks ou stubs como substituto.

O `tpl` suporta **mais do que uma série de MariaDB**. Quais são, e o que uma diferença entre séries obriga, pertence a `specification/server-contract.md` e **não se copia para aqui**.

Sempre que for preciso confirmar o conteúdo, a estrutura ou os tipos devolvidos por uma query ao `INFORMATION_SCHEMA`:

1. Lançar o container de **cada série suportada**.
2. Executar a query real em cada uma e observar a resposta efectiva.
3. Confirmar o comportamento na documentação oficial do MariaDB.
4. Documentar o código em conformidade com o que foi observado **e** confirmado.
5. Terminar os containers.

**Uma divergência entre séries é ela própria um achado** e regista-se como tal. É a razão de o `tpl` ler mais do que uma versão, e é o que a especificação exige que seja tratado — não uma nota de rodapé.

**É proibido** assumir, inferir ou documentar o comportamento do catálogo sem executar estes passos. Conhecimento genérico sobre MySQL não é suficiente: as divergências entre MariaDB e MySQL no `INFORMATION_SCHEMA` são reais e relevantes.

Os scripts `scripts/mariadb/setup.sql` e `seed.sql` devem cobrir exaustivamente a superfície que o `tpl` lê: todos os tipos de dados nativos do MariaDB, chaves primárias simples e compostas, índices únicos e compostos, chaves estrangeiras com regras `ON UPDATE`/`ON DELETE` distintas, colunas geradas, vistas, procedimentos e funções, triggers e comentários. O domínio modelado deve ser realista, nunca `id=1, name='test'`. O DDL tem de ser aceite por **todas** as séries suportadas.

## Documentação

- Toda a documentação do projecto (README, doc comments, especificação, CHANGELOG) é escrita em **inglês**, com ortografia, gramática e sintaxe impecáveis.
- A linguagem deve ser técnica, clara e sem ambiguidades, destinada a leitores humanos.
- A documentação tem de ser **fiel ao código**: nunca descrever comandos, flags, filtros ou estruturas que não estejam implementados exactamente como descritos. Enquanto a implementação não existir, o README tem de o dizer explicitamente.
- Todo o item público (`pub`) leva doc comment. `#![warn(missing_docs)]` no crate.
