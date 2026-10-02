# SDD 06 — Entropia

> **Status:** rascunho de proposta. Depende da aprovação formal do GDD e do SDD.
> Este documento define contratos e invariantes independentes de linguagem, banco ou cliente.
> Referências: GDD 02, 05, 06, 08 e 12; ADR-0005 e ADR-0006.

## Objetivo e fronteiras

A Entropia é a diretora de eventos procedurais de um mundo. Ela transforma vulnerabilidades e
oportunidades já presentes no estado em cartões de evento, escolhas e cadeias limitadas. Ela
controla ritmo e variedade, não uma configuração de dificuldade: a personalidade do mundo apenas
pondera templates que já seriam justos e elegíveis.

São suas responsabilidades:

- manter e gastar o orçamento de tensão mundial e as reservas por civilização;
- calcular janelas da curva por era, elegibilidade, proteção e opções válidas;
- selecionar, por sorteio reprodutível, alvos e parâmetros delimitados pelo catálogo;
- abrir, resolver e encadear eventos de clima, tecnologia, diplomacia, revolta, magia e relevo;
- produzir ou solicitar texto narrativo fundamentado, sem lhe dar efeito mecânico;
- registrar fatos, decisões e resultado no log de comandos e na Crônica derivada.

Não são responsabilidades da Entropia:

- calcular produção, crescimento, `D`, `G`, `W`, `E`, `P`, `C`, `L`, aceitação diplomática ou
  colapso; esses valores pertencem às regras dos respectivos subsistemas;
- criar efeitos livres, recursos, entidades, tecnologias ou regras fora do catálogo;
- escolher ações de jogadores, alterar Mandatos, aceitar tratados ou resolver diplomacia;
- revelar orçamento, pesos, seed ou lógica de seleção ao jogador;
- chamar IA, rede, relógio, armazenamento, telemetria ou qualquer I/O durante `step`.

**Base decidida.** A personalidade é `cyclical`, `contentious` ou `transformative`, sem nível de
dificuldade. Ela sugere, respectivamente, clima sazonal, estável ou instável; na criação o jogador
pode substituir o clima. Mudanças permanentes de relevo e fenômenos mágicos também são templates.

## Contratos propostos

Os nomes abaixo são contratos lógicos; não pressupõem Rust, Godot ou PostgreSQL. Se o ADR-0007 for
aceito, podem ser tipos do núcleo Rust, dados versionados e persistência PostgreSQL, mas isso não
muda seus campos nem suas regras.

```text
WorldEntropyState {
  personality: EntropyPersonality
  suggested_climate: ClimatePattern
  era_id: EraId
  world_budget: TensionPoints
  civil_budget: Map<CivilizationId, TensionPoints>
  phase: EraPhase                 // reading | crisis | respite
  protections: List<Protection>
  active_chains: List<EventChain>
  catalog_version: CatalogVersion
}

EntropyTemplate {
  id: TemplateId; version: TemplateVersion
  category: EventCategory; tension_cost: 1..4
  personality_weight: Map<EntropyPersonality, PositiveInt>
  triggers: PredicateSpec; target_selector: TargetSelectorSpec
  parameter_rules: List<BoundedParameterSpec>
  effect_ops: List<ValidatedEffectOp>
  choices: List<EventChoiceSpec>                 // 2..3, salvo evento informativo
  chain: Optional<ChainSpec>; protections: ProtectionSpec
  narrative: NarrativeSpec
}
```

`EventCategory` inclui ao menos `climate`, `technology`, `diplomacy`, `revolt`, `epidemic`,
`magic` e `terrain_change`. Todo template tem id e versão imutáveis; edição incompatível cria nova
versão. `ValidatedEffectOp` é operação finita do motor, nunca texto interpretado.

```text
EntropyOpportunity {
  opportunity_id: OpportunityId
  template_id: TemplateId@TemplateVersion
  target_ids: Sorted<EntityId[]>
  resolved_parameters: Map<ParameterName, Integer | EntityId>
  cited_fact_ids: Sorted<FactId[]>
  legal_choices: List<ChoiceId>
  tension_cost: TensionPoints
}

ActivateEntropyEvent {
  opportunity_id: OpportunityId
  selected_target_id: EntityId
  chosen_choice_deadline: Turn
  selection_source: entropy_t0 | decision_t1 | llm_t2
}

ResolveEntropyChoice { event_id: EventId, choice_id: ChoiceId, actor_id: ActorId }
```

`EntropyOpportunity` é derivada deterministicamente do estado anterior, catálogo e seed; não é
autorização implícita. `ActivateEntropyEvent` e `ResolveEntropyChoice` são comandos validados e
logados. Os parâmetros são resolvidos pelo motor antes da escolha: T1/T2 não pode inventá-los.

```text
EntropyPort.choose(options: NonEmpty<EntropyOpportunity[]>, context: GroundedEntropyContext)
  -> EntropySelectionIntent { opportunity_id, cited_fact_ids }

NarrativePort.render(event: EventView, facts: NonEmpty<Fact[]>)
  -> NarrativeIntent { title, body, cited_fact_ids }
```

`EntropyPort` pode ser T0, `DecisionPort` T1 (ADR-0005) ou LLM T2. O T1/T2 só escolhe uma opção
já elegível e escreve texto; T0 escolhe por peso fixo e desempate por id. `NarrativeIntent` nunca
é comando. As portas recebem somente ids e fatos permitidos; texto de jogador é dado não confiável
e isolado do prompt.

## Fluxo por turno e seed

Após as fases que atualizam os fatos do mundo, o motor executa a avaliação abaixo. A posição exata
na ordem global de turnos deve ser definida pelo SDD 03; esta interface exige somente que ela seja
fixa e registrada.

```text
evaluate_entropy(state, turn_seed):
  phase = curve_phase(state.era_id, state.turn)
  candidates = eligible_templates(state, phase)
  opportunities = resolve_targets_and_parameters(candidates, turn_seed)
  return sort_by_stable_id(opportunities)

select_fallback(opportunities, turn_seed):
  weighted = apply_personality_weight(opportunities)
  return seeded_draw(weighted, turn_seed, "entropy-selection")
```

O PRNG, sua versão, a derivação de sub-seed e a ordem de ordenação são parte do catálogo/protocolo
versionado. A mesma entrada produz as mesmas oportunidades e o mesmo fallback. Escolha T1/T2 aceita
vira comando gravado; replay nunca chama o provedor novamente.

## Modelo de dados e invariantes verificáveis

**Proposta de orçamento inicial.** No início de cada era:

```text
B = 12 + 2 * living_civilizations
b[c] = clamp(2, 6, 2 + floor(P_civ[c] / 25))
```

`B` é o orçamento mundial e `b[c]` a reserva da civilização. São pontos de tensão, não dano.
Uma ativação gasta seu custo de ambos quando tiver alvo civilizacional; evento de mundo só gasta
`B` conforme regra declarada pelo template. Valores são proposta para o harness, não balanceamento
aprovado. O orçamento não é apresentado ao jogador.

**Proposta de curva.** Cada era distribui janelas de leitura 25%, crise 55% e respiro 20%; numa era
de oito turnos isso sugere leitura nos turnos 1–2, crise 3–6 e respiro 7–8. Respiro não inicia
catástrofe grave. Em eras maduras aumenta-se a complexidade das cadeias, não o custo máximo de quatro
nem dano bruto.

Invariantes que o motor deve rejeitar ou testar:

- `0 <= world_budget` e `0 <= civil_budget[c]`; nenhum gasto excede orçamento ou reserva aplicável;
- custo de template é 1, 2–3 ou 4 e toda ativação aponta para template/versionamento existente;
- alvo existe, é permitido e pertence à lista ordenada de candidatos; parâmetro respeita tipo, faixa
  e fonte do template;
- gatilho cita fatos concretos (tile, cidade, grupo, tecnologia, estado ou Ledger), nunca alvo
  abstrato; `cited_fact_ids` existe no estado que habilitou o cartão;
- há resposta útil e legal: mitigação com custo viável ou alternativa diplomática aberta;
- evento não reduz `C` diretamente a zero, não remove última cidade controlada e não causa colapso
  sozinho; crises só agravam causas avaliadas pelas regras de `P` e colapso;
- proteção proposta: crise de custo >=2 concede três turnos contra a mesma categoria e dois contra
  outra crise de custo >=3, sem bloquear consequências anunciadas;
- cadeia contém nó inicial, estado pendente, no máximo duas bifurcações, duração de 2–4 turnos e
  custo acumulado <=5; sucessores são condicionais, não sorteios opacos;
- personalidade só altera `personality_weight` de elegíveis (peso inicial preferido 2, demais 1),
  jamais custo, proteção, orçamento, justiça ou dificuldade;
- `D`, `G`, `W`, `E`, `P`, `C` e `L` preservam escalas e fórmulas do GDD 12; Entropia só os lê ou
  solicita operações já autorizadas em template.

Pontos sem template justo não obrigam evento: ficam reservados para a próxima janela ou expiram,
conforme política proposta de balanceamento. Isso impede catástrofe fabricada para gastar cota.

## Templates especiais

`terrain_change` declara conjunto de tiles, transformação permitida, permanência, causa visível,
limite espacial e reversibilidade, se houver. Sua validação consulta topologia hexagonal cilíndrica
e regras de mapa; nunca aceita coordenada fora do mapa ou operação textual livre.

`magic` declara fenômeno, escopo (`tile`, `city`, `group` ou `civilization`), gatilho, práticas
elegíveis e reações políticas (`adopt`, `ban`, `regulate`). Aderir pode habilitar práticas mágicas
sujeitas às mesmas vagas, manutenção e tetos de tecnologia; template não as concede fora dessas regras.

Tecnologia emergente referencia descoberta elegível da árvore/dados. Revolta referencia grupo
político/cidade reais e aplica regras sociais. Incidente diplomático exige fato de fronteira, tratado
ou Ledger; o motor, não Entropia, calcula aceitação, confiança ou dívida.

## Interferir na Entropia — proposta

Nação pode emitir `AttemptEntropyInterference` para evento pendente: `predict`, `appease`,
`redirect` ou `invoke`. O comando declara evento, ambição (`low|medium|high`) e pagamento legal;
o template declara verbos permitidos. Jogador recebe causa, preço e faixa de risco, não seed,
chance exata ou orçamento oculto.

Motor resolve o resultado por sub-seed `entropy-interference`, com tabela versionada. A proposta é
que custo certo seja proporcional à ambição (recurso, coesão, legitimidade, dívida ou prática) e
fracasso aplique consequência catalogada de severidade maior que benefício esperado. Sucesso raro
pode reduzir uma etapa, trocar alvo dentro dos alvos legais ou revelar fato futuro; nunca anula
justiça, cria recurso livre, viola proteção ou remove escolhas de demais nações. Custo e resultado
entram no log; a mecânica deve permanecer quase sempre pior que analisar e responder ao evento.

**Pergunta de balanceamento:** tabela, probabilidades e custos concretos ficam abertos até harness;
não devem ser escolhidos por LLM.

## Falhas, fallback e registro

Falha de catálogo, schema, grounding, prazo, orçamento ou validação descarta intenção e usa T0, se
ainda houver opção legal. Sem opção legal, não há ativação. Falha do texto usa narrativa canônica do
template. Erro de provedor, timeout, indisponibilidade, excedente de custo ou resposta malformada
nunca bloqueia o turno.

Log de comandos registra versão do catálogo e PRNG, turno, command id, template/version, alvo,
parâmetros resolvidos, fatos citados, custo, orçamento antes/depois, escolha, resultado de cadeia,
fonte de seleção, motivo de fallback e hash de estado. Para IA, registra id da fixture/resposta
aceita e métricas agregadas de tokens/custo, sem depender delas para replay.

Nunca entram em `step`: prompt bruto, resposta narrativa bruta, raciocínio do modelo, segredo,
chave, URL de provedor, relógio, latência, preço corrente, telemetria, ordem de chegada ou chamada
de rede. Podem ficar em observabilidade protegida; não são estado autoritativo nem entrada de replay.

## Orçamentos propostos

| Recurso | Limite inicial | Política de degradação |
|---|---:|---|
| Avaliação determinística | <= 10 ms por mundo/turno | medir; simplificar índice/catálogo, nunca pular regra |
| Seleção T1/T2 | <= 1 chamada por oportunidade ativável | timeout => T0 |
| Narrativa T2 | <= 1 chamada por evento ativado | texto canônico |
| Contexto de escolha | <= 600 tokens | truncar para fatos canônicos ordenados |
| Texto narrativo | <= 250 tokens | truncar/usar canônico |
| Custo de IA | teto diário BYOK aplicável | T0 e texto canônico |
| Estado em memória | O(eventos ativos + proteções + cadeias) | limites acima; expirar resolvidos |

Tempo e tokens são propostas mensuráveis, a calibrar no harness e em provedores reais antes de
virarem gate de CI. Se ADR-0007 for aceito, implementação mede também alocação e latência ponta a
ponta, sem tornar essas medidas parte do `step`.

## Estratégia de testes

- **Determinismo:** mesma seed, catálogo e comandos geram oportunidades, comandos, hash e orçamento
  idênticos entre máquinas; variar ordem de estruturas de entrada não muda resultado.
- **Propriedades:** gerar estados/catálogos válidos e provar invariantes de orçamento, alvo, faixa,
  proteção, resposta útil, exclusão de colapso e limite de cadeias.
- **Golden replays:** fixtures de seca, descoberta, incidente, revolta, magia e relevo, incluindo
  respiro, expiração de orçamento, fallback e interferência, com hashes por turno.
- **Fixtures gravadas de IA:** resposta T1/T2 válida, inválida, atrasada, sem grounding e indisponível;
  CI reproduz gravação e nunca chama API real.
- **Harness longo:** mundos com personalidades e climas cruzados, eras e bots T0; mede frequência,
  concentração por civilização, repetição, cadeias, custo, tempo e fallback. Alimenta o balanço de
  `B`, `b[c]`, curva, `W` e `E`.
- **Contrato:** template antigo usado em replay permanece disponível ou migra por conversor versionado,
  com equivalência de efeitos comprovada.

## Perguntas abertas para o usuário

1. Quais verbos de interferência entram no lançamento: todos os quatro propostos ou conjunto menor?
   **Recomendação:** começar com `predict` e `appease`; são auditáveis e preservam agência.
2. Interferência pode afetar outra civilização sem consentimento, além dos efeitos normais do evento?
   **Recomendação:** não; permitir só alvo do próprio evento e alvos já declarados no template.
3. Quais fenômenos mágicos entram no catálogo inicial?
   **Recomendação:** três, um por escopo (`tile`, `group`, `civilization`), cada um com aderir,
   proibir e regulamentar, antes de magia ofensiva ou criação de relevo.
4. Números iniciais de orçamento, curva, proteção e cadeia ficam como parâmetros de balanceamento?
   **Recomendação:** sim; aprová-los como defaults de dados, não constantes, e endurecer após harness.
