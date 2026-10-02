# SDD 15 — Regras de domínio

> **Status: proposta para revisão do SDD.** GDD aprovado em 2026-10-01; ADR-0007 aceito.
> Este documento fixa contratos e invariantes portáveis; números e políticas marcados como
> proposta exigem validação no harness e aprovação de produto.

## 1. Responsabilidades e fronteiras

Este subsistema transforma `AcceptedCommand` em mudanças mecânicas de economia, cidades,
tecnologia, sociedade e unidades. É o proprietário das contas por turno, dos efeitos de
catálogos e dos `DomainEvent` causais desses domínios.

Ele inclui:

- recursos, estoques, produção, manutenção, contratos e suas entregas;
- cidades, postos de trabalho, crescimento, migração automática e grupos híbridos;
- pesquisa, tecnologias, práticas e seus limites;
- governo, políticas, reformas, satisfação de grupos, `C`, `L`, `S`, `D`, `G`, `W`, `E` e `P`;
- unidades, reserva de ataque, movimento e resolução de combate em fase própria.

Ele não inclui autenticação, presença, prontidão, UI, projeções para cliente, persistência,
rede, geração de mapa, visibilidade, diplomacia como máquina de estados, Mandato, seleção de
Entropia, memória/Crônica ou chamada de IA. Esses componentes podem propor comandos ou consumir
eventos; não executam regras deste documento. O motor de núcleo é dono de `step`, do PRNG,
serialização canônica e de `StateHash`.

Este módulo é uma biblioteca Rust pura; isso não altera os
contratos a seguir nem pressupõe Godot ou PostgreSQL.

## 2. Contratos

### 2.1 Entrada e saída do resolvedor

```text
resolve_domains(
  state: WorldSnapshot,
  commands: List<AcceptedCommand>,
  ruleset: RulesetRef,
  seed: SeedRef
) -> Resolution { state: WorldSnapshot, events: List<DomainEvent> }
```

`resolve_domains` é parte pura do `step`; recebe somente comandos já aceitos, no
`accepted_sequence` crescente. `WorldSnapshot` é o estado autoritativo em memória, não uma
projeção do cliente. O replay aplica os mesmos `AcceptedCommand`, nunca uma intenção ou resposta
de IA.

```text
AcceptedCommand {
  command_id, world_id, turn, accepted_sequence, actor_id,
  origin: player | governor | bot | entropy | fallback | system,
  kind, payload_canonical, grounding: List<GroundingRef>, ruleset_ref: RulesetRef
}
GroundingRef { kind, id, revision }
DomainEvent { event_id, turn, phase, kind, causal_command_id?, subject_ids, payload_canonical }
```

Comandos de domínio propostos (schema fechado, campos semânticos):

```text
SetCityPriority(city_id, priority, pinned_workplaces)
SetSuspension(target_id, suspended)
OfferContract(counterparty_id, offered: Bundle, requested: Bundle, route_id, duration_turns)
AcceptContract(contract_id) | CancelContract(contract_id)
SetResearchProject(technology_id?, investment_band)
SetPractice(practice_id, active, priority)
StartReform(axis, target_position | policy_id)
SetPolicy(axis, policy_id)
MoveUnit(unit_id, destination_hex) | DeclareAttack(unit_id, target_id, stance)
```

`Bundle` é um vetor ordenado por `ResourceId`; cada item tem `resource_id` e quantidade inteira
positiva. Recursos negociáveis e limites vêm do catálogo. Conhecimento e cultura não são itens
negociáveis. Sucessão e efeitos de revolta são especificados abaixo; detalhes marcados como proposta
aguardam decisão do GDD.

### 2.2 Catálogos declarativos

```text
RulesetRef { id, version, content_hash }
TechnologyDef { id, prerequisites, research_cost, practice_def?, effects: List<EffectOp> }
PracticeDef { id, maintenance_production, capped_effects: List<EffectOp> }
PolicyDef { id, axis, position, prerequisites, effects: List<EffectOp> }
UnitDef { id, movement, attack, defense, upkeep: Bundle }
EffectOp = AddClamped | ModifyYield | ReserveResource | EnableAction | ApplyStatus
```

Proposta: `EffectOp` é uma DSL fechada e interpretada, sem código arbitrário, I/O ou seleção não
ordenada. O validador rejeita catálogo com referência ausente, faixa inválida, efeito fora do teto
ou conflito de versão. Uma descoberta da Entropia só instancia `TechnologyDef`/`PracticeDef`
parametrizada dentro de seu template validado.

## 3. Ordem canônica do turno

Após a abertura, presença e aceitação de comandos, o `step` resolve na ordem abaixo. A fase de
entrada preserva `accepted_sequence`; dentro das demais, coleções usam ID estável e empates usam
PRNG versionado derivado da seed.

| Fase | Resolução |
|---|---|
| 1. Entrada | Aplicar comandos administrativos e reservas imediatas válidas; `DeclareAttack` consome custo e reserva a unidade, mas não causa dano. |
| 2. Movimento | Resolver `MoveUnit` em ordem canônica; entrada em hex hostil exige ataque declarado. |
| 3. Combate | Formar confrontos por hex/ID, calcular dano e perdas simultâneas, então ocupação e retirada. |
| 4. Trabalho e produção | Alocar postos, extrair depósitos, calcular rendimentos e produção disponível. |
| 5. Sustento e contratos | Pagar manutenção, reservar alimento local e obrigações críticas, liquidar contratos e consumir comida. |
| 6. Cidade e tecnologia | Aplicar obras, crescimento, migração, práticas, pesquisa e progresso de reforma. |
| 7. Sociedade | Atualizar grupos híbridos, `S`, `C`, `L`, `D`, `G`, `W`, `E`, `P` e elegibilidades sociais. |
| 8. Eventos externos | Aplicar apenas comandos Entropia já aceitos e válidos contra o estado pós-sustento. |
| 9. Síntese | Emitir eventos, verificar marcos e entregar o próximo `WorldSnapshot` ao núcleo. |

Combate em fase própria, perdas simultâneas e a reserva no ato de declarar são decisões do GDD.
O detalhamento de fórmulas de ataque, defesa, terreno e retirada é proposta de catálogo, não
decisão deste SDD.

## 4. Modelo de dados e resolução

```text
Civilization { treasury_wealth, cohesion: C, legitimacy: L, government, research, practices }
City { population, food_stock, housing, stability: S, workplaces, groups, construction_queue }
Group { city_id, function: cultivators | crafts | merchants, population, demands, satisfaction: A_g }
Contract { id, parties, offered, requested, route_id, remaining_turns, status, defaults_by_party }
Unit { id, owner_id, hex_id, def_id, movement_left, reserved_attack? }
```

Os três `Group` são a única representação social por função: população por grupo soma a população
da cidade e as demandas políticas pertencem a eles. Migração interna é automática; nenhum comando
muda diretamente a população. Não há limite rígido de cidades: sobrecarga administrativa deve
afetar `S` e poder tornar secessão elegível segundo regra futura.

### 4.1 Economia e contratos

Produção e manutenção usam inteiros. A ordem proposta é rendimentos, manutenção, reserva local de
comida, contratos, consumo, obras/crescimento e Ledger. Riqueza é moeda; pacotes podem conter bens
e riqueza em ambos os lados. Cada liquidação emite `ContractDelivered`, `ContractDefaulted`,
`ContractInterrupted` ou `ContractClosed`, com prometido, realizado, causa e partes; a projeção
diplomática cria o `LedgerEntry` correspondente fora deste subsistema.

Proposta de proporção para pacote misto: para cada lado `s`, calcule
`r_s = min(1, min_itens floor(disponível(item) / prometido(item)))`, após reservas e capacidade
de rota; trate riqueza como disponível, mas não como capacidade de carga. Defina
`r = min(r_origem, r_destino)`. Cada entrega é `floor(prometido(item) × r)`; capacidade é testada
novamente sobre bens na ordem `ResourceId`. Assim, um lado parcialmente capaz reduz o pacote
inteiro dos dois lados, não cria crédito nem transfere mais do que foi prometido. A fórmula é
**proposta**: a regra para pacote cujo único item é riqueza e a imputação de inadimplência parcial
precisam de aprovação.

### 4.2 Cidades, pesquisa e sociedade

O alocador escolhe postos por prioridade de cidade e desempate estável; exceções fixadas pelo
jogador só valem se ainda elegíveis. Crescimento requer regra catalogada de excedente e moradia.
Leis podem restringir a migração automática (decidido em 2026-10-01). Migração propõe uma pontuação inteira por origem/destino (comida, moradia e `S`), move uma unidade
por par canônico até o limite de fluxo e emite `PopulationMigrated`; pesos e limite são proposta.

Tecnologias têm árvore base fixa em dados; eventos podem introduzir descobertas emergentes por
template. Há no máximo três práticas ativas, inclusive mágicas; ativação exige tecnologia dominada,
vaga, pré-requisitos e manutenção. Falta de produção suspende práticas por prioridade e ID estável.
Pesquisa parcial é por tecnologia; ativar prática não é consequência automática de concluí-la.

O governo tem os eixos discretos `centralization`, `participation` e `economic_obligation`, três
posições cada, e uma reforma muda somente um eixo por vez. Política/reforma só pode aplicar efeitos
de catálogo. Revoluções moderadas adotam Mandato proposto e reversível; severas impõem Mandato do
novo regime, alterável depois por reforma. A classificação da intensidade permanece aberta.

As fórmulas compartilhadas usam a fonte canônica `12-variaveis-e-formulas.md`:

```text
D_c = min(20, 5 * units_without_food_c + essential_maintenance_unpaid_c * 5)
G_c = clamp(0, 20, ceil(20 * population_with_Ag_below_40_c / population_c))
P_c = clamp(0, 100, 2*D_c + 2*G_c + W_c + E_c + ceil((100-S_c)/10) - floor(C/5))
C' = clamp(0, 100, C + 4*fulfilled_commitments - 5*broken_commitments
  - weighted_mean(T_g)/10 - 2*active_conflicts + Delta_luxury)
```

`P_c` segue a fórmula canônica de `docs/gdd/12-variaveis-e-formulas.md`: arredondamento
para cima do termo de estabilidade e divisão inteira de `C/5`, equivalente ao piso para
`C` não negativo. Cada termo da atualização de `C` fica limitado por turno conforme o GDD 06.
`D_civ`, `G_civ` e `P_civ` são médias ponderadas por população; população zero produz zero onde
aplicável. `W` e `E` são provisórios e não devem ser tratados como balanceamento decidido.

Correção de alinhamento com a fórmula compartilhada do GDD 12, aplicada em 2026-10-01.

## 5. Invariantes verificáveis

- Estoques, população, progresso, capacidade de rota e contadores de turno nunca são negativos.
- Cada recurso entregue pertence a uma parte antes da transferência; a soma mundial só muda por
  produção, consumo, extração, perda ou efeito de catálogo explicitamente emitido.
- Comida reservada para demanda local e metal reservado para reparo obrigatório não são exportados.
- Um `Contract` liquidado no turno é processado uma vez, por ID estável; riqueza não ocupa carga.
- Cada pessoa pertence a exatamente uma cidade e a um dos três grupos; a soma dos grupos é a população.
- Uma cidade trabalha no máximo um trabalhador por posto/tile, e um tile não é trabalhado por duas cidades.
- Há no máximo três práticas ativas; prática inativa não cobra nem produz efeito.
- Governo sempre contém três eixos em posições válidas; há no máximo uma reforma institucional em curso.
- `C`, `L`, `S`, `A_g`, `D`, `G`, `W`, `E` e `P` permanecem em suas escalas definidas.
- Unidade reservada para ataque não move, ataca nem paga a reserva uma segunda vez no mesmo turno.
- Mesmo `WorldSnapshot`, lista ordenada de `AcceptedCommand`, `RulesetRef` e seed produzem os mesmos
  `DomainEvent` e `StateHash` em qualquer máquina.

## 6. Sucessão e entrada tardia

Esta seção distingue decisões do GDD de contratos ainda propostos. Os comandos aceitos entram no
event log como `AcceptedCommand`; a validação usa estado, catálogo e `RulesetRef` versionados. A IA
pode propor opções, mas não escolhe nem aplica transições.

### Colapso e comunidades sucessoras

Colapso ocorre quando `C = 0` por 2 turnos ou quando uma crise grave validada deixa o governo
incapaz por 3 turnos (GDD 10; limiares compartilhados em `gdd/12-variaveis-e-formulas.md`). Uma
revolução que resolve a perda de controle também pode produzir colapso parcial (GDD 06). Não há
eliminação por um evento isolado da Entropia.

No colapso, cidades preservam população sobrevivente, obras existentes, grupos e ligações locais;
podem integrar uma sucessora, ficar autônomas ou perder infraestrutura conforme causas registradas
(GDD 04 — decidido). O motor gera comunidades sucessoras com base em controle territorial, distância
das cidades, população e regras de sucessão do catálogo. O jogador escolhe uma comunidade para
continuar; as demais tornam-se civilizações de bot com territórios, relações e reivindicações
registrados (GDD 06/10 — decidido). O motor conserva a Crônica da civilização anterior.

Na sucessora escolhida, população e capacidade administrativa ficam em 40–60% das anteriores;
reservas ficam limitadas a até dois turnos de manutenção e permanecem ativos no máximo dois legados.
Essas faixas são valores iniciais sujeitos a balanceamento (GDD 10 e `gdd/12-variaveis-e-formulas.md`).
O destino dos excedentes é ruína, patrimônio disputado, conhecimento degradado ou memória histórica,
conforme regra/catalogo aplicável. Não se duplica estado ao dividir comunidades.

Legados têm condição, efeito limitado, origem e dono atual. Patrimônio pode ser capturado com a cidade;
se não for escolhido como instituição pela sucessora, fica como ruína. Memória social só sobrevive se
a comunidade sucessora contiver população suficiente da anterior. O limiar de população e o algoritmo
de atribuição são **proposta**, pois o GDD não os fixa. A regra de até dois legados ativos é decidida;
qual legado permanece em caso de concorrência também depende da escolha registrada e de regra de
catálogo determinística.

Relações são herdadas apenas pelo sucessor identificável que herdar o território correspondente:
dívidas, reivindicações e traições passam a esse sucessor; obrigações de defesa expiram. O novo regime
pode ratificar, renegociar ou repudiar tratados, com consequências registradas (GDD 07 — decidido).
Cada entrada do `Ledger` permanece imutável e é referenciada; a projeção de relação do sucessor não
reescreve nem replica fatos para comunidades sem território/herdeiro identificável.

### Entrada tardia e comandos do motor

Entrada tardia oferece duas opções decididas: fundar comunidade nova como assentamento protegido,
sem vínculos ou patrocinado por civilização existente; ou assumir civilização controlada por bot.
Assumir bot herda estado, `Ledger` e Crônica; o Mandato vira preset inicial editável e a Doutrina vira
sugestões. Só civilizações sem humano podem ser assumidas (GDD 10 — decidido).

Os contratos exatos abaixo são **proposta** para tornar essas decisões reproduzíveis:

```text
EntryChoice =
  | FoundCommunity { sponsorship_civilization_id?: CivilizationId }
  | TakeOverBot { target_civilization_id: CivilizationId }

ClaimCivilization { user_id, choice, turn, ruleset_ref }
SelectSuccessor { user_id, former_civilization_id, successor_civilization_id, turn, ruleset_ref }
```

Pré-condições propostas para `ClaimCivilization`: conta autorizada e sem outra civilização humana no
mundo; capacidade de entrada disponível; para `TakeOverBot`, alvo existente controlado por bot e sem
humano. Fundação requer tile/local legal e, se houver patrocínio, patrocinador elegível. Parâmetros de
proteção, distância, custos e obrigações do patrocínio seguem catálogo e ainda são proposta.
Pré-condições para `SelectSuccessor`: colapso resolvido, sucessora na lista elegível emitida pelo
motor e ator autorizado a escolher. Enquanto a escolha humana estiver pendente, o Governador não
escolhe sucessor, conforme GDD 09 — decidido.

Transições propostas e determinísticas: comando válido de fundação cria uma comunidade com estado
inicial definido por catálogo; tomada de bot troca somente o controlador, mantendo estado, Ledger e
Crônica, e prepara preset editável do Mandato e sugestões da Doutrina; sucessão atribui cada cidade,
tile, legado e relação segundo as regras acima, efetiva a escolha, converte comunidades restantes em
bots e transfere o controle humano sem intervalo com duas civilizações. Valores iniciais, empates de
atribuição, identidade dos comandos e sequência exata dos eventos continuam proposta até fixação no
catálogo/decisão do dono. Repetição idempotente do mesmo comando não pode duplicar a transição.

## 7. Falhas, fallback e determinismo

Validação rejeita antes da aceitação comando com proprietário, pré-requisito, recurso, rota, alvo,
Mandato ou `GroundingRef` inválido. Rejeição não cobra custo e não entra em `AcceptedCommand`.
Conflito posterior inevitável (por exemplo, movimento já ocupado) é resolvido na fase canônica e
emite evento causal; nunca consulta relógio, rede, mapa de hash ou IA.

Timeout, schema inválido ou grounding insuficiente de IA recebe fallback T0 fora do `step`; a opção
escolhida é validada e gravada como `AcceptedCommand` com `origin: fallback`. O log contém comandos
aceitos, `accepted_sequence`, `RulesetRef`, seed/versionamento necessário, `DomainEvent`, causas de
falha e `StateHash`. `IntentEvidence` pode ser retida para auditoria, mas não decide replay.

Nunca entram no `step`: tempo de parede, I/O, consultas a banco, rede, credenciais, texto livre de
jogador, prompt/resposta de modelo, custo de API, presença de sessão, ordem incidental de coleção
ou nova chamada de IA. Um `WorldSnapshot` periódico é armazenado pelo subsistema de persistência;
PostgreSQL é a implementação desse armazenamento conforme ADR-0007.

## 8. Orçamento

Metas propostas para a Fase 2, medidas no harness e não requisitos já aceitos:

| Recurso | Meta | Degradação |
|---|---:|---|
| Resolução de domínio | até 40% do orçamento de CPU do turno headless | registrar perfil e reduzir cenário de teste, nunca simplificar regra em silêncio |
| Memória transitória | O(entidades + contratos + confrontos), sem cópia integral por fase | processar vetores ordenados/stream de eventos |
| IA de decisão/narrativa | 0 dentro de `step` | T0 determinístico; token e custo ficam na orquestração |
| IA fora do motor | teto por chamada e por jogador definidos pelos SDDs 04/05/06 | timeout/erro vira fallback e não bloqueia o turno |

A meta numérica absoluta de milissegundos, tamanho de mundo e orçamento de IA está aberta. O CI
deve guardar baseline de tempo, alocações e número de eventos para detectar regressões mensuráveis.

## 9. Estratégia de testes

- Determinismo: executar o mesmo replay/seed em ambientes distintos e comparar cada `StateHash`.
- Propriedades: gerar sequências válidas e inválidas; provar invariantes de estoque, grupos, limites,
  contratos proporcionais, práticas e reserva de unidade.
- Fixtures gravadas: entradas `AcceptedCommand`, `WorldSnapshot`, `RulesetRef` e eventos esperados;
  CI não chama LLM nem API real.
- Casos de fronteira: saldo zero, contrato só com riqueza, capacidade parcial, fome/importação no
  mesmo turno, rota interrompida, duas ordens concorrentes, perdas simultâneas e população zero.
- Metamórficos: reordenar coleções físicas sem mudar IDs não muda `StateHash`; acrescentar comando
  rejeitado não muda estado; replay de snapshot + log restante iguala replay desde o início.
- Harness: partidas longas de bots T0 medem crises, migração, inadimplência, frequência de combate,
  distribuição de `P` e custo por turno; pesos provisórios só mudam por `RulesetRef` versionado.

## 10. Perguntas abertas

1. Qual proporção deve ser usada quando pacote misto contém apenas riqueza de um lado, e como
   atribuir uma entrega parcial causada por ambos? **Recomendação:** aprovar `r = min(r_lados)` e
   registrar inadimplência apenas quando a capacidade da parte, isoladamente, impedir `r = 1`.
2. Quais fórmulas, tetos e efeitos de catálogo definem crescimento, migração, produção e combate?
   **Recomendação:** iniciar com parâmetros declarativos pequenos e calibrá-los no harness antes de
   congelar qualquer número de balanceamento.
3. Quais limiares separam revolução moderada de severa? **Recomendação:** decidir depois de medir
   distribuição de `P`, preservando desde já os dois efeitos de Mandato já decididos.
4. Quando uma proposta de combate vira confronto, e quais posturas existem? **Recomendação:** usar
   catálogo fechado de posturas e declarar todos os custos/reservas antes da fase de combate.
5. Quais metas de CPU, memória, tamanho de mundo e custo de IA devem bloquear regressões? 
   **Recomendação:** estabelecer baselines após o primeiro harness headless, antes de fixar SLAs.

## ADRs relacionados

- ADR-0006 — motor determinístico, event sourcing e IA somente como proponente.
- ADR-0008 — turno sem relógio e fechamento por humanos presentes.
- ADR-0007 — stack tecnológica aceita.
