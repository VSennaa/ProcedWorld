# SDD 19 — Retenção e eliminação de dados

> **Status: proposta para revisão do SDD.** Este documento fecha a política única
> de classificação, retenção, exportação e expurgo. Não altera decisões mecânicas
> do GDD, ADR-0001, ADR-0002 ou ADR-0006. Os prazos e mecanismos assinalados como
> proposta dependem de aprovação do usuário e de avaliação jurídica/operacional.

## Objetivo e princípios

Este subsistema define por quanto tempo cada classe de dado existe, como ela é
exportada, apagada e verificada, e o que a exclusão de uma conta pode ou não
remover. Ele separa a história mecânica de um mundo do conteúdo operacional,
narrativo e pessoal que não é necessário para executar `step`.

Os princípios são:

- replay mecânico aplica exclusivamente `AcceptedCommand` já gravado, com seed,
  `RulesetRef`, catálogos e `WorldSnapshot` compatíveis; não chama IA e não lê
  texto livre, memória, telemetria ou `IntentEvidence` para decidir efeito;
- retenção menor de conteúdo não mecânico nunca pode destruir a capacidade de
  recuperar e verificar o mundo;
- todo expurgo é assíncrono, idempotente, auditável e verificável sem preservar
  o conteúdo apagado;
- backups obedecem à mesma classificação: não são uma exceção silenciosa à
  exclusão;
- chaves BYOK, tokens de sessão e outros segredos nunca entram em exportações,
  logs, snapshots ou evidências.

Não há novas variáveis mecânicas neste SDD. Em particular, os valores canônicos
`C`, `L`, `S`, `D`, `G`, `W`, `E`, `P`, `Cf`, `R`, `Dv`, `A` e `U` permanecem os
definidos em GDD 12 e não são alterados por retenção ou expurgo.

## Responsabilidades e fronteiras

### É responsabilidade deste subsistema

- classificar artefatos, atribuir política e registrar a base de retenção;
- agendar retenção, suspensão legal/operacional quando autorizada e expurgo;
- produzir exportações autenticadas e com escopo explícito;
- coordenar tombstone, remoção de conteúdo, purga de backups e prova de execução;
- preservar os metadados mínimos para demonstrar que uma solicitação foi atendida;
- expor métricas agregadas de filas, atrasos e falhas de expurgo.

### Não é responsabilidade deste subsistema

- alterar `step`, regras de domínio, hashes de estado, ordem `accepted_sequence`
  ou decidir quais comandos são válidos;
- reter uma resposta de IA como requisito de replay: ela é `IntentEvidence`, não
  substituto de `AcceptedCommand`;
- interpretar texto de jogador, promovê-lo a fato ou moderá-lo semanticamente;
- decifrar, revelar ou incluir segredo em exportação; o ciclo criptográfico de
  BYOK continua no SDD 11;
- definir obrigação legal, jurisdição, prazo legal de guarda ou destino físico de
  backup. Essas escolhas ainda não foram decididas pelo projeto.

## Classificação e política proposta

`RetentionClass` e os prazos abaixo são contratos **propostos**. “Vida do mundo”
termina apenas quando houver exclusão administrativa de mundo aprovada; apagar
uma conta não é apagar a história compartilhada daquele mundo.

| Classe | Dados incluídos | Necessidade | Retenção proposta | Exportação e expurgo |
|---|---|---|---|---|
| `mechanical` | `AcceptedCommand`, `TurnSeal`, `WorldSnapshot`, seed, catálogos, `RulesetRef`, hashes e arquivos de era | replay e integridade | vida do mundo; sem poda no MVP | exportação de mundo autorizada; nunca apagar por exclusão de conta |
| `player_text` | `UntrustedPlayerText`: mensagem, negociação e entrada livre do jogador | experiência/auditoria curta; nunca mecânica | 30 dias após uso ou até remoção solicitada, o que ocorrer primeiro | exportar ao titular; remover corpo e manter recibo sem conteúdo |
| `ai_evidence` | resposta sanitizada, `IntentEvidence`, pacote de contexto, referências e motivo de fallback | auditoria e fixture, não replay | corpo 30 dias; referências, hashes e resultado de validação 365 dias | exportar conteúdo atribuível quando disponível; expurgar corpo sem tocar no comando |
| `chronicle_memory` | Crônica derivada, `MemoryEntry`, `Doctrine`, itens supersedidos e contexto conversacional | apresentação e qualidade de IA | Crônica enquanto o mundo existir; memória supersedida e contexto bruto 90 dias | exportar visão autorizada; reconstruir Crônica de fatos; apagar artefatos elegíveis |
| `byok_secret` | `KeyEnvelope`, material cifrado e chave de dados | habilitar T2 | enquanto `active`; destruição imediata ao pedido | jamais exportar; destruir cifrado/chave de dados e bloquear uso |
| `telemetry` | métricas agregadas de custo, tokens, latência, erro e expurgo | operação e orçamento | agregada 90 dias; evento detalhado sem conteúdo 30 dias | exportar somente dados atribuíveis ao titular; expurgo por janela |
| `backup` | cópias consistentes de qualquer classe anterior | recuperação | mesma classe de origem, com purga máxima proposta de 30 dias após elegibilidade | não exportar diretamente; purgar conforme manifesto e emitir prova |

`UntrustedPlayerText` é um contrato proposto, comum aos SDDs 04, 07, 08 e 11:

```text
UntrustedPlayerText {
  text_id, author_user_id?, world_id, created_at, purpose,
  normalized_content, content_hash, byte_length, retention_class
}
```

O conteúdo é limitado, normalizado e entregue à IA somente como bloco de dados
delimitado. `text_id` e `content_hash` podem ser citados por auditoria; o texto
não pode se tornar `GroundingRef`, `DomainEvent`, `Fact`, `Decision` ou `Rule`
sem um fato mecânico validado e independente.

## Contratos propostos

```text
RetentionPolicy {
  policy_version, class, retain_until_rule, backup_purge_deadline,
  export_allowed, delete_scope, hold_allowed
}

RetentionRecord {
  record_id, subject_kind, subject_id, retention_class, policy_version,
  created_at, eligible_at, state: active | held | pending_purge | purged,
  content_hash?, backup_manifest_refs[]
}

DeletionRequest {
  request_id, requester_user_id, scope: player_text | ai_content |
  account | byok_key, requested_at, status
}

DeletionReceipt {
  request_id, completed_at, outcome, purged_record_count,
  pending_backup_count, verification_refs[], exceptions[]
}

ExportRequest { request_id, requester_user_id, scope, format_version, expires_at }
ExportManifest { export_id, scope, created_at, content_hashes[], omissions[], integrity_hash }
```

`RetentionPort.classify(subject) -> RetentionRecord`,
`RetentionPort.request_deletion(request) -> DeletionReceipt | Pending`,
`RetentionPort.run_purge(now, policy_version) -> PurgeReport` e
`RetentionPort.verify(receipt_id) -> VerificationReport` são portas propostas.
O relógio usado para prazo é operacional e jamais entra em `step`.

Uma suspensão (`held`) só pode ser criada por processo administrativo autorizado,
com motivo enumerado, escopo mínimo e revisão. Ela bloqueia purga, não leitura
indiscriminada, e precisa aparecer no recibo de exclusão. Não se assume neste SDD
que qualquer suspensão seja necessária.

## Fluxo de expurgo verificável

1. A criação classifica o artefato e grava `RetentionRecord` sem alterar o mundo.
2. Vencido o prazo ou recebida solicitação válida, o registro vira `pending_purge`.
3. O trabalhador remove o corpo de armazenamento ativo; para BYOK, bloqueia
   aquisição antes de destruir o cifrado e a chave de dados.
4. Para cada backup que possa conter o corpo, registra o manifesto afetado e a
   data-limite de purga. O backup não é reutilizado para restauração normal após
   sua própria data-limite sem ser reescrito ou eliminado.
5. O trabalhador recalcula a lista de objetos, confere hashes/ausência e grava
   prova sem conteúdo: IDs opacos, classe, política, resultado e referências de
   manifesto. O registro torna-se `purged` apenas quando o ativo e os backups
   elegíveis estiverem tratados, ou fica `Pending` com motivo explícito.

Para conteúdo cujo corpo foi removido, `content_hash` pode permanecer somente se
for necessário para provar a execução e não permitir reconstruir o texto. A
política proposta é apagar também esse hash no mesmo prazo da evidência mínima,
salvo suspensão autorizada.

## Replay, Crônica, memória e backups

O log mecânico não pode perder determinismo. Logo, expurgar texto de jogador,
resposta de IA, pacote de contexto ou Crônica não muda o resultado de replay:
o leitor recupera `WorldSnapshot` e aplica os `AcceptedCommand` em
`accepted_sequence`, verificando o `StateHash` de cada `TurnSeal`.

`IntentEvidence` pode explicar por que um comando existiu, mas seu expurgo não
autoriza revalidar ou substituir o comando histórico. A Crônica é derivada de
`DomainEvent`, log e estado; se sua prosa for apagada ou corrompida, ela é
regerada pelos fatos autorizados, sem incluir `UntrustedPlayerText` removido.
Memória supersedida pode ser eliminada conforme a tabela; a perda reduz contexto
de IA, nunca altera `C`, `P` ou qualquer outro estado mecânico.

Todo backup tem `BackupManifest { backup_id, created_at, records[],
integrity_hash, restoration_tested_at?, policy_version }`. O manifesto identifica
classes e políticas, não introduz conteúdo em logs operacionais. Restauração usa
somente backups ainda elegíveis; antes de expirar, uma restauração deve reaplicar
tombstones concluídos desde o ponto do backup antes de disponibilizar dados.
Falha de backup ou de sua restauração bloqueia poda de dados `mechanical`, mas
não justifica reter indefinidamente conteúdo já elegível sem registrar pendência.

## Exportação e exclusão de conta

Uma exportação é autenticada, limitada ao titular e disponível por prazo curto
**proposto**. O pacote contém manifest verificável, dados classificados que ainda
existem e uma lista explícita de omissões: segredo BYOK, tokens/sessões, dados de
outros jogadores, texto já expurgado, telemetria não atribuível e fatos ocultos
pela autorização do mundo. A exportação de `mechanical` compartilhado requer
autorização de mundo separada; exclusão de conta nunca concede esse direito.

Exclusão de conta é proposta como a sequência abaixo:

1. revogar sessões e marcar a conta `pending_deletion`;
2. apagar imediatamente as chaves BYOK e impedir novas chamadas T2;
3. oferecer exportação enquanto os dados ainda estiverem disponíveis;
4. apagar ou pseudonimizar dados de conta e conteúdo pessoal elegível;
5. preservar `AcceptedCommand`, `WorldSnapshot`, Ledger e Crônica mecânica do
   mundo, substituindo a conta por civilização sob bot, conforme a recomendação
   já registrada no SDD 17;
6. concluir a purga de backups e emitir `DeletionReceipt` verificável.

O identificador histórico do ator em `AcceptedCommand` não é reatribuído nem
reescrito: ele preserva causalidade do mundo. A projeção de identidade que liga
esse identificador à conta é apagada/pseudonimizada segundo a política proposta.
Se o usuário cancelar a exclusão antes do ponto irreversível, a reversibilidade e
o prazo são perguntas abertas; nenhuma reversão deve restaurar chave BYOK apagada.

## Invariantes verificáveis

1. Nenhum expurgo de classe não mecânica altera `AcceptedCommand`, `TurnSeal`,
   `WorldSnapshot`, catálogo, seed, `StateHash` ou `accepted_sequence`.
2. O replay de um mundo antes e depois de expurgo produz os mesmos hashes.
3. Dados de `byok_secret` não aparecem em `ExportManifest`, exportação, backup
   lógico acessível, telemetria, `IntentEvidence` ou recibo de eliminação.
4. Todo corpo removido possui estado terminal `purged` ou pendência com motivo,
   política e próxima ação; falha silenciosa é inválida.
5. Um backup restaurado não pode reintroduzir conteúdo cujo tombstone já estava
   concluído no momento da restauração.
6. Texto de jogador mantém classe `player_text` e origem não confiável em todas
   as cópias; sua retenção não o promove a fato canônico.
7. A exclusão de uma conta revoga seu acesso e BYOK antes da remoção assíncrona,
   mas não remove causalidade mecânica de civilizações compartilhadas.
8. Telemetria de retenção usa IDs opacos e valores agregados; não inclui payload,
   prompt, resposta bruta, segredo ou identificador de rede.

## Falhas e tratamento

| Falha | Tratamento proposto | Efeito no jogo |
|---|---|---|
| objeto ativo ausente antes da purga | registrar idempotentemente como removido e verificar réplicas/backups | nenhum |
| backup não regravado até o prazo | manter solicitação `Pending`, bloquear restauração normal e alertar operação | nenhum no `step` |
| manifesto ou prova inconsistente | não declarar expurgo concluído; preservar metadados e reexecutar | nenhum |
| exclusão em corrida com chamada T2 | estado `deleting` bloqueia nova aquisição; cancelar/encerrar conforme política do SDD 11 | T0 para chamadas futuras |
| exportação falha ou expira | não altera pedido de exclusão; permitir nova solicitação autenticada | nenhum |
| tentativa de apagar dado mecânico por conta | rejeitar e explicar o escopo compartilhado | replay preservado |
| restauração de backup com tombstone posterior | reaplicar tombstone antes de disponibilizar a cópia | nenhum |

## Estratégia de testes

- **Classificação:** fixtures de cada classe, inclusive `UntrustedPlayerText`,
  garantem política, escopo de exportação e ausência de promoção a fato.
- **Replay:** executar golden replay antes/depois de expurgar corpo de IA,
  Crônica, memória e texto; comparar cada `StateHash` e `accepted_sequence`.
- **Expurgo:** testar repetição, interrupção e retomada de cada etapa; confirmar
  que recibo só fica concluído após ativo e manifestos de backup elegíveis.
- **BYOK:** usar segredo sintético e provar indisponibilidade após exclusão,
  ausência em exportações/backups lógicos e fallback T0 para nova chamada.
- **Backups:** restaurar cópia anterior a um tombstone, reaplicá-lo e procurar o
  conteúdo sintético; validar também hash do mundo recuperado.
- **Conta:** excluir conta com civilização ativa, provar revogação de sessão,
  transição para bot e preservação do log mecânico sem vínculo de identidade.
- **Segurança:** procurar texto, prompts e segredos sintéticos em telemetria,
  recibos, manifestos e logs; qualquer vazamento falha o pipeline.

## Perguntas abertas para o usuário

1. Os prazos propostos de 30 dias para texto/corpo de IA, 90 dias para memória
   supersedida/telemetria agregada e 365 dias para evidência mínima são aceitáveis?
   **Recomendação:** aprová-los apenas como configuração versionada inicial,
   revisável por telemetria e requisitos jurídicos, nunca como constante de código.
2. A Crônica deve existir por toda a vida do mundo ou jogadores podem apagá-la de
   sua visão pessoal?
   **Recomendação:** manter a Crônica derivada do mundo enquanto ele existir e
   permitir ocultar/apagar somente projeções pessoais não mecânicas.
3. Qual prazo máximo de purga para backups e qual disponibilidade de recuperação
   é aceitável durante esse prazo?
   **Recomendação:** 30 dias como máximo inicial, com manifesto e teste de
   restauração; não autorizar poda do log mecânico no MVP.
4. A exclusão de conta deve ter período de cancelamento antes da pseudonimização?
   **Recomendação:** oferecer um curto período configurável para cancelar a conta,
   mas destruir BYOK imediatamente e de modo irreversível ao confirmar o pedido.
5. Quais suspensões de expurgo são legítimas e quem pode autorizá-las?
   **Recomendação:** nenhuma por padrão; se necessárias, usar escopo mínimo,
   motivo enumerado, prazo e visibilidade no recibo de exclusão.

## Referências

- [ADR-0001 — Servidor autoritativo](../adr/0001-servidor-autoritativo.md)
- [ADR-0002 — Governador bot no servidor](../adr/0002-governador-bot-no-servidor.md)
- [ADR-0006 — Motor determinístico e event sourcing](../adr/0006-motor-deterministico-event-sourcing.md)
- [SDD 08 — Memória e contexto](08-memoria-e-contexto.md)
- [SDD 09 — Persistência](09-persistencia.md)
- [SDD 11 — Segurança e chaves](11-seguranca-e-chaves.md)
- [SDD 17 — Identidade e contas](17-identidade-e-contas.md)
- [GDD 12 — Variáveis e fórmulas compartilhadas](../gdd/12-variaveis-e-formulas.md)
