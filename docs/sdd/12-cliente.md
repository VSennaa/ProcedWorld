# SDD 12 — Cliente

> **Status:** proposta para revisão do SDD. GDD aprovado em 2026-10-01 e ADR-0007 aceito;
> Godot 4 é a adaptação cliente decidida.

## Objetivo e fronteiras

O Cliente é a camada de interação: recebe a visão autorizada de uma civilização, apresenta causas e
consequências e envia intenções do jogador ao servidor. Android é o alvo principal; web/desktop é
cliente interno para teste e depuração. O padrão é retrato, com mapa opcional em paisagem desde que
as ações permaneçam acessíveis.

É responsabilidade do Cliente:

- renderizar estado conhecido, mapa, relatório e escolhas legais recebidas;
- manter cópia local descartável para abertura rápida e reconexão;
- montar, revisar e enviar comandos; exibir confirmação, rejeição e resultado;
- tornar Pauta, Mapa, Sociedade, Relações e Crônica acessíveis;
- explicar decisão por fatos, regra/template, entidades e próxima ação.

Não é responsabilidade do Cliente:

- executar `step`, gerar mapa, resolver conflito, escolher aleatoriedade ou calcular efeitos;
- ser fonte de verdade, aceitar comando, ordenar turnos simultâneos ou avançar turno;
- chamar IA, guardar API key, montar prompt ou decidir Governador/Entropia;
- revelar informação não autorizada pelo servidor.

Portanto não há modo offline jogável (ADR-0001). Offline permite somente consultar a última visão,
preparar rascunhos e reconectar; nunca simular ou confirmar efeito localmente.

## Visão de arquitetura proposta

```text
TransportSession -> ClientStore <- Presentation / Input
                         |
                         +-> LocalCache (descartável e versionado)
```

- `TransportSession` negocia versão, autenticação e reconexão; entrega mensagens por sequência do servidor.
- `ClientStore` mantém projeção do jogador, rascunhos e entrega; não contém regra do jogo.
- `Presentation / Input` transforma projeção em telas e gestos em `CommandDraft`.
- `LocalCache` é otimização apagável; o servidor pode reenviar snapshot e diferenças.

Godot 4 provê cenas/controles, sessão HTTP/WebSocket, recursos
serializáveis e renderer hexagonal. Isso não torna `TileMap` a única implementação possível.

## Experiência e navegação propostas

A barra principal contém **Pauta**, **Mapa**, **Sociedade**, **Relações** e **Crônica**. O Mandato
abre pela Pauta ou perfil do Governador. Ao retornar, a rota é sempre Pauta; links no relatório levam
ao detalhe sem exigir memória do painel anterior.

| Superfície | Pergunta | Ação que pode emitir |
|---|---|---|
| Pauta | O que exige atenção? | escolher, delegar, revisar, marcar pronto |
| Mapa | Onde ocorre e o que se conecta? | ordem espacial sobre alvo legal |
| Sociedade | O que sustenta a civilização? | resposta de crise ou foco permitido |
| Relações | Em quem confiar e o que é devido? | aceitar, recusar ou contrapor tratado |
| Crônica | O que ocorreu e qual foi o custo de IA? | consultar fatos, narrativa e consumo |

Cartões mostram causa, prazo em turnos, alternativas legais, custo/consequência e “Por quê?”. Esse
link abre a cadeia factual; a narrativa da Crônica é rotulada e nunca substitui explicação mecânica.
Indicadores usam nomes e escalas canônicos: `C`, `L`, `S`, `D`, `G`, `W`, `E` e `P`
de `docs/gdd/12-variaveis-e-formulas.md`. `P` mostra fatores, não barra opaca; `W` e `E`
permanecem provisórios até validação no harness.

### Unidades, Pronto e árvore de pesquisa (decidido em 2026-10-03)

- **Unidades no mapa**: toda unidade visível aparece no tile com ícone por papel (`assets/entities`)
  e cor da civilização; a seleção mostra tipo, movimento restante, vida e ordem atual, com as ações
  legais **Mover** (toque no destino: `MoveTo`), **Explorar**, **Fortificar**, **Prontidão** (`Sentry`)
  e **Pular**.
- **Fila de atenção**: a Pauta lista as unidades ociosas (`idle_units`); tocar centraliza o mapa
  nela. O botão **Pronto** fica desabilitado com o contador "N unidades aguardam ordem"; o servidor
  é quem garante a regra (`units_awaiting_orders`).
- **Pronto compacto e contador de decisões** (pedido do usuário em 2026-10-03): fora da Pauta, um botão
  flutuante compacto no canto inferior do Mapa (e das demais telas do jogo) mostra só ícone e um selo
  com o número de **decisões restantes** = unidades ociosas + eventos aguardando resposta + capital por
  fundar. Com decisões restantes, tocar leva à próxima (centraliza a unidade, abre o evento ou o cartão
  da capital), como o "próxima unidade" do *Civilization*; com zero, vira **Pronto** (ícone de
  confirmação) e envia `ready`. O botão grande da Pauta continua igual.
- **Painel da cidade** (pedido do usuário em 2026-10-03): fila de produção gerenciável — remover item,
  subir/descer — com produção por turno e turnos estimados por item; seletor de foco da cidade.
- **Alfa mini (2026-10-03)**: escolher pesquisa na árvore (`SetResearch`); tela Sociedade só leitura
  com `C`, `L`, `S`, `D`, `G`, `W`, `E`, `P` e fatores; tela de eventos com nome, texto e escolhas
  legíveis vindos do `catalog`; ações **Atacar** (unidade estrangeira visível e alcançável) e
  **Construir melhoria** (trabalhador, opções legais do catálogo).
- **Árvore de pesquisa**: tela a partir do `catalog`, com pré-requisitos em colunas,
  tecnologias dominadas, pesquisa atual e progresso; a escolha de pesquisa entra no alfa mini.
- **Conexão**: tela inicial com URL do servidor, criar mundo (seed, civilizações) ou entrar
  (mundo, civilização); a fixture continua como modo demonstração e nos testes headless.

## Mapa e interação espacial

O mapa renderiza hexágonos pointy-top em mundo cilíndrico: cruzar borda horizontal faz wrap visual;
polos são limites fechados. A identidade lógica é sempre axial `(q, r)` enviada pelo servidor.
Cópias desenhadas além da borda não criam tile nem alvo novo.

```text
world_x = s * sqrt(3) * (q + r / 2)
world_y = s * 3/2 * r
logical_q = modulo(q, world_width)       // somente o eixo com wrap
logical_r = r                            // rejeitar fora dos polos
```

O adaptador converte tela/toque para axial, normaliza apenas `q` e confirma que o tile está na
projeção autorizada. A regra não depende dessa geometria: o Cliente envia `TileId`; o servidor
revalida alcance, visibilidade, recursos e turno.

- toque seleciona hexágono e abre cartão; arrastar move câmera; pinçar muda zoom; toque duplo
  centraliza/alterna zoom; pressionar e segurar descreve ícone;
- ao escolher ação, só alvos legais recebidos são destacados; a ordem entra em prévia e só é enviada
  ao marcar pronto;
- no zoom regional, cidades, crises e propostas agrupam; no local, seleção/alvos têm borda ampla.
  Área de 9 mm e ampliação de alvos adjacentes são propostas de usabilidade;
- só uma camada fica ativa (produção, abastecimento, controle, pressão ou diplomacia), sem ocultar
  legenda, seleção ou prazo crítico.

Visibilidade é `unknown`, `remembered` ou `visible`. O estado `remembered` exibe última
observação marcada como antiga; cache não pode promovê-lo a `visible` nem preservar informação
retirada da projeção posterior.

## Contratos de cliente-servidor

Os tipos abaixo pertencem ao protocolo versionado; são independentes de stack. Campos mecânicos usam
inteiros e ids opacos; texto é conteúdo de exibição não confiável.

```text
type ProtocolVersion = { major: u16, minor: u16 }
type Turn = u32
type Revision = u64
type TileId = { q: i32, r: i32 }
type EntityId = string
type CommandId = string

type WorldProjection = {
  world_id: EntityId, civilization_id: EntityId, turn: Turn,
  revision: Revision, protocol: ProtocolVersion,
  map: MapChunk[], agenda: AgendaItem[], society: SocietyView,
  relations: RelationView[], chronicle: ChronicleEntry[], cost: CostView
}
type MapChunk = { chunk_id: string, revision: Revision, tiles: KnownTile[] }
type KnownTile = {
  id: TileId, visibility: "unknown" | "remembered" | "visible",
  observed_turn?: Turn, terrain?: CatalogId, overlays?: Overlay[], entities?: EntityRef[]
}
type CommandDraft = {
  client_command_id: CommandId, expected_turn: Turn, expected_revision: Revision,
  kind: CommandKind, payload: object
}
type CommandResult = {
  client_command_id: CommandId, status: "accepted" | "rejected" | "duplicate",
  reason_code?: string, projection_revision?: Revision
}
```

```text
connect(Hello { protocol, last_revision? }) -> Welcome | UpgradeRequired
subscribe(WorldId, revision?) -> ProjectionSnapshot | ProjectionDelta[]
submit(CommandDraft) -> CommandResult
resume(last_revision) -> ProjectionDelta[] | ProjectionSnapshot
```

`expected_turn` e `expected_revision` expõem ordem atrasada. `accepted` apenas confirma
aceitação pelo servidor; efeito final chega em projeção posterior. `duplicate` com o mesmo id é
idempotente. `rejected` traz código estável e, se necessário, projeção atualizada. Eventos de
transporte como `AgendaChanged`, `TurnResolved`, `CostChanged` e `SessionInvalidated` são
dicas para buscar/aplicar projeção versionada, não uma segunda fonte de estado.

## Modelo local e invariantes

```text
type ClientState = {
  projection?: WorldProjection,
  pending: Map<CommandId, CommandDraft>,
  cache_revision?: Revision,
  connection: "offline" | "connecting" | "online" | "resyncing",
  ui: { route, selected_tile?: TileId, map_layer, orientation }
}
```

Invariantes verificáveis:

1. Delta só aplica se sua `base_revision` for a atual; lacuna, regressão ou hash incompatível exige
   `resync` por snapshot.
2. `revision`, `turn` e `TileId` exibidos pertencem à projeção autorizada mais nova; cache nunca vence servidor.
3. Rascunho tem `client_command_id` único até resolução e conserva id/conteúdo em reenvio.
4. Cliente não produz ação/alvo fora de catálogo ou lista legal recebida; servidor é validador final.
5. Civilização, conta e versão de protocolo nunca compartilham chave de cache.
6. Segredo, token persistente ou texto de credencial nunca entra em cache, telemetria ou erro.
7. `C`, `P` e custos trazem turno/revisão; ausência é indisponível, não zero.

## Cache, reconexão e falhas

O cache proposto guarda snapshot compactado, chunks conhecidos, preferências e rascunhos pendentes,
sempre com `world_id`, `civilization_id`, protocolo e revisão. Retenção, limite e criptografia
local são perguntas abertas. Ele deve ser apagável sem perda de jogo.

| Situação | Comportamento do Cliente |
|---|---|
| sem rede | estado marcado “última sincronização”; só rascunho local |
| queda durante envio | conserva rascunho e reenvia idempotentemente após `resume` |
| delta ausente/corrompido | para diferenças, pede snapshot e substitui projeção |
| comando rejeitado | mostra motivo, remove rascunho e destaca escolhas atuais |
| protocolo incompatível | bloqueia ações e pede atualização; não interpreta payload |
| IA indisponível/custo atingido | mostra resultado das regras quando material; não bloqueia turno |
| pressão de memória | descarta chunks fora da área recente e baixa-os de novo |

Não há relógio de resolução no Cliente: prazos são em turnos; servidor decide prontidão, ausência e
ordem. Indicador local de tempo é UX e não altera comando, prioridade ou resultado.

## Determinismo, registro e observabilidade

Pelo ADR-0006, Cliente não participa de `step`. Não entra em `step`: frame rate, orientação,
zoom, gesto, ordem de pacote, cache, relógio local, animação, acessibilidade, texto narrativo ou
resposta bruta de IA.

No log autoritativo entram somente comandos aceitos, resultados externos normalizados em comando e
metadados de replay definidos pelo servidor. Registro local, com retenção limitada e sem segredo:
versão app/protocolo, id técnico permitido, revisão, `client_command_id`, latência em faixas,
falha de transporte, código de rejeição e motivo de ressincronização. Não entram API key, token de
sessão, payload sensível, prompt, resposta bruta de IA ou conteúdo não autorizado.

Hashes/revisões recebidos detectam divergência de projeção e transporte; o Cliente nunca declara hash
autoritativo do mundo.

## Orçamentos propostos

Metas iniciais, não decisões aprovadas. Devem ser medidas em Android de referência e cliente interno
antes de virar gate de CI.

| Recurso | Meta inicial | Degradação aceitável |
|---|---:|---|
| abrir Pauta de cache válido | até 1 s | estado antigo identificado |
| aplicar delta sem animação | até 100 ms | lote e animação posterior |
| receber e apresentar um turno já resolvido | até 1 s após projeção completa | relatório imediato; animação posterior |
| câmera/seleção | alvo de 16,7 ms por quadro | reduzir detalhe, agrupamento e animação |
| memória de mapa | limite por aparelho | LRU de chunks e novo download |
| envio de comando | uma tentativa ativa por id | fila idempotente e reconexão |
| tokens/custo de IA no Cliente | 0 | só exibir consumo do servidor |

Cliente não chama IA nem consome tokens de inferência. Crônica/custo só exibe consumo agregado e
cache/fallback autorizados pelo servidor. Nenhum valor monetário local é fonte de verdade.
O teto numérico de RAM permanece pendente até a escolha do aparelho de referência; antes disso, o
orçamento verificável é o limite configurável, a evacuação LRU e a ausência de crescimento ilimitado.

## Acessibilidade e adaptação de tela

Decisão crítica é operável por toque, foco sequencial e leitor de tela. Rótulos incluem ação, efeito
e custo; cor nunca é único canal para facção, risco, pressão ou visibilidade. Ícone, textura e texto
a acompanham. Fonte, alto contraste e redução de movimento são preferências locais e não entram em
`step`.

Em retrato, Pauta/cartão inferior têm prioridade. Em paisagem opcional, mapa ganha área, mas ações
permanecem acessíveis sem esconder confirmação/prazo. Se área útil não sustentar alvo seguro,
preferir lista/cartão a reduzir alvo abaixo de 9 mm.

## Estratégia de testes proposta

- **Contratos:** serialização, compatibilidade, mensagens desconhecidas e `CommandResult` idempotente.
- **Propriedades:** deltas ordenados equivalem a snapshot; repetição não muda estado; revisão inválida
  pede ressincronização; normalização cilíndrica preserva `q` válido e rejeita polo inválido.
- **Geometria:** fixtures pointy-top, seleção nas bordas, polos, zoom e toque de fronteira; cópia de
  wrap seleciona o mesmo `TileId`.
- **Fixtures gravadas:** snapshots/deltas, sucesso, rejeição, reconexão e fallback; CI não chama IA/rede real.
- **Harness:** reproduzir sessão em Android e web/desktop, comparando projeções, comandos e árvore
  semântica acessível, nunca pixels de animação.
- **Acessibilidade/desempenho:** foco/rótulos automatizados e inspeção com leitor de tela, fonte,
  alto contraste, redução de movimento e aparelhos de pouca memória.

## Perguntas abertas para o usuário

1. Android pode manter cache criptografado de projeções ou só cache apagável sem garantia em aparelho
   comprometido? **Recomendação:** cache apagável e proteção nativa quando disponível, apenas da projeção já vista.
2. Qual aparelho Android (versão mínima, RAM, resolução) vira referência de orçamento? **Recomendação:**
   escolher intermediário e perfil de memória restrita para harness.
3. Web/desktop interno precisa paridade visual ou só Pauta, Mapa, inspeção e depuração?
   **Recomendação:** escopo mínimo de fixture, reconexão e inspeção, sem postergar Android.
4. Como SDD 10/11 definirá autenticação e retenção de sessão? **Recomendação:** token curto em
   armazenamento seguro, nunca no cache de projeções ou log.
5. Qual janela diária, fuso e moeda aparecem no custo? **Recomendação:** servidor normaliza valores
   com moeda explícita; Cliente apenas exibe.
6. Paisagem será rotação do sistema, botão no mapa ou ambos? **Recomendação:** sistema e botão apenas
   no Mapa; demais telas em retrato quando isso melhorar legibilidade.

## Referências

- ADR-0001 — servidor autoritativo.
- ADR-0004 — grid hexagonal.
- ADR-0006 — motor determinístico e event sourcing.
- ADR-0007 — stack tecnológica (aceito).
- GDD 02 — mapa e tiles; GDD 11 — experiência mobile.
- `docs/gdd/12-variaveis-e-formulas.md` — variáveis compartilhadas.
