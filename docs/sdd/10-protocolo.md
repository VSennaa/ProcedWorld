# SDD 10 — Protocolo cliente-servidor

> **Status:** Proposta. Depende da aprovação do GDD e da ratificação do ADR-0007.
> Este documento define contratos e invariantes independentes de implementação. Menções a
> WebSocket, HTTP, Rust, Godot ou PostgreSQL são condicionais: aplicam-se **se o ADR-0007
> for aceito**.

## 1. Objetivo e fronteiras

O protocolo liga o cliente, inicialmente Android, ao servidor autoritativo (ADR-0001). Ele
autentica a sessão, publica uma visão permitida do mundo, recebe pedidos de comando e comunica
o resultado da validação e da resolução do turno.

É responsabilidade deste subsistema:

- negociar versão e capacidades do cliente;
- autenticar e renovar sessões, sem expor segredos;
- acompanhar presença no mundo e o estado `Pronto` do participante;
- entregar diffs ordenados por turno, chunks do mapa e notificações;
- limitar tamanho, frequência e forma dos payloads para rede móvel;
- preservar identificadores, correlação e metadados necessários para auditoria.

Não é responsabilidade deste subsistema:

- decidir regras, validar efeitos mecânicos, calcular combate ou executar `step`;
- definir a fórmula de `C`, `D`, `G`, `W`, `E`, `P` ou qualquer variável compartilhada;
- executar Governador, Entropia, LLM ou modelo de decisão;
- persistir o log, snapshots ou chaves (isso pertence aos subsistemas de persistência e segurança);
- renderizar mapa, selecionar UX, agendar notificações no sistema operacional ou dar modo offline
  jogável.

O servidor continua sendo a única fonte de verdade. O cache local serve para exibir e retomar a
sessão, mas não autoriza ação nem resolve turno.

## 2. Modelo de transporte proposto

Com ADR-0007 aceito, a proposta é HTTP para operações pontuais e WebSocket para a sessão
bidirecional. A semântica abaixo deve ser preservada se a implementação escolher outro transporte.

| Canal | Uso proposto | Não usar para |
|---|---|---|
| HTTP seguro | autenticação, renovação, bootstrap, download sob demanda de chunk e recuperação de estado | fluxo contínuo de turno |
| WebSocket seguro | presença, comandos, confirmações, diffs e notificações enquanto conectado | transmitir snapshot completo repetidamente |

O cliente envia `Hello` antes de qualquer mensagem de domínio. O servidor responde
`Welcome`, `UpgradeRequired` ou `Incompatible`. Uma desconexão não invalida comandos que já
receberam confirmação de aceitação; na reconexão o cliente pede recuperação a partir do último
`cursor` aplicado.

O protocolo não contém prazo de turno em horas. Prazos de tratados, crises, obras e ofertas são
expressos em `turn` (ADR-0008), nunca em relógio de parede.

## 3. Envelope, versão e tipos

Todo payload de aplicação usa um envelope versionado. A codificação proposta é JSON UTF-8; uma
codificação binária posterior só é compatível se preservar exatamente os campos e seus limites.

```text
Envelope {
  protocol_version: "1.0",
  type: MessageType,
  request_id: UUID,
  correlationId: UUID?,
  worldId: WorldId?,
  sessionId: SessionId?,
  cursor: Cursor?,
  payload: object
}
```

- `protocol_version` usa compatibilidade semântica: uma versão maior incompatível exige atualização;
  campos opcionais desconhecidos são ignorados; remover ou alterar semântica de campo exige versão
  maior incompatível.
- `request_id` permite deduplicação por sessão. Repetir o mesmo pedido idempotente retorna o mesmo
  resultado lógico, em vez de aplicar um comando duas vezes.
- `correlationId` liga resposta, erro ou confirmação ao pedido. Ele não substitui o identificador
  estável do comando no log.
- `cursor` é opaco para o cliente e ordena a visão publicada. Não é número de turno nem prova de
  que um comando foi aceito.

```text
Hello { supportedVersions: ["1.0"], capabilities: ["gzip", "chunk-cache"] }
Welcome { selectedVersion: "1.0", session: SessionView, limits: PayloadLimits }
UpgradeRequired { minimumVersion: "1.0", storeUrl: URL? }
Error { code: ErrorCode, safeMessage: string, correlationId: UUID? }
```

`storeUrl` é opcional pois a distribuição ainda não é decisão deste SDD. Mensagens de erro nunca
incluem stack trace, token, chave BYOK, conteúdo de cabeçalho ou estado de outra civilização.

## 4. Autenticação e autorização

O mecanismo concreto de credencial é aberto. A interface mínima proposta é:

```text
POST /session/login    LoginRequest -> SessionTokens
POST /session/refresh  RefreshRequest -> SessionTokens
POST /session/logout   AuthenticatedRequest -> Accepted
SessionTokens { accessToken, refreshToken?, expiresAt? }
```

O token de acesso identifica usuário e sessão; o servidor decide a associação `userId →
civilizationId` em cada mundo. Toda mensagem autenticada é autorizada pelo servidor para o
`worldId` e, quando aplicável, `civilizationId`. O cliente nunca informa esses campos como prova de
permissão.

BYOK não trafega por WebSocket, não aparece em `StateDiff`, `Notification` nem em logs de
protocolo. Seu cadastro e armazenamento pertencem ao SDD 11. O fato de a chave ser opcional é
decidido: sem ela a partida continua em T0; o protocolo só pode exibir estado de disponibilidade e
consumo já autorizado, sem revelar o segredo.

Proposta: tokens de curta duração, renovação revogável e limitação de tentativas de login. São
medidas de segurança a detalhar no SDD 11, não uma escolha de provedor de identidade.

## 5. Sessão, presença e `Pronto`

`Presence` é estado de sessão, não entrada do motor. Para operacionalizar ADR-0008, propõe-se:

- `present` é o humano com sessão autenticada e ativa no mundo, após `JoinWorld` confirmado;
- `absent` é quem não tem sessão ativa no mundo; o Governador joga por ele no turno;
- queda ou encerramento da conexão muda o jogador para `absent` depois de o servidor observar o
  desligamento; não se infere presença a partir do relógio do jogo;
- comandos aceitos antes da queda permanecem; os faltantes podem ser completados pelo Governador
  conforme Mandato. Esta última regra é proposta pendente de aprovação do GDD.

```text
JoinWorld { worldId, resumeCursor? } -> WorldBootstrap | StateRecovery
PresenceChanged { civilizationId, presence: "present" | "absent", turn }
ReadySet { turn, ready: true | false } -> ReadyConfirmed | CommandRejected
ReadySummary { turn, requiredHumanCivilizationIds, readyCivilizationIds }
```

`ReadySet(true)` encerra apenas a edição daquele humano no turno atual. `ReadySet(false)` é aceito
somente antes de `TurnClosed`; ele não remove comandos antes aceitos. Para não agir, o cliente
envia `KeepPlan` como comando explícito e então pode marcar `Pronto`.

Quando todos os humanos presentes estão prontos e as civilizações automatizadas enviaram suas
intenções, o servidor inicia o fechamento. A transição é única e idempotente: duas conexões do
mesmo usuário não podem fechar o turno duas vezes.

### 5.1 Unidades ociosas e `Pronto` (decidido em 2026-10-03)

`ready` de um humano com unidades ociosas (SDD 15 §4.3) é recusado com o erro
`units_awaiting_orders` e `detail` listando os IDs. A visão de estado inclui `idle_units` (IDs) e,
por unidade própria, `order` e `skipped_turn`, para o cliente montar a fila de atenção e desabilitar
o botão Pronto sem perguntar ao servidor. Novos comandos: `set_unit_order` e `skip_unit` via
`submit_command` (payload do motor `SetUnitOrder`/`SkipUnit`).

### 5.2 Catálogo para o cliente

Depois de `join`, o servidor envia `catalog` (sem `request_id`) com a árvore de tecnologia
(`id`, nome, custo, pré-requisitos), os tipos de unidade (`id`, nome, papel, movimento, força) e a
versão/hash do catálogo. O estado de pesquisa da própria civilização já vem em `civilization`.
O cliente nunca lê `data/` diretamente.

Correção de implementação: o envelope usa `protocol_version: "1.0"` (texto, §3) em ambos os lados.

## 6. Comandos, confirmação e resolução

O cliente envia intenção de ação em uma forma que o servidor valida contra estado autoritativo.
Somente a forma validada torna-se `Command` no log (ADR-0006).

```text
SubmitCommand {
  clientCommandId: UUID,
  expectedTurn: TurnNumber,
  kind: CommandKind,
  body: CommandBody
}

CommandAccepted {
  clientCommandId: UUID,
  commandId: CommandId,
  accepted_sequence: uint64,
  turn: TurnNumber
}
CommandRejected { clientCommandId: UUID, code: RejectionCode, safeReason: string }
TurnClosed { turn: TurnNumber, closingCommandCursor: Cursor }
TurnResolved { turn: TurnNumber, stateHash: Hash, nextTurn: TurnNumber }
```

`expectedTurn` impede aplicar silenciosamente uma ordem preparada em turno antigo. O servidor
responde `StaleTurn` com recuperação quando necessário. A ordem efetiva é `accepted_sequence`,
gravada com o comando; ordem de chegada ao rádio, renderização ou coleção do cliente não tem
efeito nas regras.

Ataque declarado reserva unidade e custo ao ser aceito; dano e perdas simultâneas pertencem à fase
de conflito do motor. Portanto `CommandAccepted` confirma a reserva, não uma vitória, ocupação ou
resultado de combate.

## 7. Visão de estado, diffs e mapa

O servidor publica somente uma `WorldView` autorizada para a civilização. Ela contém estado próprio,
informação pública e informação descoberta. Nunca contém seed secreta, comandos ainda privados,
intenções internas, dados de IA não publicados ou tiles desconhecidos.

```text
StateDiff {
  fromCursor: Cursor,
  toCursor: Cursor,
  turn: TurnNumber,
  operations: [UpsertEntity | RemoveEntity | PatchSummary | MapChunkRef]
}
MapChunkRequest { chunkId, knownRevision? }
MapChunk { chunkId, revision, tiles: [KnownTileView] }
KnownTileView { tileId, terrain, visibleFeatures, knownOwner?, knownEntities? }
```

Um chunk só inclui tiles conhecidos pela civilização na revisão solicitada. Campo, entidade ou
efeito ainda não descoberto é omitido, não codificado como valor secreto ou previsível. Se o
conhecimento for perdido ou corrigido, uma operação explícita de remoção invalida o cache local.

`StateDiff` é incremental e ordenado por `cursor`. Lacuna, cursor inválido ou diff maior que o
limite obriga `StateRecovery`, com snapshot de visão paginado/chunkado. O hash em `TurnResolved`
é o hash autoritativo do estado completo para auditoria; o cliente não deve inferir que consegue
reproduzi-lo a partir da visão parcial.

As variáveis compartilhadas usam a fonte única `docs/gdd/12-variaveis-e-formulas.md`: quando
publicadas, `C`, `D`, `G`, `W`, `E`, `P`, `S`, `L`, `Cf`, `R`, `Dv`, `A` e `U` preservam escala e
escopo definidos ali. O protocolo não recalcula fórmulas; transporta valores, causas auditáveis e
revisão do turno. `W` e `E` continuam provisórios e não ganham significado novo neste SDD.

## 8. Notificações

```text
Notification {
  notificationId: UUID,
  kind: "turn_open" | "decision_needed" | "turn_resolved" | "return_report",
  worldId, turn?, priority: "normal" | "critical",
  title, body, deepLink?, causalReferences: [EntityId | EventId]
}
```

Notificação é convite para reconectar, não comando e nem evidência de presença. O conteúdo deve
ser mínimo e não revelar mapa, negociações ou dados de uma civilização em tela bloqueada. A entrega
por serviço móvel é adaptador externo; o protocolo fornece a intenção versionada e deduplicável.

## 9. Limites e orçamento

Os números abaixo são propostas iniciais para medir e ajustar em hardware de referência, não metas
já decididas.

| Item | Proposta de limite/medida | Fallback |
|---|---|---|
| Comando cliente | 16 KiB descompactado; lote máximo 32 comandos | rejeitar antes de validar, com `PayloadTooLarge` |
| Mensagem WebSocket | 64 KiB descompactados | dividir em `MapChunk`/paginar recuperação |
| Diferença de estado | até 128 KiB por frame lógico | enviar múltiplos frames ordenados ou recuperação |
| Chunk de mapa | até 48 KiB descompactado | reduzir dimensão ou paginar tiles conhecidos |
| Frequência | limite por sessão e por tipo, a definir por teste | `RateLimited`, sem mudar estado |
| Processamento de transporte | medir p95 por comando, diff e fechamento de turno | fila limitada; nunca atrasar `step` por I/O |

O orçamento de tempo do turno é por trabalho computacional, jamais duração em horas: medir
validação, serialização, persistência e entrega separadamente do `step`. O servidor aplica backpressure
e pode atrasar diffs, mas não reordenar comandos já aceitos.

Não há chamada de IA no caminho do transporte. Se a UI mostrar consumo, recebe somente um resumo
autorizado produzido por outro subsistema. Tokens, custo por chamada e teto diário continuam sob os
orçamentos de IA; falha, timeout ou teto atingido resulta em T0 e nos comandos gravados, nunca em
payload de chave ou prompt.

## 10. Falhas, log e determinismo

| Situação | Comportamento proposto |
|---|---|
| pacote repetido | deduplicar por `request_id`/`clientCommandId`; devolver confirmação anterior |
| conexão cai após envio | cliente consulta resultado pela correlação ou recupera pelo cursor; não reenvia cegamente |
| versão incompatível | bloquear entrada de domínio e pedir atualização compatível |
| token expirado | recusar autenticação, permitir renovação; não perder comandos já aceitos |
| diff perdido/corrompido | invalidar cache afetado e recuperar visão autorizada |
| IA do Governador falha | outro subsistema aplica T0 e registra o comando resultante; transporte só publica efeito |
| limite excedido | rejeitar sem gravar comando nem cobrar custo mecânico |

O log auditável deve registrar: versão negociada, `request_id`, correlação, `worldId`, identificador
de usuário/civilização pseudonimizado quando possível, tipo, tamanho, resultado, código de rejeição,
`commandId`, sequência aceita, turno, cursor e latências. O log de comandos, em particular, registra
o comando validado e sua ordem; replays não chamam a rede ou IA novamente.

Nunca entram em log: access/refresh tokens, API keys BYOK, prompts completos, respostas brutas de
provedor, cookies, cabeçalhos de autorização, IP/hostname, conteúdo de tile não autorizado ou dados
de outra civilização. Diagnóstico que exija material sensível deve usar correlação e redigir valores.

Nunca entram em `step`: WebSocket/HTTP, sessão, presença técnica, relógio, token, cursor, retries,
payload comprimido, push, logs, I/O, IA, resposta de provedor ou ordem de iteração de coleção. A
entrada do motor é exclusivamente a sequência de comandos validados, seed versionada e versões de
catálogo estabelecidas pelo núcleo (ADR-0006).

## 11. Estratégia de testes

- Testes de contrato para cada mensagem: schema, campos obrigatórios, versão, limites e erro seguro.
- Testes de idempotência: reenvio, corrida entre duas sessões, reconexão e lote parcialmente aceito
  não duplicam `command_id` nem `accepted_sequence`.
- Testes de autorização e névoa de guerra: uma visão/chunk nunca contém tile, entidade ou atributo
  desconhecido; tentativa de outro `worldId`/civilização é recusada.
- Testes de propriedade: cursors publicados são monotônicos; uma sequência aceita é única por turno;
  `ReadySet(false)` não apaga comando aceito; valores expostos respeitam a escala da fonte de
  variáveis compartilhadas.
- Fixtures gravadas de WebSocket/HTTP cobrem `Hello`, recuperação, diffs, erros e notificações.
  Fixtures de IA são record/replay e não chamam API real em CI.
- Harness ponta a ponta injeta latência, duplicação, perda, reconexão e payload grande; então compara
  o log de comandos e hash final a um replay golden do motor.
- Se o ADR-0007 for aceito, testes de integração cobrem os adaptadores HTTP/WebSocket escolhidos;
  os mesmos vetores de contrato devem sobreviver a uma troca de framework, banco ou cliente.

## 12. Perguntas abertas para o usuário

1. Qual evento técnico define que uma sessão deixou de estar ativa no mundo após uma queda de rede?
   **Recomendação:** considerar ausente ao desligamento observado pelo servidor e permitir retorno
   normal no turno seguinte; evita relógio de jogo e não bloqueia o mundo.

2. Um humano que caiu no meio do turno pode reconectar e desfazer `Pronto` antes do fechamento?
   **Recomendação:** sim, se o turno ainda estiver aberto; comandos aceitos permanecem e novas ordens
   ficam no fim da sequência, preservando ADR-0003.

3. Qual mecanismo de identidade deve autenticar a conta na primeira versão?
   **Recomendação:** manter a porta de autenticação abstrata neste SDD e decidir o provedor em ADR de
   segurança, antes de implementar onboarding.

4. Os limites iniciais de 16/64/128 KiB para payloads que não são chunks são adequados para o alvo Android?
   **Recomendação:** aprová-los apenas como teto experimental e recalibrar com traces reais no harness.

5. O cliente pode guardar um snapshot de visão criptografado para abertura rápida, embora não haja modo
   offline jogável?
   **Recomendação:** permitir apenas cache descartável, invalidável por cursor e sem credenciais; o
   conteúdo exato e a criptografia devem ser fechados no SDD de segurança/cliente.

## Referências

- ADR-0001 — servidor autoritativo.
- ADR-0003 — turnos simultâneos e ordem de aceitação.
- ADR-0006 — motor determinístico e event sourcing.
- ADR-0008 — turno sem relógio.
- GDD 01 — loop e turnos; GDD 11 — experiência mobile.
- `docs/gdd/12-variaveis-e-formulas.md` — fonte única das variáveis compartilhadas.
