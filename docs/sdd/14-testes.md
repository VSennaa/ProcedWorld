# SDD 14 — Estratégia de testes

> **Status:** Proposta. Depende da aprovação do GDD e da ratificação do ADR-0007.
> Este documento define contratos e evidências independentes de stack. Onde houver exemplo
> de Rust, Godot, PostgreSQL ou GitHub Actions, leia-se: **se ADR-0007 for aceito**.

## Objetivo

Demonstrar continuamente que o motor reproduz uma partida a partir da mesma entrada,
que regras não violam invariantes e que integrações de IA são seguras, auditáveis e
substituíveis. O subsistema produz evidências de qualidade; ele não decide regras,
balanceamento, diplomacia ou conteúdo do jogo.

## Responsabilidades e fronteiras

É responsabilidade deste subsistema:

- definir casos, dados de entrada, oráculos e critérios de aprovação para o motor, replays,
  contratos de IA e fluxos de simulação;
- conservar artefatos de referência revisáveis: golden replays, hashes esperados, fixtures
  gravadas e relatórios do harness;
- detectar divergência entre plataformas, regressões de orçamento e ações diplomáticas sem
  justificativa rastreável;
- classificar evidência como teste vermelho (contrato/invariante quebrado) ou aviso (meta ainda
  não ratificada ou métrica sem limiar aprovado).

Não é responsabilidade deste subsistema:

- introduzir mecânicas, alterar catálogos, escolher fórmulas de balanceamento ou corrigir um
  resultado para fazê-lo passar;
- chamar provedores reais de IA em CI, armazenar chaves, prompts completos de jogadores ou
  segredos;
- substituir validação do motor, o log de comandos, snapshots ou a auditoria operacional;
- tratar texto narrativo como oráculo de efeito mecânico.

## Premissas decididas

- O motor é puro: `state[n+1] = step(state[n], commands[n], seed)`, com seed explícita,
  versionada, aritmética inteira/ponto fixo e sem I/O, relógio ou IA no `step` (ADR-0006).
- Entradas externas tornam-se comandos gravados; snapshots periódicos e hash por turno detectam
  divergência. Replays não chamam IA novamente (ADR-0006).
- Turnos são simultâneos e as ações aceitas resolvem sequencialmente; a ordem dos bots e
  Governadores deriva da seed do mundo e roda entre turnos (GDD 01).
- Diplomacia usa máquina de estados e Ledger; o motor calcula aceitação e transições. A proposta
  deve ser justificável por fatos do estado e do histórico (GDD 07).
- Texto de jogador é dado não confiável: nunca é instrução nem concede efeito mecânico (GDD 07).

## Contratos e interfaces propostos

Os nomes abaixo são contratos lógicos, não compromisso com linguagem ou serialização.

```text
type ScenarioId = string
type Turn = uint
type StateHash = fixed-length lowercase hex

ReplayManifest {
  format_version, engine_version, ruleset_version, prng_version,
  scenario_id, world_seed, initial_state_hash, turns[]
}
ReplayTurn {
  turn, accepted_commands_in_order[], expected_state_hash,
  expected_snapshot_hash?: StateHash
}

RunResult {
  scenario_id, platform_id, completed_turns, hashes_by_turn[],
  budgets, findings[], artifact_refs[]
}
```

```text
run_replay(manifest, engine) -> RunResult
  requires: manifest e versões compatíveis; comandos já validados e ordenados
  ensures: para cada turno, compara hash calculado ao hash esperado

generate_property_case(seed, profile) -> InitialState + CommandSequence
check_invariants(state, previous_state, applied_commands) -> Finding[]
audit_diplomacy(state, command_log) -> Finding[]
```

`platform_id` identifica somente ambiente de execução relevante (por exemplo, sistema,
arquitetura e versão do motor); não contém usuário, hostname, IP, caminho local ou segredo.
O algoritmo canônico de hash, a codificação canônica do estado e as versões compatíveis são
**propostas a fechar no SDD 01**, antes da primeira golden fixture.

### Fixture de IA

```text
AiFixture {
  fixture_version, port_kind, request_contract_version,
  canonical_request, recorded_outcome, expected_disposition,
  redactions, source_note
}
recorded_outcome = success(typed_intent) | timeout | provider_error | malformed
expected_disposition = accepted_command | rejected(reason_code) | fallback(command)
```

O adaptador de teste reproduz `recorded_outcome` sem rede. `canonical_request` deve conter apenas
estado mínimo, IDs e dados sintéticos ou redigidos necessários para testar o contrato. O fixture
nunca contém API key, cabeçalho de autenticação, texto privado não redigido ou resposta que não
possa ser redistribuída. Alterar schema, regra de validação ou fallback exige revisar os fixtures
afetados, não regravá-los cegamente.

## Modelo de dados e invariantes verificáveis

### Artefatos

| Artefato | Fonte | Imutabilidade e uso |
|---|---|---|
| `golden replay` | cenário curado + comandos gravados | versionado; compara hash em cada turno |
| `property seed` | gerador determinístico | reproduz caso reduzido por seed e versão |
| `AI fixture` | captura saneada ou caso sintético | sem rede; testa porta, validação e fallback |
| `harness report` | simulação headless | série de hashes, orçamento e auditoria |
| `baseline` | execução aprovada | só altera por revisão explícita da mudança causal |

### Invariantes mínimos

- Mesmo `ReplayManifest` e mesma versão de regras produzem o mesmo `StateHash` por turno em todas
  as plataformas suportadas.
- O hash inclui todo estado mecânico canônico; texto narrativo, telemetria, tempo de execução e
  ordem incidental de memória não podem influenciá-lo.
- Todo comando do replay tem turno, ordem de aceitação, versão de regras e origem auditável; não
  há comando duplicado ou aplicado fora do turno registrado.
- Recursos, populações, contadores e limites permanecem em seus domínios definidos; sob hipótese
  de regra que proíbe saldo negativo, o teste prova que ele não fica negativo.
- Rejeitar intenção, timeout, resposta malformada ou indisponibilidade de IA não interrompe o
  avanço: resulta em motivo enumerado e fallback determinístico aplicável.
- Transição diplomática só ocorre por comando/regra válida. Toda ação diplomática automatizada cita
  fatos existentes do Ledger ou estado; a auditoria pode seguir cada referência.
- O texto livre, inclusive adversarial, não muda comando, autoridade, limites de Mandato ou estado
  sem passar pelo mesmo schema e validação de uma intenção comum.

## Falhas, fallbacks e determinismo

O executor de testes falha imediatamente em divergência de hash, crash, violação de invariante,
fixture de IA que use rede, ou ação diplomática sem prova requerida. Ao divergir, deve preservar
o primeiro turno divergente, hash esperado/calculado, versões, seed, sequência de comandos e um
snapshot canônico anterior, se existir.

Entram no log de diagnóstico: `scenario_id`, versões de motor/regras/PRNG/fixture, seed ou seu
identificador permitido, turno, índice do comando, tipo e ID do comando, hash esperado/calculado,
motivo enumerado de rejeição/fallback, métricas agregadas de tokens/custo e referências a
artefatos redigidos. Não entram: API keys, credenciais, IPs, hostnames, cabeçalhos, prompt bruto,
texto privado de jogador, cadeia de raciocínio do modelo ou dados pessoais.

Nada disso entra no `step`: logger, cronômetro, alocador, rede, leitura de fixture, gerador de
relatório, comparação de hash externa e chamada de IA. O `step` recebe apenas estado, comandos,
seed e versões/dados já canônicos, conforme ADR-0006. O fallback é produzido antes de o comando
ser entregue ao motor e também fica registrado como comando/origem auditável.

Uma divergência entre arquiteturas deve primeiro ser reproduzida pelo replay mínimo e então ser
classificada como: serialização/hash, PRNG, aritmética, ordem de iteração, versão de regras ou
erro do harness. A classificação é diagnóstico; nunca se mascara a diferença aceitando hash por
plataforma.

## Orçamento proposto

Os limites numéricos abaixo são metas iniciais, não decisões de produto. Devem ser medidos em
máquinas de referência e ratificados antes de virar gate vermelho.

| Medida | Proposta de coleta | Regra inicial |
|---|---|---|
| Tempo por turno | percentis do harness headless | aviso até haver teto aprovado |
| Memória | pico residente do processo por cenário | aviso até haver teto aprovado |
| 1.000 turnos / 8 bots | tempo total, crash e hash final | vermelho para crash/divergência; desempenho é aviso inicialmente |
| IA | tokens de entrada/saída, chamadas, custo estimado | sem chamada real em CI; excesso é aviso até teto aprovado |
| Fixture | tamanho e tempo de replay | vermelho se exigir rede ou expor dado proibido |

Se ADR-0007 for aceito, a CI pode registrar plataforma, duração e memória do binário Rust; GitHub
Actions é apenas um executor possível. O contrato permanece ser capaz de comparar ao menos duas
arquiteturas suportadas (proposta: x86_64 e ARM) antes da fase que exige a garantia entre máquinas.

## Pirâmide de testes proposta

1. **Unidade e schema:** validação de comandos, serialização canônica, conversões numéricas,
   catálogos e motivos de rejeição.
2. **Propriedades:** geradores determinísticos criam mapas, estados, comandos limítrofes e
   sequências de transições. Ao falhar, reduzem o caso e gravam seed, versões e sequência mínima.
3. **Golden replays:** cenários pequenos e históricos representativos com hash por turno, incluindo
   ordem rotativa de bots, combate, falhas de IA, Entropia por comando e transições diplomáticas.
4. **Contrato de IA:** fixtures de sucesso, timeout, erro, JSON/schema malformado, ID inexistente,
   referência de Ledger ausente e pedido fora do Mandato. CI nunca chama API/LLM real.
5. **Integração/harness:** mundo headless por 1.000 turnos com 8 bots; reexecuta o mesmo log e
   compara hashes, coleta orçamento e produz relatório de diplomacia.

### Golden replays entre máquinas e arquiteturas

Cada golden replay declara explicitamente `engine_version`, `ruleset_version`, `prng_version` e
formato. Uma mudança intencional de regra não sobrescreve golden existente: cria atualização
revisada, preserva a anterior para a versão antiga quando ela ainda for suportada, e explica a
causa. O gate cruza o mesmo manifesto em ambientes independentes e exige a sequência inteira de
hashes idêntica, não apenas o hash final.

O conjunto inicial proposto inclui: mundo mínimo; disputa de comandos na mesma janela; rotação de
bots; confronto com perdas simultâneas; comando inválido; fallback por timeout; proposta
diplomática aceita/recusada; e sucessão após colapso quando a regra estiver implementada. Itens
ainda sem regra aprovada são fixture futura, não teste que finja cobertura atual.

### Auditoria de coerência diplomática

O harness emite, para cada ação automática, uma linha estruturada com turno, emissor,
destinatário, ação, estado anterior/posterior, referências do Ledger, fatos de mundo citados,
resultado do validador e componentes mecânicos disponíveis. A auditoria reprova referência
ausente, ator incompatível, transição sem comando/regra válida, ou ausência de justificativa
exigida. Ela não julga estilo literário, simpatia do texto ou se a estratégia é "boa".

### Segurança e prompt injection

Casos adversariais usam texto sintético que tenta redefinir papel, solicitar segredo, burlar
Mandato, inventar IDs, exigir chamada externa, pedir alteração de regras ou misturar instruções
com termos de negociação. O oráculo verifica isolamento como dado, saída tipada, rejeição de
campos não permitidos e invariância mecânica quando não há intenção válida. Esses casos não usam
prompts reais nem registram o texto adversarial completo se ele puder carregar conteúdo sensível.

## Avisos versus testes vermelhos

É vermelho quando viola contrato aceito: determinismo, invariante, replay, segurança de fixture,
uso de rede em CI, crash ou justificativa diplomática obrigatória. É aviso quando mede uma meta
ainda não aprovada: percentil de tempo, pico de memória, custo estimado, cobertura, frequência de
fallback ou tendência de balanceamento. Um aviso deve ter dono, cenário, valor observado, baseline
e recomendação; não bloqueia a integração até que um ADR/GDD/SDD aprove seu limiar.

## Perguntas abertas para o usuário

1. Quais arquiteturas entram na promessa inicial de determinismo: apenas x86_64 e ARM, ou também
   versões específicas de sistemas operacionais? **Recomendação:** começar com x86_64 e ARM em CI,
   fixando versões de compilador/engine no manifesto de ambiente.
2. Qual codificação e algoritmo de hash serão canônicos para o estado? **Recomendação:** decidir no
   SDD 01 antes de criar o primeiro golden; hash deve derivar de serialização canônica versionada.
3. Quais limites de tempo, memória e custo passam de aviso a gate vermelho, e em qual hardware de
   referência? **Recomendação:** coletar baseline com o harness primeiro e ratificar metas por fase,
   sem inventar números agora.
4. Fixtures gravadas de IA podem incluir respostas de provedores obtidas manualmente, sob licença e
   revisão, ou apenas fixtures sintéticas? **Recomendação:** permitir ambas somente saneadas,
   versionadas e sem dados de usuário; priorizar sintéticas para casos de segurança.
5. Que profundidade de auditoria diplomática é obrigatória no MVP: somente presença de referências
   ou também a decomposição numérica da aceitação? **Recomendação:** exigir referências e motivo de
   transição desde o início; adicionar decomposição quando a fórmula de aceitação for ratificada.
