# SDD 09 — Persistência

> **Status: proposta.** Este documento não aprova o GDD nem o ADR-0007. Os
> contratos e invariantes abaixo independem de tecnologia; referências a Rust,
> PostgreSQL ou armazenamento de objetos valem somente **se o ADR-0007 for
> aceito**.

## Objetivo e princípios

Persistência conserva a evidência necessária para o servidor autoritativo
(ADR-0001) recuperar, auditar e reproduzir um mundo sem chamar IA novamente.
Sua unidade de verdade é o comando aceito: `step` é puro e o histórico permite
reconstruir qualquer estado verificável (ADR-0006).

O subsistema preserva estes princípios:

- um turno fechado tem log ordenado, hash e versão de catálogo identificáveis;
- comandos e snapshots são imutáveis depois de publicados;
- o snapshot é aceleração de leitura, nunca substituto do log;
- dados mecânicos são separados de texto narrativo, telemetria e segredos;
- falhar em uma otimização de retenção não pode descartar a capacidade de replay.

## Responsabilidades e fronteiras

### É responsabilidade da Persistência

- receber uma tentativa já autenticada, validar sua forma e registrar a decisão
  de aceite ou rejeição junto da ordem atribuída pelo servidor;
- gravar os comandos aceitos, referências aos catálogos e artefatos de IA que
  explicam sua origem, para replay sem provedores externos;
- fechar turnos com `state_hash`, metadados de execução e, na cadência definida,
  snapshots verificáveis;
- recuperar um mundo por snapshot mais a aplicação do sufixo do log;
- versionar e aplicar migrações de catálogo e de armazenamento;
- reter, compactar, exportar e restaurar histórico com prova de integridade;
- produzir métricas operacionais sem expor conteúdo sensível.

### Não é responsabilidade da Persistência

- decidir regras, validar legalidade mecânica final ou executar `step`: isso é
  do núcleo de simulação;
- determinar ordem de bots, resolver combate, produção, crescimento, Entropia
  ou consolidação de memória; apenas registra suas entradas e resultados;
- gerar intenção, narrativa ou fallback T0, nem contactar LLM/modelos;
- autenticar o cliente, gerir sessões, notificar ou renderizar o mundo;
- ser fonte da verdade de memória de IA, Mandato, Crônica ou Ledger: esses são
  estado/catálogos do motor e podem constar em snapshots como dados do estado;
- armazenar API keys, prompts brutos que contenham segredos, IPs ou hostnames.

## Contratos propostos

Os tipos são pseudocódigo independente de linguagem. Campos `*_id` são
identificadores opacos estáveis; inteiros mecânicos usam representação definida
pelo núcleo, sem ponto flutuante.

```text
WorldRef { world_id, ruleset_version, seed_version, world_seed }

AcceptedCommand {
  command_id, world_id, turn, accepted_sequence, actor_id,
  origin: player | governor | bot | entropy | fallback | system,
  kind, payload_canonical, rules_version, catalog_version, grounding_facts
}

TurnSeal {
  world_id, turn, command_first_order, command_last_order,
  state_hash_algorithm, state_hash, ruleset_version, catalog_version,
  seed_version, resolver_version
}

Snapshot {
  snapshot_id, world_id, turn, state_hash, state_codec_version,
  compressed_state, command_last_order, catalog_version,
  created_sequence
}

IntentEvidence {
  evidence_id, command_id?, source, schema_version,
  request_fingerprint, response_canonical_json, validation_outcome,
  fallback_reason?, grounded_fact_refs, token_usage?, cost_microunits?
}
```

`payload_canonical` é canônico para hashing, mas sua semântica pertence ao
schema do comando. Uma tentativa rejeitada não vira `AcceptedCommand`; ela pode
gerar registro operacional mínimo, sem conteúdo do jogador, para diagnóstico.

```text
append_accepted(command: AcceptedCommand) -> AppendResult
seal_turn(seal: TurnSeal) -> void
store_snapshot(snapshot: Snapshot) -> void
load_replay(world_id, from_turn, to_turn) -> ReplayBundle
load_state(world_id, target_turn) -> StateRecoveryPlan
record_intent_evidence(evidence: PersistedIntentEvidence) -> EvidenceRef
verify_world(world_id, target_turn) -> VerificationReport
```

`append_accepted` deve ser idempotente por `(world_id, command_id)`. Para um
mesmo turno, somente o resolvedor autoritativo atribui `accepted_sequence`; o
contrato rejeita colisão ou lacuna na sequência ao fechar o turno.

`load_state` devolve o snapshot verificável mais recente em `turn <= target`
e todos os comandos posteriores ordenados. O chamador aplica `step`; a camada
de persistência não calcula estado durante leitura.

### Fluxo de fechamento proposto

```text
intent/tentativa -> validação do servidor -> AcceptedCommand(s) -> append
todos os presentes prontos + automações ordenadas pela seed
  -> resolver comandos e fases no núcleo -> state_hash -> TurnSeal
  -> snapshot se devido -> publicar turno fechado
```

A ordem de ações simultâneas é a ordem aceita pelo servidor; para bots e
Governadores, a ordem e a rotação são derivadas da seed do mundo. Persistência
grava o resultado dessa atribuição, não a recalcula.

## Modelo de dados relacional proposto

Se ADR-0007 for aceito, estas entidades podem ser tabelas PostgreSQL. Em outra
stack, são o esquema lógico mínimo, com as mesmas chaves e restrições.

| Entidade | Chave | Conteúdo e relações |
|---|---|---|
| `worlds` | `world_id` | seed, versões iniciais, status e turno fechado mais recente. |
| `world_turns` | `(world_id, turn)` | selo do turno, hash, intervalo de ordens e versões efetivas. |
| `command_log` | `(world_id, accepted_sequence)` | envelope imutável; `command_id` é único por mundo. |
| `snapshots` | `snapshot_id` | blob codificado, hash esperado e última ordem incorporada. |
| `catalog_releases` | `catalog_version` | manifesto imutável, hash e compatibilidade de regras. |
| `catalog_migrations` | `migration_id` | transição declarada, checksum, versões origem/destino e resultado. |
| `intent_evidence` | `evidence_id` | saída tipada/resultado de validação; referencia opcionalmente comando. |
| `archives` | `archive_id` | pacote histórico, intervalo, hashes e localizador abstrato. |
| `backup_runs` | `backup_id` | escopo, ponto consistente, hashes, restauração testada e retenção. |

Índices mínimos: `command_log(world_id, turn, accepted_sequence)`,
`snapshots(world_id, turn DESC)` e `world_turns(world_id, turn)`. Blobs grandes
podem ficar fora da base relacional se o localizador, hash criptográfico e
política de recuperação permanecerem transacionais com os metadados.

### Invariantes verificáveis

- `world_turns.turn` é contíguo desde zero; um selo só existe após todos os
  comandos daquele turno terem sido gravados.
- todo `command_log.turn` pertence a um turno selado ou ao único turno aberto;
  `accepted_sequence` é único globalmente no mundo, estritamente crescente e nunca é reusado.
- o hash de `world_turns` é calculado sobre a serialização canônica do estado,
  com algoritmo e versões declarados; deve igualar o hash ao fazer replay.
- um snapshot declara um `command_last_order` e seu hash deve igualar o selo do
  mesmo turno; não pode cobrir comando de turno posterior.
- `catalog_version`, `ruleset_version`, `seed_version` e `resolver_version`
  necessários para reproduzir um turno estão presentes no selo ou no mundo.
- catálogo publicado e migração aplicada são imutáveis; checksum divergente
  bloqueia carga, nunca é corrigido silenciosamente.
- a exclusão física de log só é permitida após arquivo íntegro, backup
  restaurável e regra de retenção satisfeita; no MVP recomendado, não excluir.
- uma evidência de IA não pode mudar o comando já aceito; sua referência é
  auditável, mas `step` depende somente de `AcceptedCommand`, seed, catálogo e estado.

## Catálogos e migrações

Catálogos são dados versionados de regras, eventos, tecnologias e demais
definições mecânicas. Cada release tem manifesto canônico e checksum. Um mundo
fixa a release de criação e registra explicitamente toda mudança futura.

Uma `CatalogMigration` propõe: versões origem/destino, pré-condições, função de
transformação determinística do estado e comandos, plano de reversão, checksum
e fixture de prova. Ela roda como operação administrativa fora de `step`, cria
snapshot antes/depois e grava uma entrada de auditoria. Não se sobrescreve um
catálogo já usado por um mundo.

Enquanto não houver contrato de migração aprovado, a recomendação é manter cada
mundo na sua release e publicar novos mundos em releases novas. Isso preserva
replay e evita transformar save antigo de modo implícito.

## Snapshots, retenção e compactação por era

Snapshot recomendado: no turno inicial, no fechamento de cada era e a cada
`N` turnos configurável. `N` é parâmetro operacional ainda aberto; o limiar
deve ser escolhido por medição de custo de recuperação, não por conveniência.

Ao fechar uma era (de 4 a 12 turnos), criar:

- snapshot de fronteira da era;
- arquivo imutável do intervalo de comandos, selos e versões de catálogo;
- resumo derivado para consulta humana, marcado como não autoritativo;
- cadeia de hashes que liga arquivo, selos e snapshot de fronteira.

Compactação reduz duplicação de snapshots e move blobs frios para arquivo, mas
não funde comandos em um evento sem preservar o log original ou uma exportação
verificável equivalente. A Crônica pode ser compactada como estado do motor;
texto narrativo derivado não substitui comandos para replay.

Política inicial proposta: histórico ativo e snapshots de eras recentes ficam
em armazenamento quente; eras antigas são arquivadas com índice de busca;
mundos eternos mantêm pelo menos uma cadeia completa restaurável. Os períodos
exatos, mídia e prazo de retenção são perguntas abertas.

## Falhas, fallbacks e determinismo

| Falha | Comportamento proposto |
|---|---|
| queda antes do selo | recuperar o último turno selado; reexecutar apenas o trabalho ainda não publicado. |
| duplicação de envio | deduplicar por `command_id`; devolver resultado já persistido. |
| snapshot corrompido | escolher snapshot anterior válido e reproduzir o log; alertar. |
| hash divergente | bloquear publicação/recuperação automática, preservar evidência e abrir incidente reproduzível. |
| IA timeout, inválida ou sem saldo | registrar evidência sanitizada e usar fallback T0 determinístico; gravar o comando resultante. |
| catálogo ausente/checksum divergente | não avançar nem migrar; restaurar release conhecida ou exigir intervenção. |
| backup inválido | não podar/arquivar definitivamente; marcar execução falha. |

Entra no log de comandos: ação validada, ator, origem, turno, ordem aceita,
payload canônico, versões requeridas e correlação de auditoria. Para IA, entra a
evidência tipada e sanitizada necessária à explicação e replay do comando, ou a
causa codificada do fallback.

Nunca entra em `step`: rede, I/O, relógio, UUID gerado na resolução, ordem de
iteração incidental, preço/latência de IA, tokens, texto livre não validado,
prompt, API key, resposta bruta de provedor ou telemetria. Esses dados podem ser
metadados externos e não podem influenciar o hash mecânico.

## Backup e recuperação de desastre

Todo backup é point-in-time consistente entre `world_turns`, log, snapshots,
catálogos e metadados de arquivo. Deve conter manifest, checksums, versão de
codec e instrução de restauração independente da implantação original.

Se ADR-0007 for aceito, a implementação deve usar os mecanismos de backup
consistente do PostgreSQL e copiar blobs referenciados pelo mesmo manifest. A
política proposta é cópia periódica automatizada, cópia antes de migrações e
teste de restauração em ambiente isolado. Sucesso significa restaurar um mundo
e verificar hash de um turno-alvo, não apenas concluir upload.

## Orçamento proposto

Os limites numéricos são parâmetros a medir antes de aprovação; o contrato mede
e expõe cada um por mundo e turno.

| Recurso | Limite/medida proposta | Ação ao exceder |
|---|---|---|
| fechamento do turno | orçamento configurável de I/O, separado de `step` | manter turno não publicado, recuperar transação; alertar. |
| recuperação | máximo de comandos desde snapshot configurável | gerar snapshot de manutenção fora da resolução. |
| memória | stream de log/snapshot; sem carregar mundo histórico inteiro | interromper arquivo e preservar dados. |
| disco | tamanho por mundo, era e classe quente/fria | compactar somente após backup verificado; alertar antes do teto. |
| IA | tokens, custo em microunits e latência como metadados | aplicar teto já decidido pelo subsistema de IA e cair para T0. |

Tokens e custo não pertencem ao estado nem ao hash; são medidos para cumprir o
teto diário do jogador e investigar regressões. A ausência dessa telemetria não
pode impedir o avanço com fallback determinístico.

## Estratégia de testes

- **Determinismo:** mesmo seed, catálogo e log produzem o mesmo hash em
  plataformas distintas; reabrir de qualquer snapshot válido produz o selo
  original.
- **Propriedades:** gerar sequências válidas/inválidas e verificar unicidade de
  ordem, contiguidade de turnos, idempotência, imutabilidade e hashes estáveis.
- **Golden replays:** fixtures gravadas de partidas curtas, combate, Entropia,
  entrada tardia e troca de era; CI compara hash de cada turno.
- **Fixtures de IA:** gravar intenção tipada, resposta inválida, timeout e
  fallback; CI não chama API real nem depende de preço/latência externos.
- **Migrações:** fixture por release, execução ida/volta quando suportada,
  checksum e replay antes/depois conforme contrato declarado.
- **Backup:** restaurar periodicamente uma cópia, reproduzir até turno-alvo e
  validar hash, catálogo e arquivos de era.
- **Harness longo:** milhares de turnos de bots, com cortes simulados durante
  append, selo, snapshot e arquivo; medir recuperação, disco e memória.

## Observabilidade e privacidade

Métricas agregadas: duração de append/selo/snapshot, bytes por mundo/era,
distância até snapshot, falhas de hash, idade do último backup testado e tokens/
custo por fonte de IA. Logs operacionais usam `world_id`, `turn`, `command_id`
e `correlation_id`, sem payload livre por padrão.

Qualquer inspeção de payload, narrativa ou evidência de IA requer trilha de
auditoria e política de acesso definida pelo futuro SDD de segurança. Segredos
nunca são persistidos neste subsistema.

## Perguntas abertas para o usuário

1. Qual RPO/RTO é aceitável para mundos eternos e qual mídia/local de backup é
   permitida? **Recomendação:** definir RPO de até um turno selado e testar uma
   restauração automática periódica antes de autorizar poda histórica.
2. Qual cadência máxima de replay é aceitável ao abrir um mundo? **Recomendação:**
   começar com snapshot no fim de toda era e medir `N` adicional em harness,
   sem fixar número antes dos dados.
3. Eras antigas podem ser apagadas após arquivo verificável ou devem permanecer
   recuperáveis indefinidamente? **Recomendação:** para MVP, conservar cadeia
   completa restaurável e usar apenas compactação sem perda.
4. Quando um catálogo novo deve migrar mundos existentes? **Recomendação:**
   manter mundos na release original até existir migração determinística,
   reversível e testada por fixtures.
5. Por quanto tempo guardar evidências sanitizadas de IA e narrativa, que não são
   necessárias ao `step`? **Recomendação:** retenção menor que o log mecânico,
   com prazo e controle de acesso definidos no SDD de segurança.
6. O ADR-0007 será aceito com PostgreSQL como implementação do esquema relacional?
   **Recomendação:** aprovar primeiro os contratos acima; ratificar a tecnologia
   somente após spike de backup, recuperação e custo na infraestrutura-alvo.

## Referências

- ADR-0001 — Servidor autoritativo.
- ADR-0006 — Motor determinístico com event sourcing; IA só propõe.
- GDD 01 — Loop e turnos.
- GDD 10 — Ciclo infinito e eras.
