# Revisão cruzada dos SDDs

## Inconsistências

| ID | Arquivos e seções | O que diverge | Proposta de correção | Precisa do usuário? |
|---|---|---|---|---|
| RC-01 | `sdd/02-hex-e-mapa.md`, todo o arquivo | O título e o texto contêm caracteres substitutos, por exemplo: `# SDD 02 ? Hex?gonos`, `n?o aprovado` e `Conven??es`. Os demais documentos estão em PT-BR legível. | Restaurar o conteúdo de `02` em UTF-8, preservando os caracteres PT-BR e LF; validar ausência de `?` no lugar de acentos. | não |
| RC-02 | `sdd/00-visao-geral.md` §Contratos principais; `01-nucleo-simulacao.md` §2.1; `03-turnos-e-sessao.md` §Contratos; `09-persistencia.md` §Contratos; `10-protocolo.md` §6 | O mesmo registro é chamado `AcceptedCommand`, `CommandEnvelope`, `AcceptedTurnCommand`/comando de sessão e `Command`; a ordem é `sequence`, `acceptedAt`, `accepted_order`, `accepted_at_sequence` e `acceptedSequence`. Alguns campos ainda duplicam o mesmo conceito no mesmo tipo (`accepted_order` e `accepted_at_sequence`). | Adotar `AcceptedCommand` como contrato canônico, com `command_id`, `world_id`, `turn`, `accepted_sequence` (único globalmente), `actor_id`, `origin`, `kind`, `payload_canonical`, versões e fatos de grounding. Deixar recibos de protocolo apenas espelharem `accepted_sequence`. | não — aplicada em 2026-10-01 |
| RC-03 | `sdd/04-camadas-de-ia.md` §Fluxo proposto e §Falhas; `sdd/09-persistencia.md` §Invariantes; ADR-0006 §Decisão | `04` diz que, no replay, o leitor usa a “`intenção normalizada registrada`” e “`passa novamente pela mesma validação`”. `09` afirma que a evidência de IA “não pode mudar o comando já aceito” e que `step` depende somente do comando. ADR-0006 decide que a entrada externa vira comando gravado. | Replay mecânico deve aplicar exclusivamente `AcceptedCommand` já gravado. `IntentEvidence`/resposta normalizada fica como evidência auditável e fixture, sem revalidar para decidir o efeito histórico. | não — aplicada em 2026-10-01 |
| RC-04 | `sdd/00-visao-geral.md` §Contratos e fluxo; `01-nucleo-simulacao.md` §2.2 e §4.1; `06-entropia.md` §Fluxo por turno; ADR-0006 §Decisão | `00` define `step` com comandos aceitos. `01` acrescenta `automation_order` e `entropy_inputs` ao `closed_turn_input`; `06` diz que “o motor executa” a avaliação de Entropia, mas também prevê seleção T1/T2. Isso torna ambíguo se uma chamada externa pode influir dentro de `step`. | Fixar `step(state, accepted_commands, seed, versions)` como única entrada mecânica. A orquestração calcula a ordem de automação e obtém/valida a escolha de Entropia antes do fechamento; o resultado entra como comando. Avaliação puramente determinística pode ser função interna, sem porta T1/T2. | não — aplicada em 2026-10-01 |
| RC-05 | `sdd/01-nucleo-simulacao.md` §5; `sdd/09-persistencia.md` §Snapshots; `sdd/00-visao-geral.md` §Modelo de dados | Divergência de cadência de snapshots. | Resolvida: política versionada no início, fim de cada era e a cada 50 turnos. Aplicada em 2026-10-01. | não — decidida em 2026-10-01 |
| RC-06 | `sdd/01-nucleo-simulacao.md` §2.1; `sdd/09-persistencia.md` §Invariantes; `sdd/14-testes.md` §Contratos | Divergência da origem do turno. | Resolvida: turno começa em 0. Aplicada em 2026-10-01. | não — decidida em 2026-10-01 |
| RC-07 | `sdd/05-governador.md` §Relatório, avisos e revolução; `gdd/06-sociedade-e-governo.md` §Decidido | `05` recomenda que revolução média “restringe escopos” e alta “substitui por `Recuperar`”; o GDD decidido determina que a moderada põe Mandato proposto do novo regime (reversível) e a severa impõe Mandato do novo regime. | Remover a tabela/recomendação incompatível. O SDD deve modelar os dois efeitos decididos; intensidades, catálogo e campos do Mandato do novo regime continuam em aberto. | não — aplicada em 2026-10-01 |
| RC-08 | `sdd/03-turnos-e-sessao.md` §Presença; `sdd/10-protocolo.md` §5 | Divergência da janela e registro de presença. | Resolvida: janela técnica de 60 s e presentes registrados na abertura do turno. Aplicada em 2026-10-01. | não — decidida em 2026-10-01 |
| RC-09 | `sdd/02-hex-e-mapa.md` §8; `sdd/10-protocolo.md` §9 | Divergência do limite de chunk. | Resolvida provisoriamente: 48 KiB até benchmark no Android; valor final pelo benchmark. Aplicada em 2026-10-01. | não — decidida em 2026-10-01 |
| RC-10 | `sdd/13-infra-e-deploy.md` §Topologia; `CLAUDE.md` §2.8 e `AGENTS.md` regra 5 | O SDD inclui literais de endereço de loopback e detalhes de binding/portas no repositório. Embora não sejam chaves, a regra do repositório veda IPs/hostnames em arquivos e manda usar placeholder para host. | Substituir endereço literal por uma descrição/placeholder neutro e manter a fonte operacional de portas fora do repositório, conforme a regra de infra. | não — aplicada em 2026-10-01 |

## Contratos compartilhados com nomenclatura divergente

| Conceito | Nomes encontrados | Nome canônico proposto |
|---|---|---|
| Intenção externa | `PlayerIntent`, `AIIntent`, `Intent`, `IntentEnvelope`, `GovernorIntent`, `DiplomaticIntent`, `EntropySelectionIntent` | `ActionIntent` com especializações por `kind`; o envelope contém `request_id`, ator, turno, versão e grounding. |
| Comando validado | `AcceptedCommand`, `CommandEnvelope`, `GovernorCommand`, `ValidatedCommand`, `Command` | `AcceptedCommand` |
| Ordem de aceitação | `sequence`, `acceptedAt`, `accepted_order`, `accepted_at_sequence`, `acceptedSequence` | `accepted_sequence` |
| Origem de comando | `player`/`Human`, `governor`/`Governor`, `bot`/`Bot`, `entropy`/`Entropy`, `system`/`System` | `CommandOrigin = player | governor | bot | entropy | fallback | system` (enum em minúsculas no payload). |
| Evidência de IA | `AiInvocation`, `RecordedAiResponse`, `PersistedIntentEvidence`, `replay_ref`, fixture | `IntentEvidence`; referenciada pelo comando, nunca substitui o comando no replay. |
| Fato de grounding | `FactRef`, `StateFactRef`, `FactId`, `LedgerRef`, `cited_fact_ids`, `accepted_facts` | `GroundingRef { kind, id, revision }`; `LedgerEntry` é um `kind`, não outro formato. |
| Evento resultante | `DomainEvent`, `Event`, `DiplomaticFact`, `ChronicleEventRef`, `LedgerEntry` | `DomainEvent` para fato emitido pelo motor; `LedgerEntry` é uma projeção/fato diplomático identificado por `DomainEventId` ou comando causal. |
| Hash de estado | `StateHash`, `Hash`, `state_hash`, `stateHash`, `expected_state_hash` | `StateHash`, serializado no campo `state_hash`; hashes de resposta/contexto têm tipos próprios. |
| Snapshot | `Snapshot`, `ProjectionSnapshot`, `WorldBootstrap`, `StateRecovery` | `WorldSnapshot` (autoritativo); `ClientViewSnapshot` (projeção), para impedir confusão entre replay e cache móvel. |
| Versão de regras | `rulesVersion`, `ruleset_ref`, `ruleset_version`, `resolver_version`, `schema_version` | `RulesetRef { id, version, content_hash }`; `schema_version` permanece somente do formato de mensagem/registro. |

## Lacunas

- **Catálogo e DSL mecânica — coberta por SDD 18** — `sdd/18` define gramática declarativa fechada, validação, versões, limites e ausência de execução de código arbitrário. Permanecem abertas as escolhas de formato-fonte, catálogo inicial e política de compatibilidade ali listadas.
- **Economia, cidades, tecnologia, sociedade e combate — coberta por SDD 15** — `sdd/15` estabelece proprietário, comandos, ordem de resolução, estado e invariantes para esses domínios. A fórmula de entrega proporcional de contratos mistos e os parâmetros de balanceamento continuam pendentes em `gdd/12` e no SDD.
- **Visibilidade/fog of war — coberta por SDD 16** — `sdd/16` define estados de conhecimento, observações, autorização de `GroundingRef`, compartilhamento de mapa e projeções. Custos de visão e atributos lembrados continuam em aberto no SDD.
- **Identidade, autorização e ciclo de vida de conta — coberta por SDD 17** — `sdd/17` define credenciais, associação usuário–civilização, revogação, múltiplos dispositivos e contratos de sessão.
- **Política de presença — coberta por SDD 17** — `sdd/17` define `PresencePolicy`, presença por usuário, múltiplos dispositivos, congelamento na abertura, reconexão, corte e auditoria de `Pronto`; a janela técnica decidida é 60 s.
- **Criação/entrada tardia e sucessão — coberta por SDDs 15 e 17** — `sdd/15` e `sdd/17` especificam os comandos, pré-condições, herança de estado/Ledger/Crônica e transição de controle já decidida; detalhes ainda marcados como proposta continuam nos próprios SDDs.
- **Retenção e eliminação de conteúdo não mecânico — coberta por SDD 19** — `sdd/19` centraliza classificação, prazo proposto, expurgo verificável, backup, exportação e exclusão; os prazos e as escolhas jurídicas/operacionais permanecem abertos nele.

## Violações dos invariantes do `CLAUDE.md` §2

| Invariante | Arquivos e seção | Violação ou risco concreto | Correção necessária |
|---|---|---|---|
| Motor determinístico / event sourcing | `sdd/04` §Fluxo proposto; `sdd/01` §2.2; `sdd/06` §Fluxo por turno | O replay pode revalidar intenção em vez de reproduzir comando (RC-03), e `closed_turn_input` pode transportar entradas de automação/Entropia fora do log canônico (RC-04). Isso permite mudança de regra/porta alterar um replay. | Aplicar apenas comandos aceitos no replay; registrar toda escolha externa como comando antes de `step`. |
| Motor determinístico | `sdd/06` §§Contratos/Fluxo | `PredicateSpec` e `TargetSelectorSpec` são supostos, mas não há DSL fechada nem semântica de ordenação/PRNG. Se forem código ou consultas não especificadas, podem introduzir I/O, ordem incidental ou resultados entre plataformas. | Definir catálogo declarativo, intérprete puro, limites de execução e ordenação canônica; rejeitar template fora do DSL. |
| IA só propõe | `sdd/06` §Fluxo por turno | A frase “o motor executa” a avaliação, junto de `EntropyPort` T1/T2, não separa inequivocamente a seleção externa da transição pura. | Separar funções puras de oportunidade/fallback da orquestração de portas; somente `ActivateEntropyEvent` validado chega a `step`. |
| Texto de jogador é dado não confiável | `sdd/04` §Texto não confiável; `07` §Negociação em dois modos; `08` §§Fontes/Fluxo | Os três SDDs exigem isolamento, mas nenhum define um contrato único de `UntrustedPlayerText` (origem, limite, normalização, retenção e proibição de promoção a `Fact`/`Rule`). A passagem por memória e Crônica fica dependente de interpretação local. | Criar tipo e regras compartilhadas; permitir somente blocos delimitados e referências, e impedir que consolidação o eleve a documento canônico sem fato validado. |
| Segredos nunca em repositório/log | `sdd/13` §Topologia | Há literal de rede/portas no documento, contrariando a regra adicional do repositório sobre IPs/hostnames; o risco não é uma API key, mas a regra de higiene já definida. Não foram encontrados segredos de BYOK nos contratos revisados. | Aplicar RC-10 e manter os SDDs em placeholders/documentação de contrato. |

## Revisão 2 (SDDs 15–18)

| ID | Arquivos | Divergência | Correção | Precisa do usuário? |
|---|---|---|---|---|
| RC2-01 | `sdd/15-regras-de-dominio.md` §4.2; `gdd/12-variaveis-e-formulas.md` §Pressão de crise `P` e §Coesão `C` | As expressões de `P_c` e `C'` no SDD reduziam as fórmulas compartilhadas a termos genéricos e não explicitavam o arredondamento inteiro de `P`. | Alinhar ambas às fórmulas do GDD 12, preservar os termos limitados por turno e documentar o arredondamento. Aplicada em 2026-10-01. | não |
| RC2-02 | `sdd/17-identidade-e-contas.md` §3; tabela de contratos canônicos acima | `AcceptedCommand` usava o campo `ruleset`, enquanto a nomenclatura canônica é `ruleset_ref: RulesetRef`. | Renomear o campo no contrato para `ruleset_ref`. Aplicada em 2026-10-01. | não |
| RC2-03 | `sdd/15-regras-de-dominio.md` §2 e §4; `sdd/17-identidade-e-contas.md` §2 | Sucessão/entrada tardia e efeitos de revolta seguem sem regras mecânicas completas; os SDDs delimitam a coordenação, mas não especificam todos os comandos e transições já decididos no GDD. | Proposta escrita em 15 e 17; decisões existentes do GDD foram especificadas e detalhes não decididos permanecem marcados como proposta. | não |
| RC2-04 | `sdd/16-visibilidade.md` §5; `sdd/10-protocolo.md` §9; `sdd/02-hex-e-mapa.md` §8 | Limite de chunk divergente entre documentos. | Resolvido: 48 KiB até benchmark no Android; valor final pelo benchmark. Aplicado em 2026-10-01. | não — decidido em 2026-10-01 |

## Revisão 3 (SDDs 19–21 e catálogos)

| ID | Arquivos | Divergência | Correção | Precisa do usuário? |
|---|---|---|---|---|
| RC3-01 | `sdd/17-identidade-e-contas.md` §4; RC-08 e decisões da revisão | `PresencePolicy.reconnect_grace_ms` estava em 45 s, mas a decisão registrada fixa janela técnica de reconexão em 60 s. | Alterar para `60_000` ms. Aplicada em 2026-10-01. | não |
| RC3-02 | `sdd/21-notificacoes.md` §FCM e privacidade | O exemplo de payload usava `category: "decision_needed"`, valor ausente de `NotificationCategory`, cujo contrato define quatro categorias específicas. | Usar `critical_choice` no exemplo e declarar que `category` segue `NotificationCategory`. Aplicada em 2026-10-01. | não |
| RC3-03 | `data/catalogs/*.json`; `sdd/18-dsl-catalogos.md` §3.1–3.3 | Os catálogos-rascunho são separados por domínio e não trazem, por template, todos os metadados do contrato de publicação proposto (`revision`, parâmetros, limites, hash e versão mínima do motor). O formato-fonte e a compatibilidade ainda estão abertos no SDD 18. | Não aplicar: definir antes se esses metadados ficam em cada arquivo, em manifesto gerado ou em outro pacote publicado; então alinhar validador e dados. | sim |
| RC3-04 | `sdd/06-entropia.md` §§Contratos/Templates especiais; `data/catalogs/event_templates.json` | SDD 06 nomeia a categoria `terrain_change`; o catálogo usa `terrain`. Os verbos de interferência também diferem (`redirect`/`invoke` no SDD e `divert` nos dados). Nenhuma das formas foi decidida. | Não aplicar: escolher os enums canônicos de categoria e interferência, depois sincronizar SDD, catálogo e validador. | sim |

## Decisões do usuário sobre itens "sim" (2026-10-01)

| Item | Decisão |
|---|---|
| RC-05 snapshots | No início do mundo, em cada fim de era e a cada 50 turnos (`SnapshotPolicy` versionada). |
| RC-06 origem do turno | O turno começa em **0**. |
| RC-08 presença | Janela técnica de reconexão de **60 s** (não é relógio de jogo); lista de presentes registrada na abertura do turno. |
| RC-09 / RC2-04 chunk | **48 KiB** até haver benchmark no Android; valor final definido pelo benchmark. |
| GDD 04/06 migração | Leis **podem restringir** a migração automática. |

<!-- encoding-check: quotes damaged text -->
