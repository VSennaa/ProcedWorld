# SDD 07 — Diplomacia

> **Status:** Proposta para revisão após aprovação do GDD. ADR-0007 aceito; Rust, Godot e PostgreSQL são a stack normativa.

## Objetivo e fronteiras

Diplomacia converte estado do mundo e histórico bilateral em compromissos verificáveis entre civilizações. Ela mantém a máquina de estados, valida e pontua propostas, registra o Ledger de Relações, trata linguagem natural como entrada não confiável e processa expiração, eras e sucessão após colapso.

Não move unidades, não resolve combate, não transfere estoque, não calcula produção, não altera coesão e não executa Mandato. Ela cria obrigações estruturadas e recebe fatos dos subsistemas proprietários: Economia liquida comércio misto; Tecnologia decide elegibilidade de intercâmbio; Núcleo aplica `step`, ordem e log. IA e texto nunca produzem efeito mecânico direto.

Entradas canônicas: civilizações, fronteiras, rotas, capacidades/reservas consultáveis, Mandato, catálogo versionado, eventos de Economia/Combate/Colapso/Entropia e estado diplomático. `Cf`, `R`, `Dv` e `A` usam [12-variaveis-e-formulas.md](../gdd/12-variaveis-e-formulas.md): `Cf`/`R` 0–100, `Dv` −100..100 por direção e `A` 0–100 por proposta.

## Máquina de estados

Para cada `PairId` não ordenado, `relation_state` é exatamente `unknown`, `contact`, `peace`, `tension`, `pact`, `alliance`, `war` ou `truce`. `vassalage` é vínculo sobreposto, com suserano, vassalo, prazo e cláusulas; não substitui estados com terceiros. O Ledger é direcional: `Cf(A→B)` não implica `Cf(B→A)`.

| Origem | Gatilho validado | Destino |
|---|---|---|
| `unknown` | contato/exploração confirmada | `contact` |
| `contact` | primeira interação bilateral aceita | `peace` |
| `peace`, `pact`, `alliance` | quebra, incidente ou escalada catalogada | `tension` |
| `peace`, `tension` | tratado delimitado aceito | `pact` |
| `pact` | aliança aceita | `alliance` |
| estado não `unknown` | declaração de guerra válida | `war` |
| `war` | cessar-fogo aceito | `truce` |
| `truce` | prazo vence sem violação | `peace` |

Esta tabela é proposta de catálogo inicial. Cada aresta precisa declarar pré-condições, comando e efeitos. Romper pacto/aliança encerra o vínculo, registra a quebra e só então recalcula o estado. Prosa, resposta de modelo ou ordem incidental de iteração nunca causam transição. Transição inválida não muda estado nem Ledger e devolve `RejectionCode` enumerado.

## Contratos e interfaces

Os esquemas são independentes de linguagem e serão implementados em Rust/JSON Schema com persistência PostgreSQL, sem mudar semântica.

```text
Proposal {
  id: ProposalId, revision: u8, issuer: CivId, recipient: CivId,
  kind: ProposalKind, terms: [CatalogTerm], duration_turns: u16?,
  cited_facts: [FactId], created_turn: Turn, expires_turn: Turn,
  acceptance_threshold: u8, rule_catalog_version: Version
}
ProposalKind = trade | passage | aid | pact | alliance | reparation |
               truce | territory_transfer | vassalage | tech_exchange
WarDeclaration { issuer, recipient, objective_id, cited_claim_or_aggression: FactId }
WarObjective = protect_route | contain_incursion | recover_territory | enforce_reparation
```

`CatalogTerm` só referencia template existente. Comércio permite pacotes livres de bens e/ou riqueza dos dois lados; o valor de referência em riqueza serve à comparação, nunca cria bens. A fórmula de proporcionalidade na entrega parcial permanece pendência da Economia.

```text
DiplomaticIntent {
  actor: CivId, action: propose | respond | declare_war | ratify | renegotiate | repudiate,
  proposal_or_war: Proposal | WarDeclaration,
  grounding: [StateFactRef], source: player | t0 | decision_model | llm
}
validate(intent, state, catalog) -> Result<ValidatedCommand, RejectionCode>
evaluate(proposal, recipient_view, state, catalog) -> Evaluation
apply(command, state, seed) -> [DiplomaticFact]
close_era(pair, state, catalog) -> [DiplomaticFact]
```

`validate` confere autoridade, partes, estado, template, limites, recursos/reservas, rota/fronteira, prazo, Mandato e grounding. Intenção de bot sem fatos válidos é recusada. `apply` executa regra já validada; não chama rede, relógio, LLM ou modelo de decisão.

### Aceitação e contraproposta

Proposta herdada do GDD:

```text
A = limitar(0, 100, 40 + 0,30(Cf - 50) - 0,25R + 0,20Dv + U - K)
```

`Cf`, `R` e `Dv` são direcionais, do destinatário ao emissor, normalizados quando necessário. `U` é utilidade concreta (proposta: −20..20); `K`, custo/risco (proposta: 0..30). Pesos, faixas e limiares devem estar em catálogo versionado e ser calibrados. `A >= 60` aceita, `A < 40` recusa e a faixa intermediária tenta contraproposta são propostas, bem como limiares especiais para guerra/trégua/vassalagem.

Contraproposta só deriva de template válido, melhora custo/risco para o destinatário e respeita as restrições de ambos. Cadeia usa `proposal_id` e `revision`; até duas contrapropostas, expiração no fim do turno seguinte e bloqueio de dois turnos para pedido idêntico recusado são parâmetros propostos. Expiração não é ofensa; mudança material de fato citado pode liberar nova proposta.

### Negociação em dois modos

Cartões rápidos produzem `DiplomaticIntent` diretamente. Para texto livre, o adaptador:

1. tenta reconhecer ação de cartão por gramática e IDs explícitos;
2. havendo divergência/ambiguidade, envia somente dados delimitados a `LLMPort` para extrair intenção no schema permitido;
3. valida a saída como a de cartão; timeout, erro ou schema inválido retorna `TextNegotiationUnavailable`, sem efeito, e sugere cartões.

O prompt fixo define mensagens do jogador, nomes, Crônica e conteúdo recebido como dados, nunca instruções. O adaptador não concede ferramentas, não interpola dado em mensagem de sistema, limita tamanho/caracteres de controle e exige IDs/termos do snapshot. Aceita JSON estrito sem campos extras; URLs, instruções, tentativa de alterar regras e cláusulas fora de catálogo são inválidas. O LLM pode redigir explicação depois da validação, mas explicação não reentra no caminho de comando.

## Ledger e invariantes

```text
LedgerEntry {
  id: FactId, pair_id: PairId, direction: CivId -> CivId?, turn: Turn,
  category: promise | debt | offense | aid | trade | border | incident | treaty,
  status: active | fulfilled | expired | breached | superseded,
  intensity: u8, deltas: { cf: i8, resentment: i8, debt: i8 },
  cause_command: CommandId?, source_event: EventId?, due_turn: Turn?,
  evidence: [StateFactRef], catalog_version: Version
}
TraitorMark { id, offender, affected, cause_entry, created_era, severity, status }
```

Entrada é imutável; correção/mudança de status cria entrada que referencia a anterior. Saldos são projeções reconstituíveis e limitadas. Fato comum pode ter `direction = null`, mas delta afetivo ou de dívida é sempre direcional.

Invariantes verificáveis:

- Um `relation_state` por `PairId`, simétrico para ambas as partes.
- Proposta, entrada, vínculo e referência citam entidades existentes na criação.
- `Cf`/`R` ficam 0–100, `Dv` −100..100 e `A` 0–100; nenhum saldo é alterado à mão.
- Tratado ativo tem duas partes distintas, termo catalogado, início e prazo/condição de término.
- Obrigação só vira `fulfilled`/`breached` após fato do subsistema proprietário.
- Marca de traição referencia quebra/ofensa grave, não decai ordinariamente e só reduz por reconciliação catalogada após reparação aceita.
- Guerra exige objetivo de catálogo e reivindicação/agressão válida; objetivo define fim/faixa de custo, não vitória automática.
- Após colapso, defesa expira; apenas dívida, reivindicação e traição passam a sucessor identificável que herda território; ratificar, renegociar ou repudiar gera fato auditável.
- O log histórico nunca é apagado; agregados por era preservam referências aos fatos fonte.

No fechamento de era, proposta herdada do GDD: `Cf` move 1 rumo a 50, `R` cai 1 e `Dv` sem vencimento move 1 rumo a zero. Compromissos ativos e marcas de traição seguem no cálculo corrente; resumos antigos permanecem consultáveis na Crônica. Deltas, intensidades e reconciliação são dados versionados, não interpretação narrativa.

## Guerra, paz e colapso

`declare_war` exige objetivo catalogado, agressão/reivindicação citada, autoridade, Mandato e estado elegível. Guerra cria obrigações/fatos para Combate, Economia e Sociedade; cada dono aplica seus custos e devolve resultado. Trégua/paz pode incluir cessar-ataques, retirada, reparação limitada, troca/retorno territorial e duração. Não há rendição total implícita nem apagamento da identidade do derrotado.

Sucessão recebe a decisão canônica de herdeiro territorial do subsistema de colapso. Expira defesa, transmite somente relações permitidas ao herdeiro identificado e abre `ratify | renegotiate | repudiate`. Sem escolha válida não presume ratificação: T0 segue prioridade catalogada/Mandato e registra motivo.

## Determinismo, falhas e auditoria

Conforme ADR-0006, `step` recebe comandos validados, estado, seed explícita e catálogo versionado. O log registra intenção tipada, comando aceito/rejeitado, resultado T0/T1/T2 usado, versões de catálogo/schema, grounding, componentes de `A`, limiar, fallback, transição, Ledger e hash do turno. Texto bruto, se retido para replay, é entrada externa com acesso definido por Segurança; nunca é prova mecânica nem instrução.

Nunca entram em `step`: rede, chave de API, prompt, relógio, aleatoriedade não semeada, ordem de hash map, resposta futura de IA, custo monetário externo, texto não validado ou decisão humana pendente. Timeout/indisponibilidade de `DecisionPort`/`LLMPort`, parser falho, injeção detectada e referência obsoleta não travam turno: cartões recebem erro enumerado; bots usam T0 determinístico ou mantêm relação sem nova proposta. Saída inválida é descartada.

Toda ação de bot produz `DiplomaticAudit`: ator, intenção, decisão, IDs de Ledger/fatos, componentes de avaliação, versão de regras e motivo de fallback/rejeição. O harness falha se ação aceita não tiver grounding verificável.

## Orçamento (proposta mensurável)

Medir por turno: tempo de validação/avaliação/aplicação, tamanho de Ledger ativo/agregado e chamadas de IA. Orçamento inicial, sujeito a medição:

- Diplomacia usa até 10% do orçamento de CPU do turno.
- Memória corrente prioriza fronteira, guerra, tratado ativo ou evento recente; demais relações são agregadas por era.
- T1: no máximo uma decisão de aceitação por proposta elegível, schema pequeno e teto de tokens.
- T2: só conversa divergente e redação opcional, com prefixo estável, teto por chamada e teto diário por jogador; T0 é sempre disponível e sem custo externo.

Milissegundos, tokens e moeda dependem de provedor e de ADR-0007; devem ser configuração versionada. CI alerta/falha em regressão mensurável.

## Estratégia de testes

- **Unitários/tabela:** estados, pré-condições, `RejectionCode`, expiração de trégua, objetivo de guerra e sucessão.
- **Propriedades:** limites de `Cf`/`R`/`Dv`/`A`, imutabilidade/reconstrução do Ledger, simetria, obrigação sem fato do dono e decaimento monotônico.
- **Determinismo:** mesmo snapshot, comandos, seed e catálogo resultam no mesmo hash, fatos e projeções; ordem de inserção não altera resultado.
- **Golden replays:** comércio misto, duas contrapropostas, quebra/reparação, guerra e colapso com/sem herdeiro; replay não chama IA.
- **IA gravada:** fixtures T1/T2, saída inválida, timeout e injeções adversariais. CI não chama provedor; texto hostil não cria comando nem altera prompt de sistema.
- **Harness longo:** bots T0 e fixtures de IA; relatório de cada ação, distribuição de `A`, guerras, traições, Ledger, CPU, memória, tokens e custo. Coerência é cobertura de fatos citados, não fluência.

## Perguntas abertas para o usuário

1. **Quais transições e termos compõem o catálogo inicial?**
   Recomendação: começar com comércio, ajuda, passagem, pacto, trégua e quatro objetivos de guerra; deixar aliança, vassalagem, cessão territorial e intercâmbio tecnológico até terem regras completas.

2. **Os pesos/limiares propostos de `A` viram baseline?**
   Recomendação: aprová-los apenas como configuração de teste, medi-los no harness e não congelá-los no GDD antes de partidas longas.

3. **Qual a retenção e exibição de texto livre malicioso?**
   Recomendação: log canônico guarda hash e intenção validada; texto bruto fica em armazenamento protegido, limitado e com política definida no SDD 11.

4. **Quem ratifica tratados após colapso de humano ausente?**
   Recomendação: Governador somente dentro do Mandato; sem permissão explícita, expirar/repudiar deterministicamente e notificar jogador.

## ADRs relacionados

- ADR-0005 — `DecisionPort`, modelo no servidor e resposta gravada.
- ADR-0006 — motor determinístico, event sourcing e IA que só propõe.
- ADR-0007 — stack aceita.
