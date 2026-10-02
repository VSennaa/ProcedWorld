# SDD 03 — Turnos e sessão

> **Status: proposta.** Este desenho depende do GDD ainda não aprovado formalmente.
> Contratos e invariantes são independentes de stack; detalhes de implementação só valem se
> o ADR-0007 for aceito.

## Escopo e fronteiras

O subsistema coordena a sessão de um mundo e a passagem de um turno: determina quem está
presente, recebe e ordena comandos, registra `Pronto`, aciona o Governador para ausentes,
fecha o turno e orquestra as fases de resolução. O servidor é autoritativo.

Ele **não** decide regras de economia, combate, cidades, diplomacia, Entropia ou Mandato. Essas
regras recebem comandos e estado pelo núcleo de simulação. Também não interpreta texto livre,
não chama provedores de IA dentro de `step`, não persiste segredos, não cria notificações de
produto nem define a interface mobile. Os subsistemas de IA produzem intenções; este apenas
agenda, valida o envelope e grava o resultado aceito ou seu fallback.

### Decidido

- Todos participam do mesmo turno; comandos aceitos são aplicados sequencialmente na ordem de
  aceitação do servidor (ADR-0003).
- Não há relógio de turno. O turno avança quando todos os humanos presentes marcaram `Pronto`;
  prazos internos contam turnos (ADR-0008).
- Ausente não é esperado: o Governador joga no turno, dentro do Mandato. Ações irreversíveis
  ficam suspensas até o retorno do jogador (GDD 09).
- Bots e Governadores têm ordem e momento de envio derivados da seed do mundo, com rotação entre
  turnos (GDD 01).
- Ataques reservam unidade e custo na entrada; dano e perdas simultâneas são resolvidos depois,
  em fase própria (GDD 01).

## Conceitos e ciclo de vida da sessão

### Proposta: presença

Uma `WorldSession` é a associação autenticada de um humano a um mundo e a uma civilização. Ela
tem `session_id` aleatório, `player_id`, `world_id`, `civilization_id`, `opened_at_turn` e estado
`connected | grace | absent | closed`.

- **Presente**: há uma sessão `connected` do humano para aquele mundo, autenticada e com
  batimento válido para o turno aberto. Uma única sessão por civilização é elegível para votar
  `Pronto`; sessões adicionais são somente leitura ou substituem explicitamente a anterior.
- **Queda**: falha de transporte não muda comandos já aceitos. A sessão entra em `grace` durante
  uma janela operacional curta, exclusivamente para reconexão; a janela não é prazo de turno e
  não aparece no estado do mundo.
- **Ausente**: ao terminar a janela de reconexão, ou ao receber encerramento explícito da sessão,
  a civilização deixa o conjunto de humanos esperados daquele turno. O Governador completa o que
  faltar. Se o jogador reconectar depois, volta como presente somente no próximo turno aberto.
- **Sem sessão ao abrir**: o humano é ausente desde a abertura e o Governador é escalonado de
  imediato. Não se espera reconexão para fechar o turno.

A duração concreta da janela de reconexão, formato do batimento e política para múltiplos
dispositivos continuam abertos. Eles são metadados operacionais, nunca entradas de `step`.

### Estados do turno

```text
Preparing -> Open -> Collecting -> Closing -> Resolving -> Published
                    ^               |                         |
                    +---------------+-------------------------+
```

- `Preparing`: seleciona participantes presentes e agenda atores automatizados.
- `Open`/`Collecting`: aceita comandos válidos e alterações de prontidão.
- `Closing`: conjunto esperado está pronto e todos os atores automatizados enviaram ou receberam
  fallback. Não aceita novas ações humanas.
- `Resolving`: executa as fases canônicas uma vez.
- `Published`: publica novo estado, relatório e hash; abre o próximo turno apenas por uma ação
  determinística do servidor, não pelo relógio.

## Contratos

Os tipos abaixo são esquemas de domínio, não uma escolha de linguagem ou transporte.

```text
type TurnNumber = u64
type CommandSequence = u64
type WorldSeed = bytes

TurnContext {
  world_id, turn: TurnNumber, catalog_version, engine_version,
  seed_version, state_hash_before, phase
}

AcceptedCommand {
  command_id, sequence: CommandSequence, turn, civilization_id,
  actor: Human | Governor | Bot | Entropy,
  origin: PlayerInput | GovernorFallback | BotPlan | EntropyTemplate,
  payload: DomainCommand, accepted_against_hash, received_order
}

CommandReceipt { command_id, status: Accepted | Rejected, sequence?, reason_code? }
ReadyState { civilization_id, turn, ready: bool, changed_sequence }
```

```text
open_turn(context) -> OpenTurnRecord
submit_command(session, command, expected_turn) -> CommandReceipt
set_ready(session, turn, ready) -> ReadyState
mark_absent(civilization, turn, cause) -> AbsenceRecord
submit_automated_intent(actor, intent) -> CommandReceipt
close_if_eligible(turn) -> CloseTurnRecord | NotEligible
resolve_turn(close_record) -> ResolutionRecord
publish_turn(resolution) -> PublishedTurn
```

`submit_command` valida autenticação, pertencimento à civilização, turno e fase atuais, schema e
pré-condições contra o estado autoritativo. A aceitação atribui `sequence` monotonicamente; lotes
são desmembrados e não recebem prioridade. Um comando rejeitado não entra no log de comandos
aceitos nem consome recurso. `set_ready(false)` é permitido apenas antes de `Closing`; não remove
comandos previamente aceitos. `MaintainPlan` é um `DomainCommand` explícito para quem não quiser
intervir.

```text
function eligible_to_close(turn):
  return every expected_human(turn) is ready
     and every scheduled_automated_civilization(turn) is submitted_or_fallback

function accept(command):
  if !valid_now(command): return Rejected(reason_code)
  seq = append_command_log(command)
  return Accepted(seq)
```

### Proposta: ordem automatizada

Para cada turno, derive uma permutação de `civilization_id` elegíveis por PRNG versionado, com
entrada `(world_seed, seed_version, turn, "automated-submit")`. A rotação é consequência dessa
permutação, não da ordem de conexão, de threads ou de um mapa em memória. Cada ator envia apenas
quando seu ponto na agenda chega; falha ou demora até o limite operacional produz T0 e libera o
próximo ator. O comando resultante entra na mesma fila global de aceitação.

## Modelo de dados e log

```text
TurnRecord {
  world_id, turn, status, context, expected_humans: Set<CivilizationId>,
  ready: Map<CivilizationId, ReadyState>, automated_schedule: [CivilizationId],
  command_log_ref, opened_event_id, closed_event_id?, state_hash_after?
}

SessionPresence {
  session_id, world_id, civilization_id, state, observed_at, effective_turn,
  absence_cause?: Disconnect | ExplicitLeave | NoSessionAtOpen
}

ResolutionRecord {
  turn, phases: [PhaseResult], state_hash_before, state_hash_after,
  engine_version, catalog_version, seed_version
}
```

O log de eventos inclui, em ordem: `OpenTurn`, mudanças de presença efetivas, comandos aceitos,
mudanças de `Pronto`, intenção/fallback automatizado, `CloseTurn`, resultado de cada fase,
`PublishTurn` e hashes. Para decisão automatizada, registra a intenção tipada validada, a origem,
versão da política/fallback, referência a fixture quando houver, justificativas estruturadas e o
comando final; texto narrativo é referência separada e não comando.

Se ADR-0007 for aceito, `TurnRecord` e o log podem ser persistidos no PostgreSQL, e conexões em
um serviço Rust. A transação que atribui `sequence` deve serializar concorrência por `world_id`.
Isso não altera o contrato: outro armazenamento precisa preservar a mesma ordem observável.

## Invariantes verificáveis

1. Para um `world_id` há no máximo um turno em `Open`, `Collecting`, `Closing` ou `Resolving`.
2. `sequence` é única, crescente por mundo e define integralmente a ordem de comandos aceitos.
3. Cada comando aceito pertence ao turno aberto e só pode ser aplicado uma vez.
4. O conjunto `expected_humans` é congelado na abertura, exceto remoção por transição efetiva para
   `absent`; reconexão não o readiciona no turno corrente.
5. O turno só fecha se cada humano ainda esperado está pronto e cada ator automatizado agendado tem
   comando aceito, `MaintainPlan` ou fallback registrado.
6. `Pronto` não apaga comandos e `Pronto=false` nunca é aceito depois de iniciar `Closing`.
7. Governador de ausente não emite ação irreversível; o bloqueio é feito na validação do motor,
   mesmo que a intenção diga o contrário.
8. Cada fase ocorre no máximo uma vez por turno e na ordem canônica abaixo.
9. Mesmo estado inicial, seed/versionamento e log de comandos aceitos produzem o mesmo hash final.
10. Hora, conexão, latência, ordem de thread, API de IA e texto não estruturado não alteram o
    resultado de `step`.

## Fases de resolução

| Ordem | Fase | Entrada e resultado |
|---:|---|---|
| 1 | Abertura | Publica estado do turno e congela presentes/agendamento. |
| 2 | Entrada | Valida e aceita ações sequenciais pela `sequence`; ataques só reservam custo/unidade. |
| 3 | Fechamento | Confirma `Pronto`, ausência e comandos automatizados/fallback. |
| 4 | Conflitos | Resolve ataques declarados, perdas simultâneas e efeitos pendentes em ordem canônica. |
| 5 | Sustento | Aplica manutenção, produção, consumo, crescimento e coesão. |
| 6 | Entropia | Aplica apenas template elegível e validado ao estado pós-sustento. |
| 7 | Síntese | Calcula marcos, consolida memória e produz os dados para Crônica/relatório. |
| 8 | Publicação | Calcula hash, grava snapshot quando aplicável e expõe o novo estado. |

As fases 4–8 são parte de `step` ou de funções puras chamadas por ele. A ordenação interna usa IDs
estáveis; empates usam PRNG explícito e versionado.

## Falhas, fallbacks e determinismo

Falhas de rede só afetam presença operacional. Comandos com recibo `Accepted` permanecem. Pedido
repetido usa `command_id` idempotente e retorna o mesmo recibo. Uma falha ao publicar permite
repetir publicação do mesmo `ResolutionRecord`, sem resolver de novo.

Uma intenção de bot/Governador inválida, com erro, timeout ou excedendo orçamento é substituída por
T0 determinístico; a falha e a identidade da política fallback vão ao log. Uma intenção permitida
mas irreversível para ausente vira `MaintainPlan`/ação segura determinística e um item de relatório.
Se a Entropia não tiver template válido, não cria evento mecânico naquele turno.

Entra em `step`: estado canônico, lista ordenada de `AcceptedCommand`, seed e versão, versões de
motor e catálogos, e parâmetros mecânicos já validados de eventos. Nunca entra: hora de parede,
timeout, batimentos, estado de socket, identificador de sessão, ordem de chegada de pacote antes
da aceitação, custos/tokens, prompt, resposta bruta de IA, chave, IP/hostname, texto livre ou
resultado de rede. Esses itens podem estar em observabilidade protegida, mas não no replay.

## Orçamentos propostos

Os valores abaixo são limites iniciais a medir e aprovar; não são decisão de design.

| Recurso | Proposta de limite | Ao exceder |
|---|---:|---|
| Aceitação de comando | 100 ms de CPU por comando | Rejeitar com `server_busy`; não altera estado. |
| Resolução de turno | 2 s de CPU por mundo | Interromper antes de publicar, diagnosticar; nunca pular fase. |
| Memória transitória | 64 MiB por resolução | Falhar antes de commit e permitir retomada do mesmo log. |
| T1 por ator automatizado | 1 chamada, 1.000 tokens totais | T0 determinístico. |
| T2 por ator/turno | 0 por padrão; só quando contrato pedir | T0/T1 conforme política e teto. |
| Custo IA | teto diário do jogador/operador aplicável | Não chamar provedor; registrar `BudgetFallback`. |

Tokens, preço e latência pertencem ao orçamento da camada de IA, não influenciam `step`. O prefixo
de prompt permanece estável para cache quando houver chamada, conforme a constituição do projeto.

## Estratégia de testes

- **Determinismo:** executar o mesmo replay em processos/máquinas distintas e comparar hash por
  turno, log de comandos e resultado de fases.
- **Propriedades:** gerar sequências de presença, reconexão, `Pronto`, duplicatas e comandos em
  paralelo; verificar os dez invariantes, idempotência e ausência de dupla resolução.
- **Golden replays:** fixtures gravadas cobrem conflito de movimento, ataque simultâneo, queda após
  comando aceito, ausência na abertura, retorno no turno seguinte e falha de IA.
- **IA record/replay:** CI usa intenções e respostas T1/T2 gravadas; nunca chama API real. Testar
  schema inválido, timeout, orçamento e bloqueio de irreversível sob ausência.
- **Harness headless:** simular partidas longas com humanos sintéticos, bots e Governadores; medir
  hash, tempo, memória, distribuição da ordem automatizada e que nenhum mundo trava por ausência.
- **Concorrência:** testar múltiplos pedidos concorrentes e reenvios com o mesmo `command_id`;
  provar uma única `sequence` e um único efeito.

Se ADR-0007 for aceito, incluir teste de integração contra a transação de persistência escolhida,
mas manter os testes de domínio independentes de PostgreSQL, Rust e Godot.

## Perguntas abertas para o usuário

1. Qual duração e qual sinal encerram a janela de reconexão antes de declarar ausência?
   **Recomendação:** 30 segundos após perda de batimento, com reconexão explícita; é curto sem
   transformar o turno em prazo de jogo.
2. Um jogador com dois dispositivos pode manter ambos ativos no mesmo mundo?
   **Recomendação:** um dispositivo controlador por civilização; o segundo é leitura até assumir
   explicitamente, evitando dois votos e comandos concorrentes acidentais.
3. Uma sessão aberta mas ociosa deve contar como presente indefinidamente?
   **Recomendação:** não; exigir batimento válido e usar a mesma janela de reconexão para que uma
   aba abandonada não congele os demais presentes.
4. O limite proposto de 2 s de CPU por resolução é adequado ao tamanho inicial do mundo?
   **Recomendação:** adotá-lo apenas como gate de benchmark e calibrar com o harness antes da Fase 2.
5. O Governador ausente deve poder aceitar ações reversíveis de diplomacia já autorizadas pelo
   Mandato, além de `MaintainPlan`?
   **Recomendação:** sim, se o motor as classificar explicitamente como reversíveis e dentro do
   Mandato; manter qualquer efeito irreversível suspenso como já decidido.

## Referências

- ADR-0003 — Turnos simultâneos com resolução sequencial.
- ADR-0008 — Turno sem relógio.
- GDD 01 — Loop e turnos.
- GDD 09 — Governador e Mandato.
