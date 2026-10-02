# SDD 01 — Núcleo de simulação

> **Status: proposta.** O GDD ainda não foi aprovado formalmente e o ADR-0007 (stack) está
> Proposto. Este desenho prioriza contratos e invariantes independentes de tecnologia. Valores
> numéricos e escolhas aqui marcados como proposta dependem de revisão.

## 1. Objetivo e fronteiras

O Núcleo de simulação calcula transições reproduzíveis do mundo. Dado um estado canônico, uma lista
ordenada de comandos já aceitos, a seed e versões imutáveis dos catálogos, produz o próximo estado,
eventos derivados e um hash verificável.

Responsabilidades:

- validar comandos contra estado, regras e Mandato, e aplicar os válidos em ordem registrada;
- executar as fases canônicas de resolução do turno;
- consumir aleatoriedade somente por PRNG explícito e versionado;
- manter regras mecânicas determinísticas para eventos, diplomacia, economia e sociedade;
- produzir hash, eventos de saída e pontos de snapshot para persistência e replay;
- ler catálogos versionados sem alterá-los durante uma partida.

Fora da fronteira:

- presença/conexão de jogador, autenticação, relógio, timers ou decisão de quando fechar turno;
- transporte de rede, UI, renderização hexagonal ou armazenamento físico;
- chamadas a LLM, provedores, APIs, banco de dados, filesystem ou relógio;
- geração de texto narrativo e escolha externa de intenção. O motor recebe resultados externos
  validados e registrados como comandos; aplica apenas sua representação mecânica;
- desenho e balanceamento final de fórmulas cujo GDD segue em proposta.

O servidor autoritativo monta o lote de entrada e chama o núcleo. Conforme ADR-0008, o turno fecha
quando todos os humanos presentes jogaram; ausentes recebem ação do Governador. A definição operacional
de “presente” pertence ao subsistema de sessão. O núcleo recebe uma entrada de turno fechada e não
consulta sessões. Bots e Governadores são ordenados pela seed do mundo com rotação, conforme GDD 01.

## 2. Contratos

### 2.1 Tipos conceituais

```text
WorldId, EntityId, PlayerId      // IDs estáveis, sem semântica de ordenação implícita
TurnNumber: UInt64
Fixed: Int64 + escala declarada   // ponto fixo; overflow é erro determinístico
CatalogRef: (catalog_id, version, content_hash)
PrngRef: (algorithm_id, algorithm_version, seed, stream_id)
```

```text
WorldState = {
  world_id, turn, ruleset_ref, catalogs: CatalogRef[], prng: PrngState,
  map: MapState, civilizations: Ordered<EntityId, CivilizationState>,
  diplomacy: DiplomacyState, world_events: EventState, chronicles: ChronicleFacts,
  schema_version
}
```

`Ordered` é uma coleção cuja ordem de serialização é definida pelo contrato (IDs em ordem binária
canônica), nunca pela iteração incidental de hash map. O conteúdo exato dos subestados é propriedade
dos SDDs de mapa, economia, cidades, tecnologia, sociedade, Entropia e diplomacia.

```text
AcceptedCommand = {
  command_id, world_id, turn, accepted_sequence: UInt64, actor_id, origin,
  kind, payload_canonical, rules_version, catalog_version, grounding_facts: GroundingRef[]
}
CommandOrigin = player | governor | bot | entropy | fallback | system
GroundingRef = { kind, id, revision }
```

`accepted_sequence` é atribuído pelo servidor ao aceitar o comando, é único globalmente no mundo e
forma a ordem total do log. IDs idempotentes evitam aplicar duas vezes a mesma submissão. `grounding_facts` é obrigatório
para propostas de IA que gerem ação e permite auditoria/grounding (CLAUDE.md §2).

Exemplos de payloads conceituais:

```text
IssueOrder { unit_id, order_template_id, target }
SetCityPriority { city_id, priority_id }
ProposeTreaty { counterpart_id, terms[] }
DeclareAttack { unit_id, target_id } // reserva unidade/custo; sem dano imediato
SetReady { player_id, ready: Bool }  // controle de turno; pode ser evento do log
ApplyEntropyEvent { template_id, parameters, evidence[] }
```

Os nomes são ilustrativos, não um catálogo final de comandos. Um comando rejeitado não modifica
estado; sua rejeição e motivo pertencem ao log operacional/auditável e não à sequência aplicada ao
`step`. Comandos aceitos permanecem em ordem, mesmo se Pronto for desfeito.

### 2.2 Funções puras

```text
validate(state, envelope, catalogs) -> Accepted(command) | Rejected(reason)
step(state, accepted_commands: AcceptedCommand[], seed, versions) -> StepResult

StepResult = {
  next_state, emitted_events: Event[], state_hash,
  rng_audit: RngAudit[], applied_command_ids: CommandId[]
}
```

`step` exige todos os comandos de um único turno em ordem estritamente crescente de `accepted_sequence`,
refs de catálogo exatas e `state.turn == input.turn`. Erros de formato/incompatibilidade retornam erro
sem estado parcial. A transição não faz I/O, não lê hora, não chama IA nem busca recursos externos.

```text
result = step(previous_state, accepted_commands, seed, versions)
assert hash(result.next_state) == result.state_hash
```

O orquestrador persiste entrada e resultado em transação própria. Se persistência falhar, não publica
o novo estado. A separação transacional é proposta; o núcleo permanece independente do mecanismo.

### 2.3 Intenções e IA

LLM/DecisionPort não entram no núcleo. Um adaptador valida schema, versão, limites e referências
factuais da intenção e a traduz para zero ou mais `AcceptedCommand`. Resposta bruta, fallback usado,
validação e comando resultante devem ser gravados para replay, observando política de privacidade.
Saída inválida, timeout ou erro de provedor aciona fallback determinístico T0; ausência de IA não
impede `step`.

## 3. Modelo e invariantes

### 3.1 Estado canônico

O estado contém apenas fatos necessários à simulação e ao replay. Valores derivados podem ser
armazenados por desempenho somente se houver regra de reconstrução e verificação contra sua fonte.
Texto de apresentação, sessões, cache de prompts e métricas de infraestrutura não compõem o estado
canônico nem seu hash.

Entidades referenciam IDs estáveis. Catálogos definem templates de unidade, tecnologia, evento,
política e regras. Estado guarda IDs e parâmetros validados, não cópias mutáveis do template. A árvore
base tecnológica é fixa em dados; descobertas emergentes vêm de templates validados da Entropia.
Práticas ativas limitam-se a no máximo 3 por civilização; tecnologias preservadas contam no limite
de 2 legados ativos após colapso. Valores e exceções são regidos pelos pilares correspondentes.

Mapa e coordenadas seguem GDD/SDD de mapa. Cada cidade mantém população agregada em três grupos por
função (cultivadores, ofícios, mercadores); demandas políticas são atributos desses grupos, sem
segundo modelo populacional. Não há teto rígido de cidades; carga administrativa crescente é regra
externa ao esquema. Migração interna é automática na simulação.

Cada civilização tem legitimidade e coesão inteiras em [0,100]; cada cidade tem estabilidade local
em [0,100]. Pressão de crise existe por cidade e civilização, sendo esta uma média ponderada por
população. Os termos de pressão vêm de economia, mapa, grupos e coesão; peso local exato de
estabilidade segue aberto no GDD. Governo tem três eixos discretos, com três posições cada. Revoltas
são eventos possíveis; magia é fenômeno originado por template e reação política nacional tem efeito
mecânico.

### 3.2 Invariantes verificáveis

- Identidade do mundo, turno e refs de catálogo não mudam silenciosamente dentro de uma transição.
- Recursos, população, reservas e custos não podem ficar negativos; saturação/limites usam regras
  explícitas e aritmética sem overflow silencioso.
- IDs são únicos no escopo definido; toda referência aponta para entidade/evento existente ou gera
  rejeição determinística.
- Sequências de comando são únicas e crescentes; nenhuma ordem aceita é reordenada por ator, rede ou
  tipo de comando.
- Rejeição não cobra custos nem produz efeitos mecânicos. Validação e rejeição são determinísticas.
- Limites de escala (incluindo [0,100]) são mantidos após cada fase, não apenas ao fim do turno.
- O dano de combate só ocorre na fase de combate: ataques declarados reservam unidade e custo;
  perdas do mesmo confronto são calculadas simultaneamente.
- Uma entrada externa só altera estado via comando registrado e validado. Texto narrativo nunca é
  fonte mecânica.
- Mesma versão de motor, estado inicial, comandos, seed, regras e catálogos implica estado final,
  eventos e hash idênticos em todas as plataformas suportadas.

## 4. `step`, ordem das fases e aleatoriedade

### 4.1 Ordem canônica proposta

GDD 01 determina aplicação sequencial das ações, combate em fase própria e resolução ao fim do turno
(produção, crescimento, Entropia, consolidação de memória). A granularidade seguinte é proposta para
que dependências sejam explícitas; cada fase percorre IDs em ordem canônica.

1. Conferir turno, sequência, refs, versões e pré-condições estruturais; falhar atomicamente se inválido.
2. Aplicar comandos aceitos em ordem de `accepted_sequence`: reservar custos, ordens, políticas, tratados e
   declarações de ataque. Uma ação posterior observa efeitos já aplicados, salvo combate adiado.
3. Resolver movimentos e ordens não combatentes, segundo regras de unidade/mapa.
4. Resolver cada confronto: agrupar ataques válidos, calcular resultados com estado pré-dano do
   confronto e aplicar perdas simultaneamente.
5. Rodar economia: rendimentos, manutenção e contratos conforme ordem definida pelo catálogo/regras.
6. Rodar crescimento populacional e migração automática.
7. Atualizar tecnologia, práticas e progressos de era conforme elegibilidade.
8. Atualizar sociedade, estabilidade e pressões de crise com estado após economia e população.
9. Selecionar/aplicar entradas de Entropia já registradas, validando template, elegibilidade,
   parâmetros e proteção contra repetição. Nenhuma geração de texto ocorre aqui.
10. Atualizar diplomacia/ledger e efeitos temporais contados em turnos, conforme ordem final a aprovar.
11. Emitir fatos de crônica e resumo de memória derivados; consolidação externa de contexto pode
    consumir esses fatos sem alterar o estado mecânico.
12. Incrementar turno, validar invariantes globais, canonicalizar, calcular hash e produzir snapshot
    quando o intervalo determinístico for atingido.

A posição da Entropia em relação à atualização diplomática, e a interação de eventos com produção,
ficam abertas. O passo 1 deve distinguir validação estrutural do replay de validação de cada comando;
comandos já aceitos são reproduzidos sem reconsultar serviços externos.

### 4.2 PRNG versionado

Proposta: PRNG determinístico identificado por `algorithm_id` e versão; seed imutável do mundo, criada
na inicialização e registrada. Cada subsistema/fase usa stream derivado estável (`world/turn/phase/
entity/purpose`) para que consumo adicional numa fase não altere sorteios alheios. Derivação e
serialização do estado interno fazem parte do contrato e dos golden tests. Não depender de RNG da
linguagem, sistema operacional, relógio ou ordenação de coleções.

Se ADR-0007 for aceito e Rust permanecer na stack, a implementação poderá escolher crate/algoritmo
concreto; algoritmo, versão e vetores de compatibilidade devem continuar explícitos no formato salvo.
Trocar algoritmo exige nova versão e compatibilidade de replay ou migração deliberada. Um comando de
Entropia gravado contém template e parâmetros finais: replay aplica essa escolha, sem pedir nova
decisão à IA. RNG continua útil para regras mecânicas determinísticas que sorteíam resultados.

### 4.3 Hash por turno

Proposta: serialização canônica versionada dos campos mecânicos, ordenação por ID e codificação
explícita de inteiros, enums, nulos e coleções; hash criptográfico com nome/versão registrados junto
ao hash. Excluir caches e apresentação. Hash inclui turno, estado mecânico, refs de catálogo e estado
do PRNG para detectar divergências futuras. Guardar hash após cada turno no log. Divergência em replay
identifica o primeiro turno e permite comparar eventos e consumo de RNG.

Não se usa hash da serialização incidental do runtime. Alterar campos canônicos ou codificação exige
incrementar `state_schema_version` e definir compatibilidade. Hash não substitui validação de snapshot.

## 5. Log, falhas e recuperação

O log append-only guarda, por turno: abertura/fechamento lógico, envelopes aceitos em ordem, origem,
IDs idempotentes, resultados de validação, respostas externas usadas para produzir comandos,
fallback selecionado, decisões de Entropia já parametrizadas, versões/schema, hashes e metadados de
replay. Payloads humanos e de IA precisam de política de retenção/redação; segredos nunca são logados.

Nunca entram como efeitos diretos no `step`: chamada ou resposta viva de LLM, decisão não gravada,
hora, timeout calculado durante execução, ordem de chegada não registrada, sessão/conexão, resultado
de rede, leitura de banco, cache, locale ou ponto flutuante dependente de plataforma. O timeout pode
determinar fallback antes do fechamento, mas o resultado/fallback efetivo precisa constar do log.

Falhas de validação rejeitam apenas o comando e retornam código estável, sem mutação/custo. Entrada
incompatível, catálogo ausente ou versão desconhecida abortam o turno sem estado parcial e exigem
reparo operacional. Falha de provedor usa T0 e grava decisão; falha de persistência não publica estado
novo. Reexecução idempotente usa IDs de comando e hash do turno para não duplicar efeitos.

Snapshots periódicos armazenam estado canônico, refs/versões, PRNG e hash. Recuperação carrega snapshot,
verifica hash e reaplica comandos posteriores. Proposta: snapshot completo a cada 20 turnos e também
em marcos administrativos; intervalo sujeito a medir e versionar. Snapshot é acelerador, não fonte
alternativa da verdade ao log. PostgreSQL só se aplica se ADR-0007 for aceito; contrato lógico é
independente do backend.

## 6. Catálogos e compatibilidade

Cada partida fixa refs imutáveis para ruleset e catálogos, com `catalog_id`, versão semântica e hash
de conteúdo. Catálogos são validados antes do uso: IDs únicos, referências resolvidas, limites
numéricos, schemas compatíveis e efeitos permitidos. Atualização do catálogo cria nova versão; uma
partida em curso continua presa às versões registradas. Replays falham com erro explícito se o pacote
exato não estiver disponível, sem substituição silenciosa pela versão atual.

Templates podem parametrizar narrativa, mas efeitos mecânicos precisam caber no schema e nos limites
validados. A Entropia não pode criar código/regra arbitrária. O formato de catálogo e a política de
migração de partidas antigas ainda dependem dos SDDs de domínio e persistência.

## 7. Orçamento

Metas iniciais propostas para MVP com até 8 civilizações, sem limite rígido de cidades: p95 de
resolução inferior a 250 ms por mundo/turno e teto operacional de 1 s; até 64 MiB de estado canônico
por mundo antes de snapshots/índices. Esses valores precisam de harness e carga representativa antes
de virarem gates. Complexidade esperada é linear em entidades mais custos de ordenação; combate pode
ser quadrático no número de unidades em uma área, sujeito a limite/particionamento por confronto.

O núcleo tem custo de tokens igual a zero. Chamadas de IA ocorrem fora dele, com orçamento por chamada
e teto diário por jogador, conforme CLAUDE.md §2. Tokens, provedor, latência, timeout, custo estimado
e fallback podem ser registrados como metadado operacional; nenhum desses valores afeta `step`.
Prompts devem manter prefixo estável. Sem chave ou serviço, T0 mantém avanço do mundo.

## 8. Testes e harness

- Determinismo cruzado: mesmos estado, comandos, seed, PRNG, ruleset e catálogos produzem bytes
  canônicos, eventos e hash iguais em execuções repetidas e plataformas CI suportadas.
- Golden replays gravados: partidas curtas e longas com hash esperado por turno, versões de catálogo,
  entradas externas e casos de fallback; não chamar APIs reais em CI.
- Propriedades: recursos/população não negativos, escalas limitadas, refs válidas, ausência de overflow,
  custos cobrados uma vez, rejeição sem mutação, ordem aceita preservada e dano simultâneo.
- PRNG: vetores de referência por algoritmo/versão e teste de isolamento entre streams.
- Snapshots: serializar/carregar em cada versão suportada, verificar hash e equivalência com replay
  integral; testar truncamento/corrupção como erro recuperável, nunca estado aceito silenciosamente.
- Catálogos: fixtures válidas e inválidas para IDs duplicados, limites, referências e schemas.
- IA: fixtures gravadas de intenção válida, inválida, timeout e erro; confirmar fallback e log exatos.
- Harness headless de muitos turnos e até 8 civilizações para regressão de performance, consumo de
  memória, balanceamento e relatório auditável das causas diplomáticas.

## 9. Questões abertas

1. Qual convenção de índice de turno deve ser persistida, 0-based ou 1-based? **Recomendação:**
   iniciar em 1 para alinhar relatórios ao número visível pelo jogador.
2. Qual algoritmo e versão de PRNG, hash canônico e derivação de streams adotar? **Recomendação:**
   escolher um algoritmo pequeno e estável com vetores públicos no spike da stack; congelar a
   versão por partida e cobrir com golden tests antes de produção.
3. A ordem de fases proposta — especialmente diplomacia e Entropia — corresponde às dependências
   pretendidas? **Recomendação:** aprovar a ordem após cruzar os SDDs de economia, combate, diplomacia
   e Entropia; efeitos de evento só entram no turno corrente se sua fase anteceder o sistema afetado.
4. Como definir “humano presente” quando a conexão cai no meio do turno? **Recomendação:** sessão
   ativa no mundo no instante em que o turno abre; registrar a lista de participantes esperados no
   evento de abertura e encaminhar ausentes ao Governador.
5. Qual intervalo de snapshot e limite de memória são adequados à escala de cidades sem teto rígido?
   **Recomendação:** começar com snapshot a cada 20 turnos e medir estado real no harness.
6. Como reter ou redigir texto humano e payloads externos no log sem perder replay/auditoria?
   **Recomendação:** persistir representação mecânica, hash e metadados necessários; definir retenção
   do texto no SDD de segurança antes de multiplayer público.

## 10. ADRs relacionados

- ADR-0003: turnos simultâneos e ordem sequencial de ações aceita.
- ADR-0006: motor puro determinístico, event sourcing, snapshots, hash e IA como proponente.
- ADR-0008: sem relógio de turno; avanço por prontidão dos humanos presentes e Governador para ausentes.
- ADR-0007: stack Proposta; detalhes de implementação condicionados à aceitação.
