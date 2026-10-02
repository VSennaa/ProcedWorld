# SDD 16 — Visibilidade e névoa de guerra

> **Status: proposta.** Depende dos pilares GDD 02, 07 e 09, ainda não aprovados
> formalmente. ADR-0004, ADR-0006 e ADR-0007 são aceitos.

## 1. Responsabilidades e fronteiras

Este subsistema mantém, por civilização, o conhecimento observável do mapa e das
entidades. Calcula visão determinística ao fechar o turno, preserva a última
observação de cada tile, autoriza fatos geográficos para <code>GroundingRef</code>
e produz a projeção mínima que cada cliente pode receber.

O GDD 02 decidiu três estados por <code>CivilizationId</code> e <code>TileId</code>:
<code>unknown</code>, <code>remembered</code> e <code>visible</code>. O visível contém
observação atual; o lembrado mostra a última informação marcada como antiga; o
desconhecido não revela atributo, recurso, dono ou entidade. A geometria é
hexagonal axial/cúbica, pointy-top e cilíndrica, conforme ADR-0004 e GDD 02.

Responsabilidades propostas:

- calcular a cobertura de visão de cidades e unidades a partir do estado canônico;
- transicionar conhecimento e gravar memória/revisão de tile de forma reproduzível;
- decidir entidades e atributos observáveis por cada civilização;
- conceder ou negar <code>GroundingRef</code> conforme conhecimento do ator;
- aplicar o efeito mecânico de uma troca de mapas aceita pela diplomacia;
- projetar <code>ClientViewSnapshot</code>, <code>StateDiff</code> e chunks só com
  dados conhecidos.

Não é responsabilidade deste subsistema:

- mover unidades, fundar cidades, alterar terreno, calcular rendimentos ou gerar mapa;
- decidir exploração, diplomacia, Mandato ou aceitar proposta;
- definir termos, valor ou aceitação de troca de mapas; isso é diplomacia;
- chamar IA, interpretar texto de jogador, construir prompts ou guardar memória de IA;
- autenticar usuário, autorizar sessão, persistir fisicamente log ou renderizar UI;
- enviar seed, estado autoritativo completo ou previsão oculta ao cliente.

O motor e o log de comandos continuam autoritativos. Visibilidade é estado do
motor, não inferência do cliente; cache móvel é descartável e nunca amplia acesso.

## 2. Contratos e fluxo

Os tipos são contratos lógicos propostos. Identificadores, enumerações e
serialização devem ter ordem canônica e versão explícita.

~~~text
VisibilityState = unknown | remembered | visible

TileObservation {
  tile_id: TileId,
  revision: TileRevision,
  observed_turn: TurnNumber,
  terrain: KnownTerrain,
  features: [FeatureId],
  known_owner: CivilizationId?,
  known_resource: ResourceObservation?,
  entity_ids: [EntityId]
}

TileKnowledge {
  state: VisibilityState,
  last_observation: TileObservation?,
  shared_from: CivilizationId?,
  shared_turn: TurnNumber?
}

VisibilitySource {
  source_id: EntityId,
  civilization_id: CivilizationId,
  origin: TileId,
  base_radius: u8,
  source_kind: city | unit
}

GroundingRef { kind: GroundingKind, id: StableId, revision: u64 }
GroundingKind = tile | region | entity | domain_event | ledger_entry
~~~

<code>KnownTerrain</code> e <code>ResourceObservation</code> são subconjuntos
versionados da informação observável. Recurso oculto não aparece até revelação por
tecnologia, exploração ou evento. Atributos omitidos não podem ser inferidos por
sentinela, ordem, tamanho de payload ou identificador previsível.

~~~text
compute_visibility(state, civilization_id, ruleset_ref) -> VisibilityCoverage
apply_visibility(state, civilization_id, coverage) -> VisibilityDelta
can_reference(state, actor_id, ref: GroundingRef) -> Result<Unit, GroundingError>
project_tile(state, viewer_civilization_id, tile_id) -> KnownTileView?
project_chunk(state, viewer_civilization_id, chunk_id, known_revision?)
  -> MapChunk | MapChunkDelta
~~~

<code>compute_visibility</code> é puro. A cobertura une fontes válidas após ordenar
<code>source_id</code> e tiles por <code>TileId</code>. A proposta do GDD 02 é
raio-base 2, +1 em terreno alto e -1 em floresta/selva, sem linha de visão
complexa: propagação consome custo de terreno e elevação conforme catálogo
versionado. Valores e semântica de bloqueio/custo continuam abertos.

~~~text
ActionIntent {
  request_id, actor_id, turn, kind, grounding: [GroundingRef], payload
}

ShareMapIntent extends ActionIntent {
  kind: "share_map",
  recipient_civilization_id,
  scope: MapShareScope
}

MapShareScope { tile_ids: [TileId] | region_ids: [RegionId] }

AcceptedCommand {
  command_id, world_id, turn, accepted_sequence, actor_id,
  origin: player | governor | bot | entropy | fallback | system,
  kind, payload_canonical, ruleset_ref, grounding: [GroundingRef],
  intent_evidence_ref?
}
~~~

Após validação diplomática, <code>ShareMapIntent</code> torna-se
<code>AcceptedCommand</code>. Ele copia ao destinatário observações lembradas
permitidas pelo escopo; não torna tile visível, não transfere fonte de visão e não
fornece dado mais novo que a observação do cedente. O motor emite
<code>DomainEvent</code>, que diplomacia pode projetar como ajuda no
<code>LedgerEntry</code>. Proposta, aceite e texto pertencem à diplomacia.

### Fluxo proposto por turno

1. A orquestração aceita entradas externas antes de <code>step</code>, incluindo
   <code>ShareMap</code>.
2. <code>step</code> aplica <code>AcceptedCommand</code> em
   <code>accepted_sequence</code> canônica.
3. Após alterações de posição, cidade, terreno e compartilhamento, calcula visão
   em ordem crescente de <code>CivilizationId</code>.
4. Tile coberto passa a <code>visible</code> e recebe observação/revisão atual; tile
   que deixa a cobertura passa de <code>visible</code> a <code>remembered</code>.
5. Entidade é projetada somente em tile visível. Dado móvel não atualiza memória de
   tile, salvo se schema futuro definir observação estática.
6. O motor gera deltas de visão; transporte somente os serializa por sessão.

A ordem é proposta e deve coincidir com as fases globais do núcleo. Ela evita que
movimento permita visão da origem e do destino quando a regra não o permite.

## 3. Modelo de dados e invariantes

O estado canônico proposto usa tabela esparsa por civilização indexada por
<code>TileId</code>: <code>unknown</code> é implícito. Índices de chunks, cache de
visibilidade e compactação são derivados e não entram no hash, salvo especificação
explícita.

~~~text
VisibilityStateStore {
  by_civilization: OrderedMap<CivilizationId, OrderedMap<TileId, TileKnowledge>>,
  entity_visibility: OrderedMap<CivilizationId, OrderedSet<EntityId>>
}
~~~

Invariantes verificáveis:

- cada chave de civilização, tile, região e entidade referencia objeto existente;
- <code>unknown</code> não possui observação; <code>remembered</code> e
  <code>visible</code> possuem uma;
- observação tem o mesmo <code>tile_id</code> de sua chave, e revisão não excede a
  revisão autoritativa do tile no turno observado;
- <code>observed_turn <= current_turn</code>; observação não é reescrita com turno menor;
- <code>visible</code> representa tile após comandos precedentes do mesmo turno;
- união de fontes independe da ordem de armazenamento ou iteração;
- <code>entity_visibility[c]</code> só contém entidade existente em tile visível para c;
- entidade oculta não aparece em chunk, diff, <code>GroundingRef</code> ou erro;
- referência a tile/região/entidade só é aceita se a civilização do ator conhece o
  objeto e a revisão citada não é futura;
- Governador, bot e jogador da mesma civilização recebem igual autorização; IA não
  ganha visão perfeita nem cita fronteira, recurso ou entidade desconhecidos;
- compartilhamento não cria observação mais nova que a origem, não altera mapa e
  não transfere dado fora de <code>MapShareScope</code> validado;
- mesmo estado, regras, seed e <code>AcceptedCommand</code> produz bytes/hash iguais.

A revisão é do conhecimento observado, não só do tile. Revisita substitui a
observação corrente segundo schema canônico; cliente exibe idade com
<code>observed_turn</code>. A política visual de obsolescência não revela mudança
invisível.

## 4. Falhas, fallback e determinismo

Não há IA, rede, relógio, banco ou aleatoriedade implícita dentro de
<code>step</code>. <code>step(state, accepted_commands, seed, versions)</code> lê
somente estado, catálogos e comandos gravados, conforme ADR-0006. Visão usa
aritmética inteira/ponto fixo, limite de raio e ordem canônica. Se houver
desempate, deriva PRNG versionado da seed e IDs estáveis.

| Situação | Comportamento proposto |
|---|---|
| Fonte inválida, removida ou sem tile | ignorar cobertura e emitir <code>DomainEvent</code> auditável; nunca falhar parcialmente turno |
| Catálogo/regra ausente ou incompatível | recusar antes de abrir mundo/turno; não inventar raio fallback |
| Overflow, raio acima do teto ou grafo inválido | rejeitar comando causal, preservar hash anterior e registrar motivo enumerado |
| <code>ShareMap</code> inválido/sem autoridade | rejeitar sem revelar quais tiles o destinatário não conhece |
| IA cita fato desconhecido | validar e recusar intenção; T0 escolhe ação legal só com contexto autorizado |
| Timeout/erro de IA ao explorar | T0 escolhe tile desconhecido mais próximo, em ordem canônica, respeitando Mandato e perigo conhecido |
| Cache de cliente divergente | enviar remoção/delta ou <code>ClientViewSnapshot</code> paginado; cache não influi no motor |

Entram no log: <code>AcceptedCommand</code>, inclusive <code>ShareMap</code>, origem,
<code>accepted_sequence</code>, <code>GroundingRef</code> aceitos,
<code>RulesetRef</code> e hash/snapshot por turno. Entram como evidência separada:
<code>IntentEvidence</code>, erro externo e decisão de fallback. Replay mecânico
aplica apenas <code>AcceptedCommand</code>; nunca consulta IA nem revalida evidência.

Nunca entram no <code>step</code>: resposta bruta de IA, prompt, texto não confiável
de jogador, token/custo/chave, relógio, latência, sessão, cache, chunk ou ordem
incidental de hash map. Texto continua dado isolado e nunca autoriza
<code>GroundingRef</code> ou efeito sem validação tipada.

## 5. Orçamento proposto

Metas ainda não foram aprovadas. São hipóteses de benchmark, a versionar após medir
mundos, dispositivos e servidor de referência.

| Medida | Proposta inicial | Fallback/limite |
|---|---|---|
| Visão por turno | O(fontes + tiles alcançados), medido separado de <code>step</code> | teto catalogado de raio/fontes; rejeitar configuração inválida |
| Memória canônica | O(tiles conhecidos + entidades visíveis), desconhecido implícito | compactar observações; não descartar fato referenciável sem política aprovada |
| Projeção | só tiles conhecidos e até limite do protocolo | paginar/reduzir chunk; nunca completar com ocultos |
| Chunk de mapa | 48 KiB descompactado até benchmark no Android; valor final pelo benchmark (decidido em 2026-10-01) | delta ou páginas ordenadas |
| IA para visão | zero | T0 universal; T1/T2 só propõem intenção fora de <code>step</code> |
| Tokens/custo de IA | orçamento de IA por ator/dia | contexto só autorizado; timeout/teto usa T0 |

Rust implementa contratos puros, PostgreSQL persiste
snapshots/log e Godot 4 renderizar <code>KnownTileView</code>; isto não altera
semântica de autorização ou replay.

## 6. Estratégia de testes

- Testes unitários de hex e custo: raio, bônus/penalidade, polos, costura, fontes
  sobrepostas e ordem canônica.
- Propriedades: permutar fontes/comandos independentes não muda hash;
  <code>visible</code> exige cobertura; <code>visible -> remembered</code> preserva
  observação; desconhecido não serializa atributo; observação nunca vem do futuro.
- Autorização: jogador, Governador e bot da mesma civilização obtêm igual resultado;
  outra civilização não obtém nem cita tile, entidade ou revisão desconhecidos, nem
  por erro, tamanho ou delta.
- Fixtures gravadas: altitude/floresta/selva, cidade/unidade, primeiro contato,
  entidade que sai da visão e troca parcial de mapas. Fixar seed, <code>RulesetRef</code>,
  comandos e hash por turno.
- Golden replay: aplicar apenas <code>AcceptedCommand</code> e comparar estado,
  conhecimento e projeção autorizada. CI não chama IA/API real.
- Harness longo com bots T0: medir tempo, memória, chunks e ausência de referência
  geográfica não autorizada; imprimir seed e caso mínimo ao falhar.
- Contrato de protocolo: <code>MapChunk</code>, delta e recuperação omitem campos
  ocultos e invalidam cache ao perder autorização.

## 7. Perguntas abertas para o usuário

1. Qual regra de custo/bloqueio aplica elevação, floresta e selva?
   **Recomendação:** iniciar com raio 2 e modificadores do GDD, sem linha de visão
   complexa, e fechar tabela de custos versionada antes da implementação.

2. Quais atributos de tile e entidade persistem em memória lembrada?
   **Recomendação:** guardar somente observáveis estáticos e dono na última visão;
   ocultar unidades móveis e mostrar <code>observed_turn</code>.

3. Troca de mapas transmite todos os tiles lembrados, região escolhida ou ambos?
   **Recomendação:** iniciar com escopo explícito de regiões/tiles e cópia da última
   observação, sem visão atual, para auditoria e consentimento claros.

4. Troca de mapas exige contato, pacto específico ou pode integrar qualquer acordo?
   **Recomendação:** exigir relação ao menos em <code>contato</code> e termo
   diplomático validado; estados elegíveis ficam no catálogo de diplomacia.

5. Como a revisão evolui ao revisitar tile?
   **Recomendação:** usar <code>TileRevision</code> monotônica por tile e
   <code>observed_turn</code> na cópia; histórico fica no log/replay, não na visão.

6. Qual meta de memória e tempo vale para mundo/dispositivo máximos?
   **Recomendação:** aprovar limite de mundo e benchmark antes de fixar raio, formato
   compacto e chunk; até lá, os números deste SDD são hipótese.
