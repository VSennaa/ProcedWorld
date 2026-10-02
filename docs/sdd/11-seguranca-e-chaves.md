# 11 — Segurança e chaves (BYOK)

> **Status: proposta.** Este documento detalha a sub-decisão proposta do ADR-0002.
> O GDD ainda aguarda aprovação e o ADR-0007 também é proposto. Contratos e
> invariantes abaixo não dependem de linguagem, banco ou provedor.

## Contexto e objetivos

O jogador pode fornecer uma API key para habilitar narrativa, planejamento e
negociação em linguagem natural (T2). A chave é opcional: sem ela, ou quando
ela não puder ser usada, a civilização continua com T0 determinístico. T1 é
separado porque a titularidade da chave do Jev/Runware ainda não foi decidida.

O subsistema protege segredos de jogadores, limita o gasto atribuível a cada
jogador e permite que ele revogue seu acesso. Ele preserva a auditabilidade do
jogo sem colocar segredos, texto livre ou I/O no motor determinístico.

## Responsabilidades e fronteiras

### É responsabilidade deste subsistema

- Receber, validar superficialmente e guardar uma chave BYOK por jogador e
  provedor, sempre por canal autenticado e protegido em trânsito.
- Criptografar a chave por envelope, controlar acesso para uso, rotacionar a
  proteção e apagá-la de forma efetiva.
- Reservar e contabilizar custo diário por jogador antes e depois de chamadas
  T2, expondo somente consumo agregado ao jogador.
- Produzir eventos de auditoria sem segredos, aplicar limites e sinalizar ao
  orquestrador quando T2 não estiver disponível.
- Preparar texto de jogador como dado delimitado para o adaptador de IA, nunca
  como instrução.

### Não é responsabilidade deste subsistema

- Decidir ações, validar intenções, aplicar Mandato ou alterar o estado do
  mundo; isso é responsabilidade do motor e das camadas de IA.
- Interpretar, moderar semanticamente ou prometer segurança absoluta contra
  conteúdo do jogador; ele isola o conteúdo e conserva os limites mecânicos.
- Calcular efeitos diplomáticos, aceitação ou valores como `Cf`, `R`, `Dv` e
  `A`; esses valores pertencem à Diplomacia e ao motor.
- Ser a fonte de verdade do estado, do log de comandos, de snapshots ou da
  identidade/autenticação de conta. Pode referenciá-los por IDs opacos.
- Escolher provedor, preço, moeda ou a chave que financia T1; essas são
  configurações/políticas sujeitas às perguntas abertas.

## Ameaças e mitigação

| Ameaça | Mitigação proposta |
|---|---|
| Vazamento pelo banco, backup ou exportação | Chave de dados aleatória por registro; segredo fica cifrado; chave-mestra fica fora do banco e de seus backups. |
| Vazamento em log, erro, métrica ou suporte | Redação estrutural, allowlist de campos e testes que procuram padrões de segredo; nunca registrar cabeçalhos de autorização nem corpos de prompt. |
| Conta A usar a chave ou orçamento de B | Toda operação recebe `player_id` autenticado; autorização é conferida antes de revelar a chave e a conta de custo é segregada por jogador. |
| Corrida que excede o teto diário | Reserva atômica antes da chamada, confirmação posterior e recusa quando não houver saldo reservado. |
| Prompt injection em mensagem diplomática | Texto livre fica em bloco de dados serializado e delimitado; instruções vêm somente do prefixo confiável; saída passa por schema e validação do motor. |
| Comprometimento da chave-mestra | Versões de chave, rotação, recriptografia gradual, acesso mínimo e resposta operacional de revogação; comprometimento requer substituir a versão afetada. |
| Provedor indisponível ou cobra diferente do previsto | Timeout, limite de saída, reconciliação de uso e fallback T0; o turno nunca depende da chamada. |

## Contratos propostos

Os tipos usam IDs opacos e valores monetários inteiros na menor unidade de
cobrança configurada. Nenhum contrato devolve o segredo em texto claro.

```text
type PlayerId = opaque string
type ProviderId = opaque string
type KeyId = opaque string
type KeyVersion = positive integer
type MoneyMicros = nonnegative integer
type UtcDay = "YYYY-MM-DD"

enum KeyState { active, deleting, deleted, disabled }
enum AiAvailability { available, no_key, budget_exhausted, disabled, provider_error }

record KeyHandle {
  key_id: KeyId
  provider_id: ProviderId
  state: KeyState
  fingerprint: string       // derivado não reversível, curto; nunca a chave
  created_at: Instant
  rotated_at: Instant?
}

record DailyBudget { limit: MoneyMicros, reserved: MoneyMicros, settled: MoneyMicros }
record UsageQuote { maximum: MoneyMicros, input_token_limit: integer, output_token_limit: integer }
```

```text
KeyVaultPort.store(player_id, provider_id, secret_bytes) -> KeyHandle
KeyVaultPort.acquire_for_call(player_id, provider_id, purpose) -> EphemeralSecret | AiAvailability
KeyVaultPort.rotate_protection(key_id) -> KeyHandle
KeyVaultPort.delete(player_id, provider_id) -> DeleteReceipt
KeyVaultPort.list(player_id) -> KeyHandle[]

BudgetPort.configure(player_id, provider_id, limit, utc_day) -> DailyBudget
BudgetPort.reserve(player_id, provider_id, quote, call_id) -> Reservation | AiAvailability
BudgetPort.settle(reservation, actual_cost, provider_usage) -> DailyBudget
BudgetPort.release(reservation, reason) -> DailyBudget

PromptBoundaryPort.wrap_untrusted(player_text, source, text_id) -> UntrustedDataBlock
AuditPort.record_security(event) -> void
```

`EphemeralSecret` só pode ser consumido pelo adaptador do provedor durante uma
chamada; sua representação textual não é exposta a logs, erros, telemetria ou
ao cliente. `purpose` é uma enumeração curta, por exemplo `narrative`,
`planning` ou `diplomacy_parse`, e não recebe texto livre.

Exemplo de fluxo de T2:

```text
quote = policy.quote(provider, purpose, input_tokens, output_tokens)
reservation = BudgetPort.reserve(player, provider, quote, call_id)
if reservation is unavailable: return T0(reason)

secret = KeyVaultPort.acquire_for_call(player, provider, purpose)
if secret is unavailable: release(reservation); return T0(reason)

response = provider.call(secret, trusted_prefix, state_facts, untrusted_blocks)
settle(reservation, measured_cost_or_quote, provider_usage)
return validate_as_intention_or_fallback(response)
```

O adaptador recebe fatos de estado e dados não confiáveis em campos distintos.
Para diplomacia, tenta-se primeiro o cartão estruturado; a conversa livre só é
enviada ao LLM quando essa interpretação não basta, conforme o GDD 07.

## Modelo de dados proposto

```text
KeyEnvelope {
  key_id, player_id, provider_id, state,
  encrypted_data_key, encrypted_secret,
  master_key_version, algorithm_version,
  fingerprint, created_at, rotated_at, deleted_at
}

DailyUsage {
  player_id, provider_id, utc_day, limit,
  reserved, settled, updated_at, version
}

UsageReservation {
  reservation_id, call_id, player_id, provider_id, utc_day,
  maximum, state(pending|settled|released|expired), created_at, expires_at
}

SecurityAuditEvent {
  event_id, occurred_at, actor_kind, actor_id_hash,
  event_type, key_id?, provider_id?, call_id?, outcome, reason_code,
  cost_micros?, master_key_version?
}
```

Na criptografia de envelope, gera-se uma chave de dados aleatória por
`KeyEnvelope`; ela cifra `encrypted_secret`. A chave de dados é cifrada pela
chave-mestra versionada, que vive em mecanismo de segredos externo ao banco.
Metadados necessários para decifrar são autenticados junto ao cifrado, incluindo
`key_id`, `player_id`, `provider_id` e `algorithm_version`, para impedir troca
entre registros.

Se ADR-0007 for aceito, `KeyEnvelope`, `DailyUsage` e `UsageReservation` podem
ser tabelas PostgreSQL com controle transacional de versão; a chave-mestra deve
continuar fora do PostgreSQL. A escolha de biblioteca criptográfica, cofre de
segredos e mecanismo de lock fica para ADR específico ou implementação aprovada.

## Invariantes verificáveis

1. Um segredo em texto claro existe apenas na memória do processo durante a
   chamada autorizada; não existe em banco, snapshot, evento, backup lógico,
   log, métrica, mensagem de erro ou resposta de API.
2. `player_id` e `provider_id` autenticados devem coincidir com os metadados
   autenticados do envelope antes de qualquer decifragem ou chamada.
3. Apenas uma chave `active` por par `(player_id, provider_id)` é utilizável;
   versões anteriores ficam indisponíveis assim que a substituição é confirmada.
4. Uma chave `deleted` ou `deleting` nunca é adquirida para nova chamada.
5. Para cada conta diária, `0 <= settled + reserved <= limit`, salvo registro
   explícito de ajuste administrativo auditado. Reservas são idempotentes por
   `call_id`.
6. Uma resposta de provedor sem custo mensurável liquida no máximo o valor
   reservado; divergência fica auditada para reconciliação, nunca vira saldo
   negativo silencioso.
7. Rotação preserva a capacidade de decifrar envelopes ainda ativos até que cada
   um tenha sido recriptografado ou revogado; `master_key_version` é rastreável.
8. Apagar a chave destrói o cifrado e a chave de dados associada; eventos de
   auditoria preservam somente IDs, timestamps e resultado, não o segredo.
9. Texto de jogador e texto devolvido por provedor não são comandos. Apenas uma
   intenção tipada, validada pelo motor, pode resultar em comando.

## Ciclo de vida, falhas e fallbacks

### Cadastro, troca, rotação e exclusão

O cadastro aceita a chave em memória, cria o envelope e devolve somente
`KeyHandle`. Uma troca cria e valida o novo envelope antes de desativar o
anterior. A rotação da chave-mestra cria uma nova versão para escritas futuras;
um trabalhador recriptografa envelopes ativos de modo idempotente. Durante esse
processo, leituras aceitam versões ainda autorizadas.

Ao apagar, o serviço marca `deleting`, bloqueia novas aquisições, aguarda ou
cancela chamadas pendentes conforme política, destrói material cifrado e marca
`deleted`. A interface deve avisar que chamadas já aceitas pelo provedor não são
desfeitas. Retenção de backups e prazo máximo de purga são perguntas abertas;
até a purga, backups devem permanecer cifrados e inacessíveis ao caminho normal.

### Erros operacionais

| Situação | Resultado |
|---|---|
| Sem chave, chave inválida, removida ou desativada | `no_key`; usar T0 e informar estado sem expor detalhe do segredo. |
| Teto esgotado ou reserva recusada | `budget_exhausted`; usar T0 até a próxima janela diária configurada ou ajuste explícito. |
| Cofre/chave-mestra indisponível | Não decifrar; usar T0 e registrar falha operacional. |
| Timeout, rede ou erro do provedor | Liberar ou liquidar reserva conforme confirmação de envio; usar T0. |
| Saída malformada, injection ou intenção inválida | Descartar a saída, registrar motivo enumerado e usar fallback T0. |
| Conflito de concorrência na conta | Repetir a operação idempotente ou recusar a chamada; nunca ultrapassar o limite. |

T0 é o fallback universal exigido pelo ADR-0006. Se T1 estiver disponível pode
ser usado por sua própria política, mas não é requisito nem justificativa para
alterar a regra de T0. A indisponibilidade de IA degrada qualidade narrativa e
de planejamento, nunca impede o avanço do turno.

## Determinismo, logs e event sourcing

Criptografia, relógio UTC, custo, rede, orçamento e chamadas ao provedor são
I/O e ficam fora de `step`. Antes de `step`, a resposta externa válida é
normalizada em intenção e, depois da validação, em comando. O log de comandos
guarda o comando aceito, a origem (`player`, `t0`, `t1` ou `t2`), IDs de
catálogo/schema, `call_id` opaco e motivo de fallback quando houver. Um replay
reexecuta esses comandos sem consultar chave, cofre, orçamento ou IA.

Nunca entram no `step`, no comando gravado ou no hash de estado: API key,
material cifrado, chave de dados, chave-mestra, cabeçalho de autorização, corpo
de prompt, texto livre do jogador, texto integral do provedor, preço externo,
timestamp de rede, saldo diário ou resultado de decifragem. A eventual prosa
para Crônica é artefato externo e não pode mudar efeitos mecânicos.

O `SecurityAuditEvent` registra criação, troca, rotação, exclusão, aquisição
negada, reserva, liquidação, fallback e falha: código de resultado, IDs opacos,
versão de proteção e custo agregado quando aplicável. Logs operacionais usam os
mesmos campos permitidos; toda exceção é convertida em código antes de ser
emitida.

## Orçamentos propostos

Os valores numéricos ainda não foram decididos. A política deve ser configurável
por provedor, finalidade e jogador, com limites conservadores e visíveis na tela
de custo. O custo diário é do jogador cuja chave financia T2, em uma janela UTC
documentada; mudança de moeda, preço ou janela deve ser versionada e auditável.

| Recurso | Limite proposto | Ação ao exceder |
|---|---|---|
| Tempo de aquisição/decifragem | prazo curto, separado do turno | não chamar T2; T0 |
| Tempo de chamada T2 | timeout por finalidade | cancelar/encerrar, liquidar conforme evidência e T0 |
| Tokens de entrada e saída | teto por chamada e por finalidade | truncar apenas dados reconstruíveis; se insuficiente, T0 |
| Custo diário por jogador | teto configurável, reservado antes de chamar | recusar T2 até nova janela ou alteração explícita |
| Memória de segredo | uma chamada, buffer mínimo e descarte imediato | falhar fechado e T0 |
| Auditoria | eventos estruturados e tamanho limitado | omitir conteúdo, nunca omitir o código de falha |

O orçamento de tokens considera apenas informação canônica mínima e fatos
necessários. Texto livre é limitado por tamanho antes de entrar como bloco de
dados. Compactação e reset de memória seguem o SDD 08; perda de contexto não
autoriza aumentar automaticamente o teto de custo.

## Estratégia de testes

- Testes de contrato verificam que todos os adaptadores respeitam
  `KeyVaultPort`, `BudgetPort` e não conseguem serializar `EphemeralSecret`.
- Testes de propriedade geram sequências concorrentes de reserva/liquidação/
  liberação e verificam `settled + reserved <= limit`, idempotência por
  `call_id` e ausência de saldo negativo.
- Fixtures cifradas de teste usam apenas segredos sintéticos; cobrem troca de
  jogador/provedor, adulteração de metadados autenticados, versão de chave e
  rotação interrompida/retomada.
- Testes de exclusão comprovam que a chave não pode mais ser adquirida e que
  exportações de auditoria não contêm o segredo nem seu texto reversível.
- Testes de redação injetam chaves sintéticas em caminhos de erro, logs,
  métricas e traces e falham se qualquer uma aparecer fora da memória efêmera.
- Fixtures gravadas de IA incluem somente IDs, schema, intenção normalizada e
  resposta sanitizada; CI nunca chama provedor real nem necessita de chave.
- Harness de replay compara hash de estado com e sem falhas de cofre/provedor:
  dados os mesmos comandos gravados, o resultado deve ser idêntico. Testa-se
  também que nenhum campo de segurança altera o hash ou entra no `step`.
- Testes de prompt verificam que texto malicioso permanece em `UntrustedDataBlock`,
  não muda instruções confiáveis e uma saída que tente criar efeito mecânico sem
  schema é rejeitada.

## Perguntas abertas para o usuário

1. Quem fornece e paga a chave T1 do Jev/Runware: operador do servidor ou cada
   jogador? **Recomendação:** operador, com orçamento global separado e T0 como
   fallback; reduz atrito do BYOK e evita exigir duas chaves ao jogador.
2. Qual teto diário inicial, moeda de exibição e comportamento ao alcançá-lo?
   **Recomendação:** teto configurável pelo jogador, reserva prévia por chamada,
   exibição em moeda do provedor e bloqueio de T2 até a próxima janela UTC.
3. Qual provedor/cofre manterá a chave-mestra fora do banco e quem pode operá-lo?
   **Recomendação:** escolher um mecanismo de segredos com versionamento,
   auditoria e acesso mínimo antes de implementar armazenamento de chaves.
4. Por quanto tempo backups cifrados podem reter envelopes apagados e como será
   a purga verificável? **Recomendação:** definir prazo curto, documentar a
   retenção na interface e testar a remoção ao fim do prazo.
5. Uma chave BYOK vale para todos os provedores T2 compatíveis ou uma por
   provedor? **Recomendação:** uma chave por `(jogador, provedor)`; evita enviar
   segredo a endpoint não escolhido e facilita custo e revogação.
