# SDD 08 — Memória e contexto

> **Status: proposta para revisão do SDD.** GDD aprovado em 2026-10-01; ADR-0007 aceito.
> Este documento define contratos e invariantes de memória.

## Objetivo

O subsistema prepara contexto limitado, rastreável e recuperável para o
Governador e a Entropia. Ele organiza informação derivada do estado do motor em
memórias de durações distintas, antecipa saturação e reconstrói o contexto quando
necessário. É inspirado nas camadas `working → episodic → semantic → procedural`
do *ai-memory*, mas não adota sua implementação como decisão.

O resultado esperado é melhor continuidade das propostas de IA, sem que uma
perda, atraso ou alteração dessa memória possa alterar a verdade mecânica do
mundo. A camada atende uma civilização por vez e uma instância mundial da
Entropia; não cria uma memória compartilhada implícita entre civilizações.

## Responsabilidades e fronteiras

### É responsabilidade deste subsistema

- Derivar, versionar e disponibilizar os documentos canônicos de contexto.
- Registrar observações e consolidá-las entre as quatro camadas de memória.
- Selecionar trechos por escopo, relevância, orçamento e ordem estável.
- Medir uso e tendência de tokens; aplicar compactação e reset de contexto.
- Montar um prefixo estável para chamadas de IA e um sufixo variável por turno.
- Produzir evidência de quais documentos, versões e fatos fundamentaram uma
  proposta de IA.
- Preservar artefatos necessários à auditoria, replay e testes, sem guardar
  segredos ou texto de jogador como instrução.

### Não é responsabilidade deste subsistema

- Ser fonte de verdade do estado, aplicar regras, escolher efeitos mecânicos ou
  executar `step`.
- Autorizar ações: o motor continua bloqueando violações do Mandato e validando
  intenções.
- Calcular aceitação diplomática, orçamento de tensão ou elegibilidade de
  templates; apenas representa fatos já apurados por seus donos.
- Gerar texto narrativo, decidir entre opções ou chamar um provedor diretamente.
  Esses papéis pertencem às portas de IA e seus orquestradores.
- Interpretar texto livre do jogador como regra. Esse texto é dado não confiável
  e só pode ser apresentado isoladamente como conteúdo.
- Substituir o log de comandos, snapshots ou hashes previstos no ADR-0006.

## Fontes, escopos e documentos canônicos

Os documentos abaixo são derivados do estado e do log já validados. Devem poder
ser regenerados de um snapshot mais comandos aceitos, com a versão apropriada de
catálogo e renderizador. Não são a fonte de verdade nem exigem preservação para a
correção mecânica do jogo.

| Documento | Escopo | Conteúdo mínimo proposto | Dono da verdade |
|---|---|---|---|
| `Mandate` | civilização | prioridades, linhas vermelhas, reservas, postura, avisos e versão | estado/comandos do Mandato |
| `StateSummary` | civilização | fatos atuais relevantes, riscos, recursos, compromissos e ids citáveis | estado do motor |
| `RelationshipLedger` | par de civilizações | dívidas, ofensas, tratados, promessas e decaimento já calculado | estado diplomático |
| `Chronicle` | civilização e mundo | história compactada por era, eventos e consequências confirmadas | log + estado |
| `Doctrine` | civilização/bot | regras procedurais aprendidas, com evidência e validade | memória consolidada, revisável |

`Mandate` é uma representação do Mandato aceito, não uma reinterpretação pela IA.
Ao assumir um bot, estado, Ledger e Crônica são herdados; seu Mandato torna-se
preset editável e sua Doutrina vira apenas sugestão, como decidido no GDD 10.

## Modelo conceitual de dados

Os tipos são contratos propostos; nomes e serialização podem mudar com a stack.

```text
MemoryScope = Civilization(civilization_id) | World
MemoryLayer = Working | Episodic | Semantic | Procedural
MemoryKind  = Fact | Decision | Gotcha | Rule

CanonicalDocument {
  document_id, scope, kind, schema_version, source_turn,
  content, content_hash, source_refs[]
}

MemoryEntry {
  entry_id, scope, layer, kind, created_turn, valid_from_turn,
  valid_to_turn?, summary, source_refs[], supersedes?, is_latest,
  importance, last_used_turn, provenance
}

ContextBudget {
  max_input_tokens, reserved_output_tokens, soft_ratio, hard_ratio,
  projection_horizon_turns
}

ContextPackage {
  package_id, scope, turn, canonical_versions[], memory_entries[],
  stable_prefix, variable_suffix, estimated_input_tokens, evidence_refs[]
}
```

`source_refs` aponta somente para ids e versões de fatos, entradas do Ledger,
eventos da Crônica, documentos canônicos ou comandos registrados. `provenance`
identifica derivação determinística, consolidação ou fixture gravada; nunca
inclui credenciais. Uma entrada de IA não verificada não pode ganhar o tipo
`Fact`, `Decision` ou `Rule` apenas por ter sido gerada.

### Camadas

- **Working:** observações específicas do turno ou janela curta, ordenadas por
  turno e id; é a camada mais descartável.
- **Episodic:** resumos de um período ou era, incluindo contexto, decisão e
  consequência observável.
- **Semantic:** conhecimento consolidado sobre relações, padrões e fatos
  duradouros, sempre ancorado em fontes.
- **Procedural:** regras de conduta para o Governador (`Rule` e `Gotcha`), que
  orientam proposta, mas não sobrepõem Mandato, catálogo ou validador.

A consolidação é proposta ao fim de cada período ou era. Ela produz nova entrada
e marca a antecessora como `is_latest=false`; não apaga a evidência enquanto a
política de retenção e auditoria não permitir. Uma regra deixa de valer ao
expirar sua evidência, ser supersedida ou conflitar com documento canônico atual.

## Contratos e fluxo

```text
MemoryPort.build_context(request: ContextRequest) -> ContextPackage | MemoryFailure
MemoryPort.record_observations(batch: ObservationBatch) -> RecordResult
MemoryPort.consolidate(scope, boundary: Period | Era) -> ConsolidationResult
MemoryPort.project_saturation(scope, budget) -> SaturationForecast
MemoryPort.compact(scope, reason) -> CompactionResult
MemoryPort.reset(scope, reason) -> ResetResult
MemoryPort.rebuild_canonical(scope, at_turn) -> CanonicalSet

ContextRequest {
  scope, actor: Governor | Entropy, turn, task_kind,
  budget, required_fact_refs[], prompt_schema_version
}
```

`build_context` recebe uma visão já autorizada do estado. Ele não consulta relógio,
rede nem aleatoriedade para escolher conteúdo: para os mesmos documentos, pedido e
versões, retorna a mesma ordem e o mesmo `content_hash`. A ordenação proposta é
por prioridade de documento, relevância inteira, turno mais recente e `entry_id`.

```text
on_turn_accepted(turn, scope):
  canonical = rebuild_canonical(scope, turn)
  record_observations(derive_observations(canonical, accepted_commands))
  forecast = project_saturation(scope, budget)
  if forecast.crosses_hard: reset(scope, SaturationHard)
  else if forecast.crosses_soft: compact(scope, SaturationSoft)
  return build_context(request)
```

O orquestrador entrega `ContextPackage` à porta `LLMPort` ou `DecisionPort` e
associa seu `package_id` à resposta externa gravada. A IA devolve uma intenção
tipada com `evidence_refs`; o validador exige que cada referência pertença ao
pacote ou ao estado permitido. O motor, e não este subsistema, valida a intenção
e a converte em comando.

## Invariantes verificáveis

1. Um documento canônico deve ser regenerável no mesmo `content_hash` a partir
   das mesmas fontes, versão de esquema e turno.
2. Todo `source_ref` resolve para uma fonte existente e autorizada ao escopo.
3. Uma memória de civilização não é selecionada para outra civilização, exceto
   documentos mundiais explicitamente compartilháveis à Entropia.
4. Há no máximo uma entrada `is_latest=true` por cadeia de supersessão.
5. `Doctrine` nunca contradiz `Mandate` ou um fato atual: na colisão, ela é
   excluída do pacote e registrada como obsoleta para revisão.
6. Contexto reduzido, vazio ou corrompido só pode reduzir qualidade da proposta;
   não pode impedir a progressão do turno nem mudar regras do motor.
7. O pacote declara token estimado menor ou igual ao orçamento de entrada antes
   de ser enviado; itens excedentes são removidos por ordem determinística.
8. Nenhum segredo, chave BYOK, cabeçalho de autenticação ou texto de sistema de
   provedor é persistido em `MemoryEntry`, `ContextPackage` ou log de auditoria.
9. Texto de jogador preserva origem `untrusted` e só pode ocupar bloco de dados
   delimitado; nunca altera instruções estáveis nem documentos canônicos.

## Saturação, compactação e reset

Propõe-se medir tokens por documento e camada a cada turno. A projeção usa uma
janela versionada de observações recentes e informa a estimativa para `K` turnos,
sem decidir mecânica do jogo. Os valores iniciais abaixo requerem calibração:

```text
projected = current_tokens + trend_tokens_per_turn * K
soft = projected >= max_input_tokens * 0.60
hard = projected >= max_input_tokens * 0.85
```

No limiar suave, `working` elegível é resumida em `episodic`; em seguida, itens
episódicos maduros podem formar `semantic` ou `procedural` se tiverem fontes
suficientes. O original é supersedido, não silenciosamente reescrito. No limiar
duro, o subsistema descarta o contexto conversacional e todas as seleções
efêmeras, reconstrói somente os cinco documentos canônicos e adiciona apenas
memórias consolidadas que caibam no orçamento. Isso não apaga estado, comandos,
Ledger nem Crônica canônica.

O prefixo estável, para favorecer cache do provedor, contém versão do contrato,
papel, regras de segurança, schema da saída e documentos canônicos ordenados que
não mudaram. O sufixo variável contém turno, fatos recentes, consulta, opções e
bloco de texto não confiável quando houver. Uma mudança de documento, versão ou
ordem muda explicitamente o `prefix_hash`; não se tenta reutilizar cache de forma
opaca.

## Falhas, fallback e determinismo

| Falha | Tratamento proposto | Efeito no jogo |
|---|---|---|
| índice/armazenamento indisponível | reconstruir documentos canônicos das fontes disponíveis; se falhar, pacote mínimo | T0 decide; turno avança |
| compactação inválida ou sem fonte | rejeitar artefato, manter a versão anterior e registrar diagnóstico | sem ação mecânica |
| orçamento excedido | truncar por política estável; aplicar compactação ou reset | contexto menor |
| provedor timeout, resposta inválida ou injeção | resposta não entra como memória confiável; fallback T0 | comando determinístico |
| hash canônico divergente | marcar inconsistência, reconstruir e impedir uso do artefato divergente | sem alterar `step` |

Conforme o ADR-0006, entram no log de comandos as entradas externas que afetam
uma proposta: id da requisição, `package_id`, hashes/versões de documentos,
resposta bruta sanitizada ou referência imutável a ela, intenção validada ou
motivo de rejeição, fallback escolhido e comando resultante. Também entram os
resultados de `compact` e `reset` quando forem necessários para explicar o pacote
gravado, com suas fontes e versões.

Nunca entram em `step`: busca de memória, estimativa de tokens, tendência de
saturação, renderização Markdown, índice, cache do provedor, chamada de IA,
timeout, data/hora, custo monetário ou decisão de compactar/resetar. `step` recebe
somente os comandos já aceitos, seed explícita e dados versionados; replay não
chama IA nem precisa reproduzir um índice.

## Orçamento operacional

Os números são limites iniciais propostos e devem ser medidos em spike/harness
antes de adoção. O subsistema entrega telemetria de `estimated_input_tokens`,
tokens efetivos quando retornados pelo provedor, latência, custo atribuído e
motivo de degradação, sem segredos.

| Recurso | Proposta de limite | Degradação |
|---|---|---|
| preparação por ator/turno | teto configurável; não bloqueia a resolução | pacote mínimo ou T0 |
| memória residente por escopo | teto configurável por camada e mundo | consolidar, depois resetar seleção |
| entrada de IA | `max_input_tokens - reserved_output_tokens` | remover itens em ordem estável |
| saída e custo diário | orçamento da chamada e teto diário do jogador | T1/T0; nunca repetir chamada automaticamente |
| compactação | no máximo uma tentativa por escopo/turno | adiar para reset no limiar duro |

Não há prazo, tamanho de memória, tokens ou preço aceitos no GDD/ADR para este
subsistema. Portanto, os valores de configuração e a janela `K` são propostas,
não metas de produto. PostgreSQL guarda metadados e
artefatos de auditoria, enquanto Markdown como fonte legível e SQLite/FTS5 como
índice derivado são implementações possíveis; a semântica acima permanece igual.

## Estratégia de testes

- **Determinismo:** reconstruir os mesmos documentos e pacote em máquinas e
  execuções distintas; comparar hashes, ordem e conjunto de fontes.
- **Propriedades:** gerar cadeias de supersessão, escopos e orçamentos; verificar
  unicidade de versão atual, isolamento de civilização, referências resolvíveis e
  tamanho dentro do teto.
- **Fixtures gravadas:** guardar estado/snapshot, comandos, documentos,
  respostas externas sanitizadas e intenção validada; o replay consome a fixture,
  nunca o provedor.
- **Harness de saturação:** simular partidas longas, crescimento adversarial de
  `working`, falha de índice e repetidos resets; conferir que todo turno termina
  e que o estado final depende apenas dos comandos.
- **Segurança:** fixtures com injeção em texto de jogador e saída malformada;
  confirmar isolamento, rejeição e ausência de segredos nos artefatos.
- **Contrato:** testar serialização e compatibilidade de `ContextPackage` contra
  versões de schema; fixtures antigas permanecem legíveis ou falham
  explicitamente por versão incompatível.

## Perguntas abertas para o usuário

1. Qual deve ser a retenção auditável de entradas supersedidas e de respostas de
   IA por mundo/jogador? **Recomendação:** reter referências e hashes enquanto o
   save existir; permitir expurgo do conteúdo bruto por política de privacidade.
2. A Doutrina pode ser alterada automaticamente após consolidação ou só entrar
   como sugestão para o jogador? **Recomendação:** atualização automática apenas
   como sugestão não vinculante, sempre citando fontes; o Mandato continua sendo
   a única autoridade configurável pelo jogador.
3. Quais limites iniciais de tokens, memória, prazo e custo por ator/turno são
   aceitáveis para o MVP? **Recomendação:** decidir após um spike com provedores
   reais e fixar valores por configuração versionada, não no código.
4. O conteúdo bruto de respostas de IA deve ser preservado para auditoria de
   replay ou apenas hash, intenção validada e fixture sob demanda? **Recomendação:**
   preservar intenção e hashes sempre; reter bruto sanitizado somente por uma
   janela configurável, pois replay mecânico não depende dele.
5. A Crônica mundial da Entropia deve ser visível integralmente a todas as
   civilizações ou filtrada por conhecimento? **Recomendação:** manter uma fonte
   mundial interna e gerar visões por civilização; assim não se vaza informação
   oculta nem se duplica a história mecânica.

## Referências

- [ADR-0005 — Modelo de decisão](../adr/0005-modelo-de-decisao-no-servidor.md)
- [ADR-0006 — Motor determinístico e event sourcing](../adr/0006-motor-deterministico-event-sourcing.md)
- [GDD 08 — Entropia e eventos](../gdd/08-entropia-e-eventos.md)
- [GDD 09 — Governador e Mandato](../gdd/09-governador-e-mandato.md)
- [GDD 10 — Ciclo infinito e eras](../gdd/10-ciclo-infinito-e-eras.md)
