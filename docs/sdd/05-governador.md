# SDD 05 — Governador

> **Status:** Proposta. Depende da aprovação do GDD e não aprova stack nem números de balanceamento.
> Este documento define contratos e invariantes; qualquer adaptação tecnológica deve preservá-los.

## Objetivo e referências

O Governador é o bot de uma civilização. Ele transforma o Mandato configurado pelo jogador
em intenções limitadas, justificáveis e, depois da validação do motor, em comandos do turno.
Ele roda no servidor autoritativo (ADR-0002), mas não é parte do núcleo determinístico.

Referências: ADR-0002 (Governador no servidor), ADR-0006 (motor determinístico e event
sourcing), ADR-0008 (turno sem relógio); GDD 01, 07, 09 e variáveis compartilhadas do GDD 12.

## Responsabilidades e fronteiras

### Responsabilidades

- Carregar a versão do Mandato válida para o turno e produzir decisões nas áreas permitidas.
- Montar uma lista de ações candidatas, filtrá-la por escopo, linhas vermelhas, reservas,
  confirmação, pré-condições e sobrevivência; calcular e ordenar sua utilidade `U`.
- Usar T0 como decisão universal e, quando disponível e dentro do orçamento, T1 para escolher
  entre opções tipadas. T2 pode redigir explicações, nunca decidir efeito mecânico.
- Submeter uma intenção com grounding; converter somente a intenção validada em comando.
- Emitir relatório causal de retorno: **fiz**, **não fiz** e **precisa de você**.
- Sugerir mudanças no Mandato como recomendação separada; a confirmação é exclusivamente humana.

### Não responsabilidades

- Não executa efeitos, não altera estado, não aceita tratados, não resolve combate e não calcula
  transições diplomáticas. Isso pertence ao motor e seus catálogos.
- Não define elegibilidade ou parametrização da Entropia; apenas escolhe respostas permitidas a
  efeitos que já estão no estado.
- Não substitui decisões irreversíveis nem confirma uma intenção marcada como `propose_wait`.
- Não é fonte da verdade para Mandato, recursos, relações, memória ou Crônica; esses dados são
  derivados do estado e do log.
- Não interpreta texto do jogador como instrução. Texto livre é dado não confiável e só pode chegar
  ao motor pela mesma intenção tipada e validada das demais entradas.

## Fluxo proposto por turno

1. O servidor fixa `mandate_version` e obtém o estado publicado e as ações candidatas válidas.
2. O filtro do Mandato elimina ações proibidas. O motor calcula atributos e `U` em inteiros.
3. T0 ordena e escolhe; T1, se usado, escolhe somente entre os IDs já filtrados. A escolha T1 é
   revalidada como qualquer outra. T2, se usado, apenas redige justificativa baseada nos fatos.
4. A validação final gera um `GovernorCommand` ou uma recusa/inação explicável.
5. O comando aceito entra na ordem de aceitação do turno. O `step` o aplica posteriormente.
6. O Governador recebe o resultado e produz cartões de relatório fora do `step`.

Bots e Governadores recebem opções no momento determinado pela ordem rotativa derivada da seed do
mundo. A revalidação na aceitação é obrigatória: uma ação anterior pode ter removido sua
elegibilidade. A rotação e o desempate são reprodutíveis, sem prioridade fixa de civilização.

## Contratos propostos

Os esquemas abaixo são contratos lógicos. Serialização, transporte e persistência ficam em aberto;
Seus adaptadores não podem mudar semântica, escala ou ordenação, conforme a stack aceita no ADR-0007.

```text
type Scope = cities_economy | exploration_defense | technology | diplomacy | crisis_response
type ScopeMode = act_within_limits | propose_wait
type Stance = conciliatory | reciprocal | deterrent
type ReserveLevel = zero | low | medium | high
type RedLine = no_start_war | no_break_treaty | no_cede_city
             | no_spend_reserve_diplomacy | no_relocate_population

type Mandate = {
  id: MandateId, version: u32, effective_turn: Turn,
  direction: { security: u8, sustenance: u8, development: u8, relations: u8 },
  red_lines: Set<RedLine>,
  reserves: { treasury: ReserveLevel, strategic_stock: ReserveLevel,
              unit_loss: ReserveLevel },
  stance: Stance,
  scopes: Map<Scope, ScopeMode>,
  alerts: Set<AlertRule>
}
```

`direction` soma 100 e usa incrementos de 10. A lista de `RedLine` só pode crescer por categoria
validada de template. Os cinco escopos e os níveis de reserva são propostas iniciais, sujeitas a
balanceamento, até aprovação do GDD.

```text
type CandidateAction = {
  id: ActionId, scope: Scope, template_id: TemplateId,
  irreversible: bool, requires_confirmation: bool,
  benefits: { security: 0..100, sustenance: 0..100,
              development: 0..100, relations: 0..100 },
  opportunity_cost: 0..100, exposed_risk: 0..100,
  costs: ResourceDelta[], facts: FactRef[], ledger_refs: LedgerRef[]
}

type GovernorIntent = {
  civil_id: CivilId, turn: Turn, mandate_version: u32,
  action_id: ActionId | null, source: t0 | t1,
  fact_refs: FactRef[], ledger_refs: LedgerRef[], rationale_code: RationaleCode
}

type MandateRecommendation = {
  base_version: u32, patch: MandatePatch, fact_refs: FactRef[],
  explanation: string
}
```

`FactRef` aponta para fato do estado (cidade, estoque, unidade, evento ou prazo), e `LedgerRef`
para fato diplomático. Ambos são IDs estáveis, não texto livre. `explanation` é conteúdo auxiliar,
não tem efeito e pode ser produzido fora do caminho determinístico.

```text
filter(mandate, action, state, presence) -> Allow | Deny(reason) | Await(reason)
score(mandate, action) -> i32
choose(filtered_actions, decision_port?) -> GovernorIntent | NoAction
validate(intent, state, mandate) -> GovernorCommand | Rejection
report(results, state) -> GovernorReport
```

`DecisionPort` (T1) recebe apenas `CandidateAction` já filtradas e devolve `action_id` e,
opcionalmente, score/probabilidade. Sua resposta não pode criar ação, mudar campos ou escolher ID
fora da lista. A porta é substituível; T0 não depende dela.

## Mandato, presets e regra de ausência

Os presets visíveis decididos são **Equilibrado**, **Recuperar** e **Crescer com cautela**. Cada um
deve expor rumo, linhas vermelhas, reservas e escopos antes da confirmação. Os valores iniciais
propostos para o rumo são, respectivamente, `25/25/25/25`, `30/40/10/20` e `20/30/35/15`, na ordem
segurança/sustento/desenvolvimento/relações; seguem sujeitos a balanceamento.

Com jogador presente, apenas escopos ligados em `act_within_limits` viram candidatos. Em
`propose_wait`, a intenção permanece no painel sem efeito. Com jogador ausente, o Governador atua
em **todas** as áreas dentro do Mandato, inclusive as antes não delegadas, mas suspende sempre:

- iniciar guerra;
- romper tratado;
- ceder cidade;
- aceitar vassalagem;
- escolher comunidade sucessora.

Essa é a lista decidida de irreversíveis. Ações com custo, reserva ou regra de sobrevivência ainda
devem respeitar todos os demais filtros. A ausência não interrompe a civilização: a inação também é
uma escolha registrada quando não existe ação permitida.

## Modelo de dados e invariantes verificáveis

| Elemento | Fonte canônica | Invariante |
|---|---|---|
| Mandato | comando confirmado e log | `version` é imutável; alteração vale só em turno futuro |
| Preset | catálogo versionado | distribuição soma 100; detalhes são visíveis antes de aplicar |
| Reserva | estado no início do turno + Mandato | gasto não excede parcela livre nem guarda de sobrevivência |
| Intenção | entrada externa gravada | cita Mandato e fatos existentes; não muda estado |
| Comando | log de comandos aceitos | deriva de intenção validada ou T0 e contém `mandate_version` |
| Relatório | derivado de resultados/log | não cria comando nem altera Mandato |

Invariantes adicionais:

- Uma ação irreversível nunca é emitida pelo Governador para jogador ausente, nem com escopo ligado.
- Linhas vermelhas vencem postura, pontuação, recomendação T1/T2 e fallback.
- Reservas mapeiam, como valor inicial, para 0%, 10%, 25% e 40% do disponível no começo do turno;
  recursos prometidos ou essenciais à sobrevivência não pertencem ao disponível livre.
- Toda intenção aceita tem grounding suficiente para auditoria; intenção sem fatos é descartada.
- O Governador não modifica o Mandato; `MandateRecommendation` não é `GovernorCommand`.
- Todo empate de `U` usa `ActionId` estável. Nenhuma iteração de mapa, relógio ou resposta externa
  pode definir a decisão aplicada.

## Utilidade, postura e diplomacia

Para ação já permitida, o motor calcula em inteiro:

```text
U = 3*Bseg + 3*Bsus + 2*Bdes + 2*Brel - 2*Cop - X
```

Cada benefício é normalizado em `0..100` pelas prioridades do rumo; `Cop` e `X` também ficam em
`0..100`. O Governador considera somente ações com benefício acima de 10, escolhe maior `U` e usa
`ActionId` no empate. Pesos, faixa e limiar são valores iniciais de balanceamento. `U` não é
coesão `C`, privação `D`, estabilidade `S` nem pressão `P`; esses símbolos preservam as definições
do GDD 12.

A postura só filtra propostas diplomáticas permitidas: conciliatória admite ajuda e tratados;
recíproca exige benefício equivalente ou dívida reconhecida; dissuasória prioriza reforço, aviso e
garantias. Ela não altera a máquina de estados, aceitação, custos ou Ledger. Para proposta entre
civilizações automáticas, o motor calcula `A` e suas componentes conforme GDD 07; T1 pode escolher
entre propostas já válidas, jamais aceitar uma proposta pelo texto.

## Falhas, fallback e determinismo

| Condição | Tratamento proposto |
|---|---|
| T1 indisponível, timeout ou saída inválida | T0 escolhe a maior `U` filtrada; não segura o turno |
| grounding ausente/IDs inválidos | descartar intenção e usar T0 ou `NoAction` |
| ação perdeu elegibilidade | rejeitar na aceitação, sem custo, com motivo enumerado |
| não há ação permitida | emitir `NoAction` causal; a simulação continua |
| teto de IA atingido | desabilitar T1/T2 para a civilização no período e usar T0 |
| jogador retorna | exibir relatório acumulado e intenções `propose_wait`; não executar pendências |

Entram no log: presença/ausência que acionou o Governador, versão e ID do Mandato, candidatos
considerados ou sua hash versionada, intenção externa bruta validável, decisão T0/T1, motivo de
rejeição/fallback, comando aceito, `NoAction`, referências de fatos, versões de catálogo e hash de
estado por turno. Segredos, chaves, prompts completos, texto não confiável sem isolamento, hora de
parede, latência, ordem interna de coleções, chamadas de rede e resposta não validada **nunca**
entram no `step`. Metadados operacionais podem existir fora do replay, sem influenciar resultado.

## Orçamento proposto

Os limites numéricos abaixo devem ser medidos no harness e aprovados antes de produção:

- T0: orçamento-alvo de até 5 ms por civilização/turno e memória transitória limitada à lista de
  candidatos e fatos referenciados; ultrapassagem é regressão mensurável.
- T1: no máximo uma escolha tipada por civilização/turno, somente se houver mais de uma candidata
  elegível; teto de tokens/custo por chamada e por jogador/dia é configuração auditável, ainda a
  definir com a política de chaves do ADR-0002.
- T2: opcional, assíncrono e fora do fechamento do turno; tem teto separado de tokens/custo e sua
  ausência reduz explicação, não qualidade mecânica.
- Contexto: somente estado resumido, Mandato e fatos selecionados. Prefixo estável permite cache;
  contexto saturado é reconstruído de documentos canônicos, nunca vira fonte de verdade.

A instrumentação deve registrar tempo, memória, tokens de entrada/saída,
custo estimado, provedor e motivo de fallback fora do `step`, sem registrar segredos.

## Estratégia de testes

- Determinismo: mesmo estado, seed, Mandato versionado e log de comandos produzem o mesmo hash em
  máquinas distintas; replay não chama T1/T2.
- Propriedades: soma do rumo é 100; nenhum gasto excede reserva/guarda; irreversível é bloqueado na
  ausência; linhas vermelhas são absolutas; toda intenção aceita tem grounding; empate usa ID.
- Fixtures gravadas: respostas T1 válidas, inválidas, timeout e IDs fora da lista. CI reproduz as
  fixtures e nunca chama API real.
- Golden replays: ausência longa, retorno com relatório, disputa de ações, proposta diplomática,
  crise e revolução. Cada fixture verifica comandos, motivos e hash final.
- Harness longo: mede frequência de `NoAction`, uso de reserva, bloqueios, tempo T0, tokens/custo e
  distribuição de primeira aceitação por civilização; falha mensurável de orçamento reprova CI.
- Contrato de relatório: cada cartão tem ação/intenção, custo, efeito, regra de Mandato e dois fatos;
  recomendações não produzem comando até confirmação humana gravada.

## Relatório, avisos e revolução

O relatório tem no máximo três cartões: **fiz**, **não fiz**, **precisa de você**. Cada cartão liga
resultado ao comando/template, custo, efeito e fatos do estado ou Ledger. Avisos são gatilhos
determinísticos: confirmação requerida, guerra contra a civilização, `P >= 40`, `C < 30`, reserva
prestes a uso e proposta que expira no próximo turno. Limiar e apresentação são valores iniciais.

Em revolução moderada, o Mandato proposto do novo regime é reversível; em revolução severa, o
Mandato do novo regime é imposto, conforme decisão do GDD 06. Intensidades intermediárias, catálogo
e campos do Mandato do novo regime permanecem em aberto. Nada é aplicado por narrativa ou T1.

## Perguntas abertas para o usuário

1. Quem fornece a chave/custo de T1: operador ou jogador? **Recomendação:** operador, com teto por
   civilização e fallback T0; reduz fricção e conserva BYOK apenas para usos T2 opcionais.
2. Quais ações entram na tabela de intensidade de revolução e quais intensidades existem?
   **Recomendação:** três faixas (baixa/média/alta) em catálogo versionado, com transição revisável.
3. Os valores iniciais de presets, reservas e avisos devem ser aceitos como baseline do harness?
   **Recomendação:** sim, tratá-los explicitamente como parâmetros de dados, não constantes de código.
4. O relatório acumulado de ausência longa deve agrupar por turno, por crise ou por tema?
   **Recomendação:** por tema causal, com links para os turnos, para manter a volta legível no celular.
