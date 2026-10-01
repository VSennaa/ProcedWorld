# CLAUDE.md — ProcedWorld

> Constituição do projeto para agentes (Claude Code, `/goal`, sessões na VPS).
> Leia este arquivo inteiro antes de qualquer trabalho. Depois leia `docs/STATUS.md`.

---

## 1. O que é o ProcedWorld

Jogo de estratégia 4X **por turnos, em tiles**, no espírito de *Civilization*, com **ciclo infinito**:
não existe condição de vitória obrigatória. O objetivo é **desenvolver uma sociedade ao longo de eras**
enquanto o mundo — outras civilizações e eventos — continua vivo.

- **Civilizações** são controladas por jogadores ou bots.
- O **Governador** é um bot do jogo, rodando no servidor, que o jogador configura por um **Mandato**
  (prioridades, limites, linhas vermelhas) para decidir por ele quando está ausente ou quando delega.
- A **Entropia** é a IA diretora de eventos procedurais: catástrofes climáticas, descobertas
  tecnológicas, incidentes diplomáticos, revoltas, pragas, cismas etc.
- A **diplomacia entre bots precisa fazer sentido**: toda ação diplomática deve ser justificável
  pelo estado do mundo e pelo histórico entre as civilizações.
- Alvo: **celular**. O usuário idealmente só informa a **API key** (BYOK) e joga; a civilização
  começa do zero.
- Modelos de linguagem **baratos** (ex.: DeepSeek) para narrativa e julgamento aberto;
  **modelos de decisão** (Jev → futuramente Laya) para decisões tipadas e frequentes.

**Não é:** um jogo de "tap e espere" estilo Clash of Clans, nem um gerador de texto com UI em volta.
É um simulador sistêmico onde a IA *propõe* e o motor *decide*.

---

## 2. Princípios inegociáveis (invariantes de arquitetura)

Qualquer código ou documento que viole um destes itens está errado, mesmo que "funcione".

1. **Motor determinístico.** O núcleo da simulação é uma função pura:
   `estado[n+1] = step(estado[n], comandos[n], seed)`.
   - PRNG com seed explícita e versionada (ex.: PCG/xoshiro); nunca `Math.random`, relógio ou ordem
     de iteração de hash map.
   - Matemática inteira/ponto fixo no núcleo. Sem I/O, sem rede, sem LLM dentro do `step`.
   - Mesmo seed + mesmos comandos ⇒ mesmo hash de estado, em qualquer máquina. Isso é testado em CI.
2. **A IA nunca altera o estado diretamente.** LLMs e modelos de decisão produzem **intenções**
   em schema tipado (JSON Schema). O motor de regras **valida** e só então converte em comandos.
   Saída inválida, timeout ou erro ⇒ **fallback determinístico** (heurística/utility AI). O jogo
   sempre avança, mesmo sem nenhum provedor de IA disponível.
3. **Event sourcing.** Toda entrada externa (ação de jogador, resposta de LLM, resultado de modelo de
   decisão, evento da Entropia) é gravada no log de comandos. Replays reproduzem a partida **sem**
   chamar IA de novo. Snapshots periódicos + hash por turno para detectar divergência.
4. **Maleabilidade por dados, não por código.** Eventos, tecnologias, unidades, governos e regras de
   diplomacia vivem em definições de dados versionadas (catálogos). A Entropia escolhe e parametriza
   *templates* dentro de limites; ela pode criar variações narrativas, mas os efeitos mecânicos
   sempre passam por templates validados.
5. **Grounding obrigatório.** Toda saída de IA que gera ação deve referenciar fatos do estado
   (ids de entidades, entradas do ledger de relações, eventos da crônica). Ação sem justificativa
   rastreável é descartada.
6. **Provedores são plugáveis.** Toda IA entra por portas (`LLMPort`, `DecisionPort`, `MemoryPort`).
   Trocar DeepSeek por outro LLM, ou **Jev por Laya**, é mudança de configuração, não de código.
7. **Custo é requisito.** Cada chamada de IA tem orçamento de tokens; cada jogador tem teto de custo
   diário (é a chave dele). Prompts são montados com **prefixo estável** para aproveitar cache de
   contexto do provedor.
8. **Segredos nunca no repositório nem em logs.** O repositório é **público**: nada de IP, hostname,
   chave ou `.env` real (use `<VPS_HOST>`). API keys de jogadores ficam criptografadas no servidor
   (ADR-0002). Use `.env.example`.

---

## 3. Arquitetura de IA (visão alvo — detalhar no SDD)

### 3.1 Camadas de decisão (do mais barato ao mais caro)

| Camada | O que é | Uso | Disponibilidade |
|---|---|---|---|
| **T0 — Heurística** | Utility AI / regras determinísticas | Fallback universal; micro-gestão; bots sem chave | Sempre |
| **T1 — Modelo de decisão** | Jev (API Runware `/v1/systemone`, `typesafe:jev@latest`) → Laya (`runware:laya@1`, open-weight Apache 2.0, pode rodar local) | Decisões tipadas e frequentes: *escolha* entre opções, *score* em escala, *sim/não* com probabilidade | Com chave/serviço |
| **T2 — LLM barato** | DeepSeek (ou equivalente) | Planejamento por era, negociação diplomática em linguagem natural, narrativa da Entropia, crônica | Com chave do jogador |

Regras: use a camada mais barata que resolve. Probabilidades do T1 só controlam ações relevantes
depois de **calibradas** em dados de teste. Toda chamada T1/T2 tem fallback T0.

### 3.2 Papéis de IA

- **Governador** (um por civilização): age dentro do **Mandato** do jogador. O Mandato é um
  documento estruturado (prioridades ponderadas, ações proibidas, limites de gasto/guerra,
  postura diplomática, quando "acordar" o jogador com notificação). O Governador nunca excede o
  Mandato; violações são bloqueadas pelo motor, não pelo prompt.
- **Entropia** (uma por mundo): diretor de eventos ao estilo "storyteller". Opera com um
  **orçamento de tensão** e curvas por era; escolhe templates do catálogo, parametriza dentro de
  limites e escreve a narrativa. Deve ser imprevisível para o jogador e reprodutível para o motor.
- **Diplomacia**: estado diplomático é uma máquina de estados explícita (paz, tensão, aliança,
  guerra, vassalagem...) mais um **ledger de relações** (dívidas, ofensas, tratados, promessas
  cumpridas/quebradas, com decaimento). A aceitação de propostas é calculada pelo motor (T0/T1);
  o LLM gera a proposta e o texto, não o resultado.

### 3.3 Memória e gestão de contexto (inspirada no *ai-memory* de Fabio Akita)

Referência: https://github.com/akitaonrails/ai-memory — Markdown como fonte da verdade,
índice SQLite/FTS5 derivado, camadas `working → episodic → semantic → procedural`, tipos
(`fact`, `decision`, `gotcha`, `rule`), supersessão (`is_latest=false` em vez de apagar),
decaimento por uso e handoff tipado (`open_questions`, `next_steps`).

Adaptação in-game (por civilização e para a Entropia):

- **Documentos canônicos de contexto** (derivados do estado, regeneráveis a qualquer momento):
  `Mandato`, `Resumo do Estado`, `Ledger de Relações`, `Crônica` (história compactada por era),
  `Doutrina` (regras procedurais aprendidas pelo Governador).
- **Memória em camadas**: observações do turno (`working`) → resumos de período (`episodic`) →
  conhecimento consolidado (`semantic`) → regras de conduta (`procedural`). Consolidação ao fim de
  cada período/era.
- **Previsão de saturação**: medir tokens por documento a cada turno, projetar a tendência e:
  - limiar suave (ex.: 60% do orçamento projetado em K turnos) ⇒ compactar `working` em `episodic`;
  - limiar duro (ex.: 85%) ⇒ **reset**: descartar o contexto conversacional e reconstruir só a
    partir dos documentos canônicos.
- O contexto da IA **nunca** é fonte da verdade; o estado do motor é. Perder toda a memória da IA
  deve degradar a qualidade das decisões, nunca corromper o jogo.

---

## 4. Processo: documentação antes de código

Ordem obrigatória: **GDD → SDD → desenvolvimento**. Não escreva código de produção para um sistema
cujo GDD e SDD não estejam aprovados pelo usuário. Exceção: *spikes* descartáveis em branches
`spike/*` para responder perguntas técnicas — nunca mergeados, o aprendizado vira ADR.

### 4.1 Estrutura de documentação

```
docs/
  STATUS.md          # fase atual, o que foi feito, próximos passos, bloqueios (handoff entre sessões)
  ROADMAP.md         # fases, marcos e critérios de saída
  GLOSSARY.md        # vocabulário do jogo (Mandato, Entropia, Crônica, Era...)
  gdd/               # Game Design Document, um arquivo por pilar/sistema
  sdd/               # Software Design Document, um arquivo por subsistema
  adr/               # Architecture Decision Records: NNNN-titulo.md (contexto, decisão, consequências)
```

- Docs em **português (PT-BR)**. Código, identificadores, commits e comentários em **inglês**.
- Toda decisão técnica relevante gera um ADR. Decisões não se revogam em silêncio: um novo ADR
  substitui o anterior (`Superseded by`).

### 4.2 Conteúdo esperado

- **GDD**: pilares, loop principal (turno, era, ciclo infinito), mapa e tiles, recursos e
  rendimentos, cidades, tecnologia, governo/sociedade, diplomacia, Entropia e catálogo de eventos,
  Governador e Mandato, experiência mobile (sessões curtas, notificações), progressão sem vitória,
  ascensão/colapso/renascimento, onboarding BYOK, balanceamento e métricas de diversão.
- **SDD**: topologia (cliente/servidor), modelo de estado e comandos, motor determinístico,
  persistência e snapshots, camadas de IA e portas, memória/contexto, orçamento de custo,
  segurança de chaves, API cliente-servidor, cliente mobile, infra/deploy, observabilidade,
  estratégia de testes.

---

## 5. Decisões e stack

Índice completo em [docs/adr/README.md](docs/adr/README.md). Resumo do que já está **aceito**:

- **Servidor autoritativo** (ADR-0001): a simulação roda no servidor; o celular é cliente. O servidor
  faz parte deste repositório e é auto-hospedável (VPS ou PC em dev, cliente separado). Não existe
  modo offline jogável no celular.
- **Governador é um bot do jogo, no servidor** (ADR-0002): o jogador o configura pelo Mandato.
  Consequência proposta: a chave do jogador fica criptografada no servidor; sem chave ou com teto
  atingido, degrada para T0/T1.
- **Turnos simultâneos** como no multiplayer do *Civilization* (ADR-0003): todos jogam o mesmo turno;
  ações aplicadas sequencialmente na ordem aceita pelo servidor (gravada no log); o turno fecha quando
  todos estão prontos ou o tempo acaba.
- **Grid hexagonal** (ADR-0004), coordenadas axiais/cúbicas.
- **Jev → Laya no servidor** (ADR-0005), atrás de `DecisionPort`. O host de referência não tem GPU:
  Laya local nele provavelmente é inviável; decidir com spike medido.
- **Motor determinístico + event sourcing** (ADR-0006).

**Stack — Proposto, não aceito** (ADR-0007): núcleo e servidor em **Rust**, cliente **Godot 4**
(MCP `godot-ai` disponível), **PostgreSQL**, memória em Markdown + SQLite FTS5, Docker Compose,
GitHub Actions. Não escrever código de produção antes de o usuário aceitar o ADR-0007.

Perguntas abertas vivem em `docs/STATUS.md` e na seção "Perguntas abertas" de cada arquivo do GDD.

---

## 6. Fases do projeto e critérios de saída

| Fase | Tag | Entrega | Critério de saída |
|---|---|---|---|
| **0 — Concepção** | — | GDD completo | GDD aprovado pelo usuário |
| **1 — Especificação** | — | SDD + ADRs iniciais + ROADMAP | SDD aprovado; decisões em aberto resolvidas |
| **2 — Indev** | `v0.1.0-indev.N` | Motor headless determinístico, mapa procedural, cidades, recursos, tecnologia básica, bots T0, harness de simulação, replay | 1.000 turnos com 8 bots sem crash; replay bit-a-bit idêntico; hash estável entre máquinas |
| **3 — Alfa** | `v0.x.0-alpha.N` | Portas de IA, Governador + Mandato, Entropia, diplomacia com ledger, memória/compactação, servidor na VPS, cliente mínimo em tiles | Partida longa com IA real sem intervenção; saturação de contexto nunca estoura; custo por turno medido e dentro do orçamento |
| **4 — Beta / MVP** | `v0.x.0-beta.N` | Cliente mobile jogável, onboarding BYOK, persistência, notificações, telemetria, limites de custo | Testadores externos jogam por dias; sem perda de save; diplomacia avaliada como coerente |
| **5 — 1.0** | `v1.0.0` | Polimento, balanceamento, documentação de usuário | Critérios definidos no ROADMAP aprovados |

Mudança de fase **só com aprovação explícita do usuário**.

---

## 7. Git e repositório

Remoto: `origin` → https://github.com/VSennaa/ProcedWorld (o remoto é a fonte da verdade; o trabalho
alterna entre PC e VPS).

### Branches

- `main` — apenas versões estáveis e tags. Nunca commitar direto. Merge só via PR aprovado pelo usuário.
- `develop` — integração contínua. Recebe PRs das branches de trabalho.
- Branches de trabalho, sempre a partir de `develop`, no formato `<tipo>/<área>/<descrição-curta>`:
  - tipos: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `spike`
  - áreas: `engine`, `ai`, `back`, `front`, `infra`, `gdd`, `sdd`
  - ex.: `docs/gdd/diplomacia`, `feat/engine/map-generation`, `feat/ai/entropia-director`,
    `chore/infra/ci-determinism`.
- `release/<versão>` — estabilização antes de tag; `hotfix/<descrição>` — a partir de `main`.

### Regras

- **Push frequente** da branch de trabalho para `origin` (no mínimo ao fim de cada sessão), para que
  outra máquina possa continuar.
- Commits no padrão **Conventional Commits** com escopo da área: `feat(engine): add hex grid`.
- PRs pequenos e focados, com descrição do porquê, como foi testado e link para GDD/SDD/ADR.
- Nunca reescrever histórico de `main`/`develop`. Nunca `--force` em branch compartilhada.
- Versões seguem SemVer com sufixo de fase (ver tabela da seção 6).
- Identidade de commit: `VSenna <69941024+VSennaa@users.noreply.github.com>` (e-mail noreply do GitHub).

### Infra / VPS

Leia [infra/README.md](infra/README.md) antes de mexer no host. Resumo: entrar como `deploy`
(nunca root); a fonte da verdade de portas é `/opt/infra/PORTS.md` **no host**; faixa do ProcedWorld
**8100–8199**, sempre com bind em `127.0.0.1`; banco sem porta publicada; stack em
`/opt/stacks/procedworld/`; clone de trabalho em `~deploy/ProcedWorld`. O host tem 2 vCPU e 2 GB de
RAM: evite builds pesados em paralelo com outros serviços.

---

## 8. Qualidade — regras contra "AI slop"

- **Profundidade antes de largura.** Poucos sistemas que interagem bem valem mais que muitos rasos.
  Toda mecânica precisa interagir com pelo menos duas outras (ex.: clima → produção → estabilidade).
- **Nada de feature de fachada.** Proibido: dados fictícios apresentados como reais, botões que não
  fazem nada, `TODO` no lugar de lógica, stubs mergeados como se fossem implementação. Se algo é
  parcial, diga isso no PR e em `docs/STATUS.md`.
- **Pronto significa**: implementado ponta a ponta, testado, documentado no SDD se mudou contrato,
  e verificado rodando — não apenas compilando.
- **Testes obrigatórios**:
  - determinismo (mesmo seed + comandos ⇒ mesmo hash), em CI;
  - replays "golden" de partidas de referência;
  - testes de propriedade nas regras do motor (invariantes: recursos nunca negativos, etc.);
  - IA testada com **fixtures gravadas** (record/replay). CI **nunca** chama LLM/API real;
  - harness de simulação longa para regressões de balanceamento e desempenho.
- **Coerência diplomática verificável**: o harness gera relatórios onde cada ação diplomática de bot
  mostra as entradas do ledger que a justificam. Ações sem justificativa são bugs.
- **Orçamentos explícitos**: tempo de processamento por turno, tokens por chamada, custo por turno.
  Regressões de orçamento falham o CI quando mensuráveis.
- Prefira bibliotecas maduras a reinventar; justifique dependências novas no PR.

---

## 9. Como trabalhar neste repositório (protocolo do agente)

**Início de sessão**
1. Ler `CLAUDE.md` e `docs/STATUS.md` (se não existir, criá-lo na primeira sessão).
2. `git fetch` e conferir branch atual; continuar a branch indicada em `STATUS.md`.
3. Confirmar em que fase o projeto está e respeitar o que ela permite.
4. Rodar a auditoria de cotas (skill `quota-audit`) e decidir quem trabalha.

**Durante**
- Trabalhar em passos pequenos e verificáveis; commitar a cada passo coerente.
- Ao encontrar uma decisão de design ou arquitetura não coberta por GDD/SDD/ADR: **parar e perguntar**
  ao usuário, ou registrar como pergunta aberta em `STATUS.md` e seguir com outra tarefa desbloqueada.
- Não inventar fatos sobre ferramentas externas (APIs, modelos, preços): verificar na documentação.

**Fim de sessão (handoff)**
- Atualizar `docs/STATUS.md` com: o que foi feito, estado dos testes, próximos passos, perguntas
  abertas e bloqueios.
- Commitar e dar push da branch.

**Sempre exigem aprovação do usuário**
- Aprovar GDD, SDD, ADRs de stack/topologia, mudança de fase.
- Merge em `main`, criação de tags/releases.
- Mudanças na VPS (deploy, portas, serviços, custos), criação de recursos pagos.
- Qualquer uso de chave de API real.

**Agentes, subagentes e cotas** — ver [docs/process/agentes-e-cotas.md](docs/process/agentes-e-cotas.md).
- Claude é o supervisor; tarefas independentes vão para subagentes Codex (conta ChatGPT) cujo modelo é
  escolhido pelo **Jev Router** (skill `jev-subagents`, script `tools/agents/jev-codex.sh`). O OpenRouter
  é usado **só** para a decisão do Jev, nunca para executar tarefas.
- **Auto-auditoria obrigatória** (skill `quota-audit`): checar a cota do Claude (`get_usage`), do Codex
  e do OpenRouter (`tools/agents/quota-check.sh`) no início da sessão, antes de cada lote de
  subagentes e no fim das tarefas grandes. Claude ≥ 75% ⇒ delegar; tudo esgotado ⇒ commit, push,
  handoff e retomada agendada. O projeto nunca pode ficar parado sem próximo passo agendado.

**Sugestão para o fluxo de desenvolvimento**: instalar o próprio *ai-memory* na VPS para dar memória
persistente e handoff entre sessões de agentes de código — complementar a `docs/STATUS.md`, não
substituto.

---

## 10. Glossário rápido

- **Turno**: unidade atômica da simulação. **Era**: agrupamento de turnos com consolidação de memória
  e mudança de curva da Entropia.
- **Mandato**: configuração do jogador que restringe o Governador.
- **Governador**: IA que administra uma civilização (sempre para bots; para jogadores, quando offline
  ou por delegação).
- **Entropia**: IA diretora de eventos procedurais do mundo.
- **Ledger de Relações**: registro auditável das interações entre duas civilizações.
- **Crônica**: história compactada do mundo/civilização, usada como contexto de IA e como conteúdo
  para o jogador.
- **Intenção**: proposta tipada de ação vinda de uma IA, antes da validação do motor.
- **Comando**: ação validada que entra no log e é aplicada pelo `step`.
