# SDD 21 — Notificações push

> **Status:** proposta para revisão. Deriva das categorias do GDD 11 e da integração técnica
> pesquisada; os detalhes de persistência, políticas e FCM abaixo ainda não foram aprovados.

## Objetivo e fronteiras

Despertar o jogador no Android quando houver uma decisão relevante, sem transformar o jogo por
turnos em tempo real. O push é um aviso de melhor esforço: estado, opções e prazos autoritativos
continuam no servidor e são recuperados pela API autenticada.

Este subsistema observa transições já persistidas e produz notificações. Não calcula regras de jogo,
não chama `step`, não muda `WorldState`, não aceita comandos e não condiciona avanço de turno à
entrega. `step(state, accepted_commands, seed, versions)` continua puro. Relatórios rotineiros,
narrativa e ações delegadas bem-sucedidas aparecem na Pauta ao abrir o app, não geram push.

## Eventos elegíveis

As únicas categorias acionáveis previstas pelo GDD 11 são:

- escolha crítica com prazo de poucos turnos;
- proposta diplomática que expira;
- gatilho do Mandato que pede intervenção do jogador;
- crise iminente.

O produtor deriva a elegibilidade de uma transição ou projeção autoritativa persistida, por exemplo
um `DomainEvent` ou uma pendência de decisão. O push começa pela consequência, seguida de causa
curta, e abre o cartão correspondente. A pauta agrupa ocorrências repetidas. A distinção entre
`crise iminente` e os limiares mecânicos de `P` deve seguir GDD 11 e GDD 12; este SDD não cria
limiares novos.

O GDD 11 registra como **valor inicial sujeito a balanceamento** no máximo uma notificação
acionável por civilização a cada 6 horas, com exceção quando uma decisão anterior do jogador tiver
prazo menor explicitamente exibido. Proposta técnica: aplicar o limite no serviço de despacho usando
relógio operacional, agrupar outras ocorrências elegíveis durante a janela e encaminhar no próximo
despertar ou na reconciliação do app. A semântica exata da exceção, janela móvel versus janela fixa,
e tratamento de prioridades são decisões abertas; a frequência de push nunca altera prazos do jogo,
que contam turnos (ADR-0008).

## Arquitetura proposta

1. O orquestrador resolve a transição e persiste seus resultados autoritativos.
2. Na mesma transação lógica de persistência, grava um registro de outbox para cada notificação
   elegível. A outbox é escrita fora de `step`; a transação associa o aviso a dados já confirmados,
   evitando push de transição revertida.
3. Um worker externo reivindica registros pendentes, aplica preferências e limite de frequência,
   envia mensagem de dados mínima pelo FCM e registra tentativa/resultado.
4. O app recebe o aviso e, ao toque, busca estado/delta e decisões pendentes pela API autenticada.
   Também reconcilia ao abrir, mesmo sem receber push.

O worker usa entrega pelo menos uma vez; servidor e cliente toleram duplicatas. Falha ou indisponibilidade
do FCM não bloqueia o mundo. Registros com falha podem ser repetidos com espera limitada e backoff;
tentativas esgotadas ficam observáveis e são recuperadas pela reconciliação. Política de retenção e
prazo de expurgo da outbox são propostas a alinhar com SDD 19.

## Contratos e dados

Os contratos abaixo são propostas. `Notification` no protocolo (SDD 10) continua sendo o envelope
de entrega em sessão conectada; a mensagem FCM é um sinal menor e não transporta esse envelope
completo.

```text
enum NotificationCategory {
  critical_choice, expiring_proposal, mandate_attention, imminent_crisis
}

enum OutboxStatus { pending, claimed, sent, suppressed, failed }

NotificationOutboxRecord {
  notification_id: OpaqueId<"notification">
  user_id: UserId
  world_id: WorldId
  civilization_id: CivilizationId
  category: NotificationCategory
  source_event_id?: DomainEventId
  decision_id?: OpaqueId<"decision">
  turn: TurnNumber
  state_revision: Integer
  deduplication_key: String
  status: OutboxStatus
  created_at: OperationalTimestamp
  attempt_count: Integer
  next_attempt_at?: OperationalTimestamp
}

PushPreference {
  user_id: UserId
  device_id: DeviceId
  enabled: Boolean
  categories_enabled: Map<NotificationCategory, Boolean>
  updated_at: OperationalTimestamp
}
```

`OperationalTimestamp` é infraestrutura fora do estado simulado. A chave de deduplicação proposta é
estável para a mesma decisão lógica, por exemplo mundo + civilização + categoria + ID da decisão ou
evento causal; quando não houver ID causal, usar turno e revisão da projeção. Uma restrição única na
outbox impede a criação duplicada. Uma decisão ainda aberta não deve gerar novo aviso a cada retry,
reconexão ou reprocessamento. Uma nova revisão só gera outro aviso se representar decisão distinta
ou mudança material que reabra a escolha, regra ainda sujeita a validação.

Preferências são por usuário/dispositivo e categoria; desabilitar push não desabilita a Pauta nem a
reconciliação autenticada. A associação, revogação e remoção do token por dispositivo devem respeitar
os contratos de sessão e ciclo de vida do SDD 17. Token FCM não é identidade nem credencial de
autorização.

## FCM e privacidade

Proposta: cliente Android registra o token FCM e o envia por canal autenticado; servidor auto-hospedável
armazena o token associado a `DeviceId` e envia mensagens de dados pelo FCM. O adaptador servidor
usa HTTP v1 e credencial do operador mantida fora do repositório e dos logs; seleção e operação da
credencial precisam de spike/decisão de segurança antes da produção. Não usar a antiga Server Key.
FCM serve apenas para despertar, não como transporte confiável do estado. O Android apresenta a
notificação conforme o estado do app e as permissões/canais locais.

Payload mínimo proposto, sem texto específico do mundo:

```json
{
  "notification_id": "opaque-id",
  "world_ref": "opaque-ref",
  "category": "critical_choice",
  "revision": 42
}
```

O valor de `category` usa `NotificationCategory`; os nomes e esquema finais devem ser alinhados ao protocolo.
Correção aplicada em 2026-10-01. Não incluir narrativa, texto de jogador,
Mandato, termos diplomáticos, nomes de entidades, conteúdo de tiles, estado do mundo, API key ou token
de sessão. O app usa os identificadores opacos somente como pistas para buscar dados que a sessão
autorizada pode acessar. Logs e métricas registram IDs de correlação, categoria, latência e resultado,
sem token FCM nem conteúdo sensível.

## Reconciliação no cliente

Ao abrir o app ou tocar um push, o cliente autentica e solicita notificações/decisões pendentes desde
sua última revisão confirmada. O servidor valida autorização atual e devolve a Pauta autoritativa,
com decisão ainda aberta, prazo em turnos e referências necessárias ao cartão. Notificações já
resolvidas, expiradas, revogadas ou inacessíveis são descartadas/atualizadas sem aplicar ação. A
revisão recebida é monotônica por escopo definido no contrato; cliente nunca infere resultado do
texto do push. O fluxo funciona se todos os pushes forem perdidos, duplicados ou chegarem fora de
ordem.

## Falhas, limites e observabilidade

- Timeout, quota, token inválido ou indisponibilidade do FCM: registrar resultado operacional e seguir
  sem bloquear persistência, turno ou API.
- Token rejeitado permanentemente: marcar inválido e solicitar novo registro no próximo uso do app.
- Preferência desativada ou limite de frequência: marcar como `suppressed`, mantendo item na Pauta.
- Sessão/civilização revogada: não entregar; reavaliar autorização antes de toda reconciliação.
- Duplicidade após timeout ambíguo: pode haver apresentação repetida pelo provedor; `notification_id`
  permite deduplicação do lado do cliente quando o app puder fazê-la.

Métricas propostas: tamanho/idade da outbox, tentativas, suppressões por preferência/limite, taxa de
tokens inválidos, latência de despacho e reconciliações com itens novos. Não medir conteúdo privado.

## Testes propostos

- Teste de fronteira: gerar e despachar notificações não chama `step` nem altera hash/estado do mundo.
- Atomicidade: rollback da transição não deixa registro enviável; commit deixa exatamente a outbox
  correspondente.
- Idempotência e concorrência: reprocessar evento, executar workers concorrentes e repetir envio não
  cria itens lógicos duplicados.
- Elegibilidade: fixtures para as quatro categorias; eventos rotineiros, narrativa e delegação
  bem-sucedida não produzem push.
- Frequência: relógio operacional injetável cobre limite de 6 h e casos da exceção de prazo curto,
  sem afetar contador de turnos.
- Preferências: configuração global/categoria e revogação de dispositivo suprimem entrega, mas
  preservam a Pauta/reconciliação.
- Privacidade: inspeção de payload/log garante ausência de dados sensíveis e tamanho dentro do limite
  adotado pelo adaptador.
- Falhas: timeout, rejeição de token, indisponibilidade, retry e backoff não bloqueiam o jogo nem
  perdem a decisão pendente.
- Reconciliação: push perdido, duplicado, atrasado e fora de ordem resulta na mesma pauta autoritativa;
  item expirado/resolvido não abre ação inválida.
- CI usa FCM falso/local e fixtures; nunca contata Firebase.

## Decisões e perguntas abertas

- **Decidido por referência:** categorias permitidas e limite inicial de uma notificação acionável por
  civilização a cada 6 h, sujeito a balanceamento, com a exceção descrita no GDD 11; push não leva
  estado sensível e a reconciliação autenticada é necessária, conforme pesquisa técnica.
- **Propostas neste SDD:** esquema da outbox, chave de deduplicação, preferências por categoria,
  política de retry, contrato da reconciliação e uso de FCM HTTP v1 pelo servidor auto-hospedável.
- **Aberto:** semântica exata da exceção ao limite; janela móvel/fixa; expurgo e retenção; credencial e
  rotação FCM; esquema final do payload em coordenação com SDD 10/12/17; sincronização de preferências
  entre dispositivos; comportamento de agrupamento no Android.

Referências: GDD 09 e GDD 11; GDD 12 (variáveis compartilhadas, sem novas fórmulas); SDD 09, 10, 12,
17 e 19; `docs/research/integracoes-tecnicas.md` — seção “FCM a partir de servidor auto-hospedado”.
