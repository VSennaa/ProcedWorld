# SDD 17 — Identidade, contas e sessões

> **Status:** proposta. Depende da aprovação do GDD e da ratificação da stack do ADR-0007.
> Este documento define contratos e invariantes; adaptadores concretos são intercambiáveis.

## 1. Objetivo, responsabilidades e fronteiras

O subsistema identifica uma pessoa, autentica suas sessões, autoriza ações fora e dentro de um
mundo e mantém a associação entre usuário, civilização e dispositivos. Ele permite revogar acesso,
aplica a política de presença do ADR-0008 e coordena entrada tardia e sucessão com o subsistema de
simulação. O servidor continua autoritativo (ADR-0001).

É responsável por:

- conta, credencial, recuperação e ciclo de vida de sessão;
- associação `UserId`–`WorldId`–`CivilizationId`, limitada a uma civilização por usuário em cada
  mundo;
- autorização de leituras e de propostas de ação, inclusive em múltiplos dispositivos;
- `PresencePolicy`, marcação de pronto por usuário e delegação técnica ao Governador;
- encaminhar ao motor os comandos administrativos que alteram participantes ou controle;
- entrada tardia, tomada de bot e escolha de sucessora, sempre depois de validação do motor.

Não é responsável por criar regras de mapa, economia, combate, diplomacia, Mandato, sucessão
territorial ou efeitos de uma ação. Tampouco escolhe decisões do Governador, executa IA, guarda
contexto de IA, calcula custo de inferência ou é fonte de verdade do estado mecânico. Esses domínios
recebem identidade e autorização como contexto, mas o motor decide a legalidade mecânica.

Uma conta pode existir sem mundo, uma sessão pode existir sem civilização e uma civilização de bot
não tem associação humana. Credenciais e sessões são estado operacional; `WorldState` não deve
conter tokens, horários de autenticação nem endereços de dispositivo.

## 2. Decisões herdadas e premissas

- Cada mundo tem 4–8 civilizações no MVP e, por usuário, no máximo uma civilização naquele mundo
  (GDD 00 — decidido).
- O cliente é uma projeção; somente o servidor aceita ações (ADR-0001).
- O Governador roda no servidor e respeita o Mandato; sem chave de IA, T0 mantém o jogo avançando
  (ADR-0002 e GDD 11 — decidido).
- Não existe prazo de turno: todos os humanos presentes marcam pronto; humanos ausentes são
  representados pelo Governador (ADR-0008).
- Entrada tardia permite fundar uma comunidade ou assumir uma civilização de bot. Ao assumir o bot,
  o estado, Ledger e Crônica são herdados; Mandato é preset editável e Doutrina vira sugestão
  (GDD 10 — decidido).

As estruturas e valores abaixo são **propostas**, exceto onde a decisão acima é citada.

## 3. Vocabulário e contratos públicos

```text
type UserId = OpaqueId<"user">
type SessionId = OpaqueId<"session">
type DeviceId = OpaqueId<"device">
type WorldId = OpaqueId<"world">
type CivilizationId = OpaqueId<"civilization">
type MembershipId = OpaqueId<"membership">
type TurnNumber = Integer

enum MembershipRole { owner, observer }
enum MembershipStatus { active, pending_claim, suspended, ended }
enum SessionStatus { active, revoked, expired }
enum PresenceState { present, reconnecting, absent }
```

`UserId` é estável e não reutilizável. `DeviceId` é um identificador aleatório de instalação, nunca
um fingerprint. `SessionId` é opaco, rotacionável e não é aceito como identidade dentro do motor.

```text
Account {
  user_id: UserId
  status: active | disabled | pending_deletion
  created_at: OperationalTimestamp
}

Session {
  session_id: SessionId
  user_id: UserId
  device_id: DeviceId
  status: SessionStatus
  issued_at, expires_at, revoked_at?: OperationalTimestamp
  credential_version: Integer
}

Membership {
  membership_id: MembershipId
  user_id: UserId
  world_id: WorldId
  civilization_id?: CivilizationId
  role: MembershipRole
  status: MembershipStatus
  control_revision: Integer
}
```

Uma `Membership` é a fonte operacional de autorização. `control_revision` evita que uma requisição
assinada antes de uma revogação ou troca de civilização seja aceita depois dela.

```text
Authenticate(credential, device_proof) -> AuthResult { session, user }
Authorize(session, world_id, capability) -> Authorization { membership, revision }
RevokeSession(actor, target_session_id | target_device_id | all_other_sessions) -> RevocationReceipt
SubmitAction(session, ActionIntent) -> Accepted | Rejected
MarkReady(session, world_id, turn) -> Accepted | Rejected
ChooseEntry(session, world_id, EntryChoice) -> Accepted | Rejected
ChooseSuccessor(session, world_id, successor_id) -> Accepted | Rejected
```

`SubmitAction` autentica e autoriza antes de enviar a intenção ao validador de domínio. O recibo
exibe `accepted_sequence`, nunca pressupõe que a intenção alterou o estado. Os contratos de ação
usam os nomes canônicos compartilhados:

```text
ActionIntent {
  request_id, actor_id: UserId, world_id, turn, kind, payload,
  schema_version, grounding: GroundingRef[], control_revision
}

AcceptedCommand {
  command_id, world_id, turn, accepted_sequence, actor_id,
  origin: player | governor | bot | entropy | fallback | system,
  kind, payload_canonical, schema_version, ruleset_ref: RulesetRef,
  grounding: GroundingRef[], intent_evidence_ref?
}
```

`ActionIntent` não entra no replay por si só. Depois de validada, ela vira `AcceptedCommand`; replay
aplica somente esse último. `IntentEvidence` é auditável e não altera o comando histórico.
O campo `ruleset_ref` segue o nome canônico da revisão cruzada (correção aplicada em 2026-10-01).

## 4. Política de presença proposta

`PresencePolicy` é configuração operacional versionada e não uma regra de tempo do jogo.

```text
PresencePolicy {
  reconnect_grace_ms: 60_000 // correção aplicada em 2026-10-01
  heartbeat_interval_ms: 15_000
  enrollment: "at_turn_open"
  cutoff: "delegate_for_current_turn"
  ready_scope: "per_user_per_turn"
}
```

Um usuário está `present` se tiver ao menos uma sessão `active` conectada ao mundo. Várias sessões
do mesmo usuário contam como uma só presença e compartilham uma única marca de pronto. A conexão
de qualquer dispositivo durante a janela mantém o usuário presente; isso permite trocar de aparelho
sem duplicar ação ou bloquear o turno.

Fluxo por turno:

1. Na abertura, o orquestrador observa a presença e registra `FreezeTurnParticipants` com a lista
   canônica de `UserId` esperados e a revisão da política.
2. Um participante pode enviar comandos e, ao final, `MarkReady`. Após pronto, novas ações daquele
   usuário para o turno são rejeitadas; outro dispositivo pode apenas consultar o recibo.
3. Queda de todas as sessões abre `reconnecting`; a janela é medida fora do motor. Se uma sessão
   retornar, nada mecânico ocorre. Se expirar, o orquestrador aceita `DelegateAbsentParticipant`,
   que retira o usuário dos esperados daquele turno e permite ao Governador agir.
4. Após a delegação, o usuário não reentra naquele turno; volta como candidato no turno seguinte e
   recebe o relatório de retorno. Revogação tem o mesmo efeito de uma ausência já confirmada.
5. O turno fecha quando todo participante ainda esperado está pronto ou delegado e os comandos de
   automação requeridos já foram aceitos.

```text
FreezeTurnParticipants { turn, participant_user_ids: UserId[], policy_revision }
MarkReady { turn, user_id, membership_id, control_revision }
DelegateAbsentParticipant { turn, user_id, membership_id, reason: disconnect | revoked }
```

Esses três são `AcceptedCommand` de origem `system` ou `player` conforme o caso. A proposta evita
introduzir hora, heartbeat ou duração de janela em `step`: o replay vê apenas a decisão registrada.
Mudanças de associação ou entrada tardia solicitadas depois de `FreezeTurnParticipants` passam a
valer no próximo turno, preservando uma lista de participantes estável.

## 5. Associação, entrada tardia e sucessão

Antes de criar ou alterar uma associação, o serviço consulta uma projeção autoritativa do mundo e
envia o comando ao motor. A gravação operacional de `Membership` só é efetivada após aceitação do
comando correspondente. Entrada tardia tem duas opções decididas: fundar uma comunidade nova,
protegida e sem vínculos ou patrocinada; assumir civilização controlada por bot. Só alvo sem humano
pode ser assumido. Ao assumir, herda estado, `Ledger` e Crônica; o Mandato vira preset inicial
editável e a Doutrina vira sugestões (GDD 10 — decidido).

```text
EntryChoice =
  | FoundCommunity { sponsorship_civilization_id?: CivilizationId }
  | TakeOverBot { target_civilization_id: CivilizationId }

ClaimCivilization { user_id, membership_id, choice: EntryChoice, turn, control_revision, ruleset_ref }
SelectSuccessor { user_id, former_civilization_id, successor_civilization_id, turn,
                  control_revision, ruleset_ref }
```

Os nomes/campos desses comandos são **proposta**. Eles representam a reivindicação de entrada
tardia e a escolha humana da comunidade sucessora; sua aceitação gera `AcceptedCommand` com origem
`player`, sequência aceita e `ruleset_ref` canônico. A associação operacional não é a fonte do
resultado mecânico.

Pré-condições verificáveis para `ClaimCivilization`:

- conta/sessão autorizadas e no máximo uma civilização humana por usuário/mundo (GDD 00 — decidido);
- capacidade de entrada disponível segundo a regra de mundo (detalhe de capacidade é **proposta**);
- `FoundCommunity` tem local legal; patrocinador, se informado, é elegível. Proteção, tile/local,
  custos e obrigações do patrocínio seguem catálogo, mas seus valores e validações específicas são
  **proposta**;
- `TakeOverBot` aponta para civilização existente controlada por bot e sem humano (GDD 10 — decidido);
- controle/revisão e turno correspondem à projeção autoritativa. Compare-and-swap impede duas
  reivindicações concorrentes; mecanismo técnico é **proposta**.

Ao assumir bot, o comando não reescreve história: mantém estado, `Ledger` e Crônica, transforma o
Mandato existente em preset inicial editável e expõe a Doutrina como sugestões (GDD 10 — decidido).
Fundar comunidade cria o assentamento pelo comando e regras do motor. A transição de ownership e a
materialização do preset no registro da conta após aceitação são **propostas**; não se copia estado
para uma nova civilização na tomada de bot.

No colapso, o Governador só mantém sobrevivência básica validada; escolha de sucessor exige o jogador
(GDD 09 — decidido). O motor calcula comunidades elegíveis e aplica território, relações e legados
pelas regras da seção 6 de SDD 15. O serviço apresenta IDs estáveis e só encaminha `SelectSuccessor`
para uma opção elegível; ordenação/apresentação da lista é **proposta**. Após aceitação, a associação
ativa é transferida atomicamente da civilização colapsada à escolhida; as demais comunidades tornam-se
civilizações de bot com suas relações e reivindicações. A transferência única sem intervalo de dois
ownerships é contrato operacional **proposto**, derivado do limite de uma civilização humana por
mundo.

Herança determinística: estado/Ledger/Crônica existentes são mantidos ao tomar bot; na sucessão, cada
relação (dívida, reivindicação, traição) só acompanha o sucessor identificável que herdou seu
território, obrigações de defesa expiram e tratados podem ser ratificados, renegociados ou repudiados
com registro causal (GDD 07 — decidido). Território, população, capacidade, reservas e legados da
sucessora obedecem às proporções e condições decididas em GDD 04/10 e à regra mecânica de SDD 15.
Matriz de campos herdados na projeção de conta, retentativa idempotente e atomicidade transacional
são **propostas**; nenhum desses metadados de conta substitui o estado autoritativo do motor.

## 6. Modelo de dados, invariantes e autorização

Projeções operacionais podem ser mutáveis, mas precisam preservar uma trilha de auditoria de criação,
troca e revogação. O estado do mundo, log de comandos e snapshots permanecem os artefatos
autoritativos para mecânica.

Invariantes:

- existe no máximo uma `Membership(active, owner)` por par `(user_id, world_id)` e por par
  `(world_id, civilization_id)`;
- uma sessão ativa pertence a exatamente uma conta ativa; sessão revogada ou expirada jamais ganha
  novo acesso;
- um `DeviceId` pode ter várias sessões históricas, mas a revogação por dispositivo invalida todas
  as ativas e incrementa a revisão de credencial;
- uma única marca `MarkReady` é aceita por `(world_id, turn, user_id)`; seu efeito é idempotente;
- `DelegateAbsentParticipant` só é aceito para participante congelado, ainda não pronto e cuja
  ausência foi confirmada pela política;
- toda mudança de controle possui `control_revision` monotônica e uma causa de comando ou decisão
  administrativa auditável;
- credenciais, tokens de sessão, chaves BYOK e seus hashes verificadores nunca são retornados ao
  cliente, gravados em `AcceptedCommand`, `IntentEvidence`, log de aplicação ou `WorldSnapshot`.

O adaptador de persistência deve impor índices únicos transacionais para as duas primeiras regras e
uma transação/compare-and-swap para prontidão, reivindicação e sucessão concorrentes. Se ADR-0007
for aceito, PostgreSQL é uma implementação possível; nenhum contrato acima depende dele.

## 7. Falhas, fallback e determinismo

Falha de login, credencial inválida, sessão expirada, autorização negada, revisão obsoleta ou disputa
por tomada retorna erro tipado sem alterar mundo. Falha temporária de presença mantém
`reconnecting` até a janela técnica acabar; indisponibilidade do detector não declara ausência sem
evidência observável e deve alertar operação. Se a confirmação de delegação falhar, o turno fica
aberto e o servidor tenta registrar a mesma transição idempotente; ele nunca inventa uma jogada.

O Governador é o fallback de ausência, não o sistema de identidade. Se uma chamada T1/T2 falhar,
estourar custo ou não tiver chave, o fallback T0 gera uma intenção válida dentro do Mandato; a ação
aceita, e não a chamada de IA, entra no replay. BYOK opcional não é condição para criar conta nem
civilização (GDD 11 — decidido).

No log de comandos entram `FreezeTurnParticipants`, `MarkReady`, delegação confirmada, reivindicação
aceita, seleção de sucessora e qualquer ação mecânica convertida em `AcceptedCommand`. Em auditoria
operacional entram criação, autenticação bem-sucedida/falha sem segredo, sessão, revogação, troca de
controle, motivo, identificadores e horário técnico. Logs podem usar um identificador de dispositivo
pseudônimo, com retenção definida separadamente.

Nunca entram em `step`: senha, token, chave API, IP/hostname, cabeçalho, horário, heartbeat,
latência, estado de socket, modelo de aparelho, corpo de texto de login, decisão bruta de IA ou
ordem incidental de requisições. `step(state, accepted_commands, seed, versions)` recebe comandos
em `accepted_sequence`, seed explícita e versões; nenhum adaptador de autenticação ou relógio é
consultado durante a transição. Isso atende ao determinismo e event sourcing do ADR-0006.

## 8. Orçamento proposto

O caminho de autenticação deve ser curto e isolado do fechamento do turno. Metas iniciais, a medir
antes de aprovação: autorização local p95 até 50 ms, emissão/renovação p95 até 150 ms, marca de
pronto p95 até 100 ms e revogação visível a novos pedidos em até 1 s operacional. Nenhuma dessas
metas equivale a relógio de jogo.

Por mundo, o estado de presença mantém no máximo uma entrada por associação humana e uma por sessão
ativa; no MVP são no máximo oito civilizações. O servidor limita sessões ativas por conta a **cinco**
(proposta), rejeitando a sexta até revogação explícita. Retenção de auditoria e limite por conta
precisam de validação operacional antes da implementação.

Identidade, presença e revogação não fazem chamadas de IA e têm orçamento de IA zero. As chamadas
do Governador obedecem o teto diário da civilização: prefixo estável, limite de tokens por porta e
custo atribuído ao usuário/civilização sem registrar chave ou prompt sensível. Sem orçamento, T0 é
obrigatório e determinístico.

## 9. Estratégia de testes

- testes de propriedade para unicidade usuário–mundo e civilização–dono, monotonicidade de revisão,
  idempotência de pronto/revogação e impossibilidade de agir com sessão revogada;
- testes de corrida com duas sessões e duas reivindicações simultâneas, provando exatamente um
  vencedor e uma associação final consistente;
- testes de política: reconexão em outro dispositivo dentro da janela, queda após pronto, expiração
  da janela, revogação no meio do turno e retorno somente no turno seguinte;
- replay golden: mesma sequência de `AcceptedCommand`, inclusive congelamento e delegação, produz o
  mesmo `StateHash` em máquinas distintas sem relógio, rede ou provedor de IA;
- fixtures gravadas para autenticação/portas de IA usam identificadores sintéticos e segredos
  redigidos; CI nunca chama provedor de identidade, LLM ou API real;
- harness headless com bots, entradas tardias, tomada de bot e colapso/sucessão verifica que todo
  usuário tem no máximo uma civilização e que cada delegação tem comando causal;
- teste de segurança procura tokens em respostas, logs, comandos, snapshots e fixtures; qualquer
  vazamento falha o pipeline.

## 10. Perguntas abertas para o usuário

1. Qual mecanismo de credencial entra no MVP (senha, passkey, provedor externo ou combinação)?
   **Recomendação:** passkey mais recuperação controlada, para reduzir senhas e funcionar bem no
   celular, mantendo a porta de autenticação independente de fornecedor.

2. Até cinco sessões ativas por conta são aceitáveis?
   **Recomendação:** manter cinco como limite operacional versionado e ajustável
   por telemetria, sem efeito no relógio do jogo.

3. Observadores sem civilização podem entrar em mundo no MVP, e o que podem ver?
   **Recomendação:** não habilitar observação pública até o SDD de visibilidade definir fog of war;
   manter somente o papel no contrato para não bloquear evolução posterior.

4. Quando a conta for desativada ou apagada, a civilização permanece como bot, pode ser assumida por
   outra pessoa ou exige ação do proprietário?
   **Recomendação:** revogar imediatamente as sessões, preservar a civilização como bot e permitir
   tomada normal por outro usuário, sem apagar log, Ledger ou Crônica.

5. A janela diária e a atribuição da chave T1 pertencem ao operador ou ao jogador?
   **Recomendação:** decidir isso junto da sub-decisão de chaves do ADR-0002; até lá, contabilizar
   separadamente custo de T1 e T2 por civilização e garantir fallback T0 em ambos.

## ADRs relacionados

- ADR-0001 — servidor autoritativo.
- ADR-0002 — Governador no servidor e BYOK proposto.
- ADR-0006 — motor determinístico com event sourcing.
- ADR-0008 — turno sem relógio.
- ADR-0007 — stack tecnológica proposta; detalhes de implementação só serão fixados se aceito.
