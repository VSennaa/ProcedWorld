# SDD 04 — Camadas de IA e portas

> **Status:** Proposta — depende da aprovação do GDD e deste SDD. O ADR-0007 (stack) também está
> proposto; por isso, contratos e invariantes abaixo não dependem de linguagem, banco ou provedor.
>
> **Relacionados:** ADR-0002, ADR-0005 e ADR-0006; GDD 07 (Diplomacia), 08 (Entropia e eventos) e
> 09 (Governador e Mandato).

## Objetivo

Definir as fronteiras pelas quais modelos externos ou locais podem sugerir decisões e texto sem
ganhar autoridade sobre a simulação. O subsistema escolhe a camada mais barata aplicável, limita
custo e latência, converte saídas em intenções verificáveis, registra o que foi efetivamente usado e
sempre permite que a partida avance por T0.

As camadas são: T0, heurística/utility AI determinística; T1, modelo de decisão para escolhas
tipadas frequentes; e T2, LLM para planejamento, interpretação de conversa e narrativa. T1 inicia
com Jev e pode migrar para Laya, conforme ADR-0005. T2 é um LLM barato configurável. Esses nomes
não fazem parte dos contratos.

## Responsabilidades e fronteiras

Este subsistema é responsável por:

- expor `LLMPort`, `DecisionPort` e `MemoryPort` independentes de provedor;
- montar contexto mínimo, versionado e fundamentado no estado já observado;
- solicitar, normalizar, validar sintaticamente e encaminhar intenções para validação de regras;
- selecionar e registrar fallback T0, orçamento, custo observado e motivo de degradação;
- gravar respostas normalizadas para record/replay sem nova chamada de IA;
- isolar texto livre de jogador como dado não confiável.

Não é responsável por:

- executar `step`, consultar relógio/rede dentro dele, ou alterar estado diretamente;
- decidir legalidade, custo, aceitação diplomática, transição de estado ou efeitos de eventos;
- guardar API keys, autenticar cliente, criptografar segredos ou definir cobrança; esses pertencem ao
  subsistema de segurança/chaves (a guarda de chave do jogador é proposta do ADR-0002);
- ser fonte de verdade de Mandato, Ledger, Crônica, catálogo, memória ou estado do mundo;
- renderizar UI, notificar o jogador ou escolher a política de produto para tetos monetários.

O motor entrega uma visão imutável e já autorizada do estado. A camada de IA devolve somente uma
`Intent` ou um resultado de indisponibilidade. O adaptador de regras é o único que pode transformar
uma intenção válida em `Command` para o log.

## Fluxo proposto

```text
state snapshot + catalog + mandate/ledger -> context builder -> T0/T1/T2 port
                                                     |               |
                                                     |         recorded response
                                                     v               v
                                              Intent envelope -> rule validator -> Command log -> step
                                                     |                    |
                                                     +-> deterministic T0 -+
```

`step(state, commands, seed)` recebe apenas comandos já aceitos e a seed versionada. Não recebe
prompt, resposta, modelo, preço, timeout, memória, relógio nem identificador de provedor.

Para replay mecânico, o leitor aplica exclusivamente os `AcceptedCommand` já gravados. A intenção
normalizada, a resposta da IA e o fallback ficam em `IntentEvidence` para auditoria e fixtures;
nunca são revalidados para decidir o efeito histórico. O replay não chama `LLMPort` nem `DecisionPort`.

## Contratos propostos

Os schemas têm `schema_version` e identificadores estáveis. IDs são opacos; listas de candidatos e
fatos são ordenadas de forma estável antes de entrarem no contexto ou no fallback.

```text
enum AiLayer { T0, T1, T2 }
enum AiStatus { Ok, Timeout, Unavailable, InvalidOutput, BudgetExceeded, SafetyRejected }

type FactRef = {
  kind: "entity" | "ledger_entry" | "chronicle_event" | "catalog" | "mandate",
  id: String,
  revision: UInt
}

type Intent = {
  schema_version: UInt,
  kind: String,                 // e.g. "diplomacy.propose", "entropy.select_template"
  actor_id: EntityId,
  payload: JsonObject,          // constrained by the schema for `kind`
  fact_refs: FactRef[],
  explanation_key: String?,     // optional presentation key, never an executable instruction
  source: AiLayer
}

type IntentEnvelope = {
  request_id: RequestId,
  turn: Turn,
  ruleset_version: String,
  context_hash: Hash,
  intent: Intent,
  response_hash: Hash,
  replay_ref: ReplayRef
}

type AiResult<T> = { status: AiStatus, value: T?, diagnostics: AiDiagnostics }
```

`payload` não é JSON arbitrário em tempo de execução: cada `kind` aponta para schema fechado do
catálogo/versionamento de regras. Campos desconhecidos são rejeitados. Uma intenção diplomática, por
exemplo, deve listar emissor, destinatário, termos catalogados, prazo e `fact_refs`; a aceitação é
calculada pelo motor, nunca pelo modelo. Uma intenção de Entropia só pode citar
`template_id`, alvo e parâmetros que o template permite.

```text
interface LLMPort {
  generate(request: LlmRequest, budget: CallBudget): AiResult<LlmResponse>
}

interface DecisionPort {
  choose(request: ChoiceRequest, budget: CallBudget): AiResult<ChoiceResponse>
  score(request: ScoreRequest, budget: CallBudget): AiResult<ScoreResponse>
  decide(request: BinaryRequest, budget: CallBudget): AiResult<BinaryResponse>
}

interface MemoryPort {
  read(query: MemoryQuery, budget: ContextBudget): MemorySlice
  append(entry: MemoryEntry): MemoryWriteResult
  compact(scope: MemoryScope, policy: CompactionPolicy): CompactionResult
  reset(scope: MemoryScope): ResetResult
}
```

`DecisionPort` aceita apenas pergunta versionada, alternativas enumeradas/escala limitada e fatos
permitidos; uma resposta deve retornar o ID da alternativa, score inteiro na faixa declarada ou
probabilidade quantizada, além de referências de fatos. Probabilidade não controla ação relevante
antes de calibração em dados de teste (ADR-0005).

`LLMPort` recebe instruções estáveis do sistema, schema de saída e blocos de dados separados. Ele
não recebe acesso a ferramentas, comandos de sistema, chaves, estado mutável ou permissão de chamar
qualquer porta. T2 pode redigir narrativa e converter linguagem natural em intenção; a intenção
continua sujeita a todos os validadores.

`MemoryPort` armazena contexto derivado, não autoridade. Suas camadas propostas são `working`,
`episodic`, `semantic` e `procedural`; documentos canônicos regeneráveis continuam sendo a referência
para Mandato, Resumo do Estado, Ledger, Crônica e Doutrina. Se ADR-0007 for aceito, Markdown poderá
ser a fonte canônica desses documentos e SQLite/FTS5 um índice derivado; isto não é requisito do
contrato.

## Validação, grounding e conversão

A validação ocorre nesta ordem, sempre fora de `step`:

1. validar versão, tipo, tamanho, tipos e campos fechados do schema;
2. verificar autoria, escopo delegado, Mandato e ações irreversíveis que exigem jogador presente;
3. verificar que cada `FactRef` existe, é visível ao ator, pertence à revisão informada e sustenta o
   tipo de ação; fatos mínimos são definidos pelo schema;
4. revalidar catálogo, pré-condições, capacidades, recursos, estado diplomático, limites e prazo;
5. converter a intenção aceita em um ou mais `Command` canônicos, ou rejeitar com código enumerado.

O motor repete as verificações que dependem do estado de resolução. Portanto, uma intenção aceita na
fila pode ainda produzir rejeição determinística se um comando anterior gravado tornou sua
pré-condição falsa.

Exemplo de intenção permitida:

```json
{
  "schema_version": 1,
  "kind": "diplomacy.propose",
  "actor_id": "civ:12",
  "payload": {"recipient_id": "civ:41", "term_id": "trade.grain", "amount": 3, "duration_turns": 2},
  "fact_refs": [{"kind": "ledger_entry", "id": "ledger:12:41:889", "revision": 4}],
  "source": "T2"
}
```

Uma citação plausível não basta: o validador do `term_id` define quais fatos, quantidades e estados
são aceitáveis. Prosa explicativa não pode acrescentar cláusula nem alterar a interpretação.

## Texto não confiável e defesa contra prompt injection

Mensagem, negociação ou nome dado pelo jogador é conteúdo não confiável. O construtor de prompt o
serializa em bloco delimitado e rotulado `UNTRUSTED_PLAYER_DATA`, separado das instruções e do schema.
Ele nunca é concatenado como instrução, ferramenta, policy ou documento de memória procedural.

O subsistema limita comprimento, normaliza Unicode e preserva o original somente onde a política de
dados permitir. A interpretação retorna uma intenção estruturada; instruções inseridas no texto,
tentativas de mudar regras, pedidos de segredo, URLs e alegações sem `FactRef` são apenas texto e não
recebem privilégio. Ações de cartão estruturado são tentadas antes de T2, como decidido no GDD 07.

Não se registra API key, cabeçalho de autorização, prompt de sistema secreto, credencial, cadeia de
raciocínio do provedor ou texto que a política de retenção proibir. Logs de depuração usam hashes,
contagens e códigos, não esses valores.

## Modelo de dados e invariantes verificáveis

O registro lógico proposto é independente de armazenamento:

```text
AiInvocation {
  invocation_id, request_id, turn, actor_id?, layer, provider_config_id,
  request_schema_version, context_hash, status, latency_ms,
  input_tokens?, output_tokens?, cost_microunits?, response_hash, replay_ref,
  fallback_reason?, recorded_at_sequence
}

RecordedAiResponse {
  replay_ref, response_schema_version, normalized_payload, response_hash,
  fixture_class, redaction_version
}

MemoryEntry {
  id, scope, layer, kind, content_ref, source_fact_refs, created_turn,
  supersedes?, is_latest, token_estimate
}
```

Invariantes:

- cada `AiInvocation` tem `request_id` único e está ligado a no máximo uma resposta normalizada;
- cada intenção aceita tem `context_hash`, versão de regras e `replay_ref` imutáveis no log;
- `hash(normalized_payload) == response_hash`; a resposta usada no replay é a gravada, não a atual;
- `source == T0` não depende de rede e informa motivo se substituiu T1/T2;
- nenhuma intenção vira comando sem schema válido, grounding e validação de regras;
- memória aponta para fatos, pode ser supersedida sem apagar histórico, e nunca é usada como fato
  quando diverge do estado; reset de memória não muda o hash do estado;
- custos e tokens são não negativos; uma chamada acima de qualquer limite não emite intenção externa;
- o modelo/configuração pode mudar entre invocações, mas nunca muda a interpretação de uma resposta
  já registrada.

Se ADR-0007 for aceito, esses registros podem ser persistidos em PostgreSQL e corpos de fixtures em
artefatos/armazenamento apropriado. A escolha de tabelas, índices e retenção ainda é proposta.

## Falhas, fallback e determinismo

T0 é a implementação determinística de referência: filtra ações legais, aplica Mandato e regras,
pontua utility/candidatos por dados versionados e resolve empate por ID estável. Para a Entropia, T0
seleciona somente template, alvo e parâmetros elegíveis com o PRNG versionado; para diplomacia, nunca
aceita pelo texto. T0 também fornece texto canônico de catálogo quando a narrativa falha.

Timeout, erro de transporte, resposta inválida, rejeição de segurança, ausência de chave/serviço ou
orçamento esgotado resultam em T0 no mesmo turno. Se não houver ação legal, a saída T0 é
`no_op`/adiamento canônico quando a regra o permitir, com motivo gravado; não há espera por IA.

Entram no log auditável: sequência, turno, `request_id`, camada tentada/usada, versão de schema e
regras, hashes de contexto e resposta, status, motivo do fallback, orçamento reservado/consumido,
latência, tokens/custo quando fornecidos, intenção normalizada ou marcador T0, comandos derivados e
códigos de rejeição. A resposta normalizada é preservada via `replay_ref` para replay.

Nunca entram em `step`: prompt, resposta bruta, texto de jogador, memória, custo, tokens, latência,
nome/versão de provedor, timeout, exceção, rede, data/hora real, API key nem escolha de fallback feita
por informação não registrada. O único efeito de falha em `step` é o comando T0/no-op que já está no
log.

## Orçamento e teto de custo

Os números são configuração de produto ainda em aberto. Cada pedido carrega `CallBudget` com prazo
máximo, tokens de entrada/saída, custo máximo em microunidades e prioridade; o planejador reserva o
pior caso antes da chamada e libera a diferença após contabilização. Um `TurnBudget` limita tempo
total de portas, memória de contexto, quantidade de chamadas T1/T2 e custo por turno. Um
`PlayerDailyBudget` limita o total atribuível à chave daquele jogador, como propõe ADR-0002.

Ordem proposta de admissão: obrigação de sobrevivência/T0, decisão tipada T1, interpretação ou
narrativa T2. Ao faltar qualquer orçamento, a chamada não é iniciada e T0 é registrado. Contexto usa
prefixo estável e documentos por hash para favorecer cache do provedor, sem depender dele para
correção. Métricas devem separar tokens estimados de tokens/custo confirmados pelo provedor.

O teto do modelo de decisão ainda depende da pergunta aberta do ADR-0002: chave do operador ou do
jogador. A atribuição de custo deve permanecer explícita em `provider_config_id` sem conter segredo.

## Configuração e troca Jev → Laya

`DecisionPort` é selecionada por configuração, por exemplo:

```text
decision.adapter = "runware-systemone"
decision.model = "typesafe:jev@latest"   # futuro: "runware:laya@1"
decision.question_set_version = "v1"
```

O adaptador traduz somente o contrato comum. Perguntas, escalas, opções e calibração são dados
versionados; uma migração exige fixtures e nova calibração antes de probabilidades influenciarem ações
relevantes. A hospedagem de Laya não é decidida: o host de referência não tem GPU e ADR-0005 exige
spike medido antes da migração. Se ADR-0007 for aceito, variáveis de ambiente/configuração podem
fornecer essa seleção; o contrato não exige Rust, Godot ou Runware.

## Estratégia de testes

- **Unitários:** schemas fechados, grounding ausente/falso, isolamento de texto não confiável,
  admission control e mapeamento de cada falha para T0.
- **Propriedades:** mesma visão ordenada + seed + regras + log produz os mesmos comandos e hash;
  saída arbitrária de porta nunca altera estado sem validação; custo reservado nunca ultrapassa teto;
  `reset(MemoryPort)` não altera a simulação.
- **Record/replay:** fixtures gravadas incluem resposta normalizada, metadados redigidos e comando
  esperado. CI simula `LLMPort`/`DecisionPort`; nunca chama API real.
- **Golden replays:** uma partida com T0, outra com respostas registradas T1/T2 e casos de timeout,
  schema inválido e orçamento esgotado devem reproduzir hash por turno.
- **Harness longo:** mundos com bots, Entropia e diplomacia verificam progresso sem provedor, limites
  de memória/contexto, orçamento por turno e relatório que liga cada ação diplomática a fatos do
  Ledger.
- **Integração de adaptador:** contra servidor falso/gravado, verifica serialização e normalização;
  testes reais de provedor ficam fora do CI determinístico e não usam chave de jogador.

## Perguntas abertas para o usuário

1. Quem paga e fornece a chave T1 durante Jev e depois Laya: operador, jogador, ou modelo híbrido?
   **Recomendação:** definir operador para T1 no alfa, com teto global e por civilização; manter T2
   BYOK para que a experiência não transfira custo narrativo ao operador sem controle.
2. Qual é a política de retenção e visibilidade do texto livre de jogador nas respostas gravadas?
   **Recomendação:** gravar no replay apenas a intenção normalizada e hashes; guardar texto bruto em
   armazenamento separado, protegido e com retenção curta somente se for indispensável a suporte.
3. Quais valores iniciais de prazo por turno, tokens, memória e teto diário devem bloquear chamadas?
   **Recomendação:** aprovar números apenas após um spike com fixtures representativas; até lá, expor
   todos como configuração obrigatória e fazer fallback quando ausentes.
4. O que acontece quando uma ação irreversível do Governador está pendente e o jogador segue ausente?
   **Recomendação:** registrar a proposta e aplicar `no_op` até confirmação, coerente com o GDD 09;
   confirmar a lista final de ações irreversíveis no SDD do Governador/Diplomacia.
5. Qual armazenamento e criptografia de fixtures/respostas de replay atendem privacidade e suporte?
   **Recomendação:** decidir no SDD 09/11 antes de produção; este SDD exige somente redaction,
   integridade por hash e capacidade de replay sem rede.
