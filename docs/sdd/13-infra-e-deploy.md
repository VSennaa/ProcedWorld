# SDD 13 — Infra e deploy

> **Status: proposta para revisão do SDD.** GDD aprovado em 2026-10-01 e ADR-0007 aceito.
> Rust, Godot 4, PostgreSQL, Docker Compose e GitHub Actions compõem a stack decidida.

## Objetivo

Operar uma instância auto-hospedável do servidor autoritativo (ADR-0001), com
implantação reprodutível, recuperação verificável e capacidade compatível com o host
de referência: 2 vCPU, 2 GB de RAM, 30 GB de disco e sem GPU. Infra entrega artefato,
dependências operacionais e evidências; não decide regras do jogo.

## Responsabilidades e fronteiras

### É responsabilidade deste subsistema

- Empacotar uma versão imutável e declarar configuração, segredos e armazenamento.
- Publicar o protocolo do servidor só por proxy reverso com TLS; manter dependências
  internas fora da internet.
- Orquestrar inicialização, saúde, atualização, rollback, backups e retenção.
- Coletar logs estruturados, métricas e alertas sem segredos ou conteúdo sensível.
- Executar build, verificações e matriz de determinismo no CI; promover artefatos por tag.
- Provar que backups restauram log de comandos, snapshots e metadados para servir partida.

### Não é responsabilidade deste subsistema

- Implementar `step`, regras, PRNG, hash, ordem de turno, comandos ou balanceamento.
  Fórmulas `P`, `C`, `D`, `G`, `W` e `E` pertencem ao núcleo e seguem
  `docs/gdd/12-variaveis-e-formulas.md`.
- Decidir ações de Governador, Entropia, diplomacia ou IA, nem substituir seus fallbacks T0.
- Interpretar/descriptografar API key de jogador, ou transferi-la ao cliente.
- Ser fonte de verdade do mundo: cache, health e dashboards são derivados e recuperáveis.
- Definir UX mobile ou a semântica do protocolo cliente-servidor.

## Topologia proposta

Cada ambiente terá uma stack Docker Compose. Só o proxy fica
na rede pública; servidor e banco usam rede interna. Na VPS, a stack fica em
`/opt/stacks/procedworld/`; o clone de trabalho permanece separado em `~deploy/ProcedWorld`.

```text
Internet --TLS--> reverse proxy --internal--> authoritative server
                                      |              |
                                      |              +--> command log / snapshots
                                      +--> PostgreSQL (internal only)
CI --versioned image--> registry --> deploy host --pull--> Compose
backup runner <--- database and snapshot manifests
```

O proxy é o único processo que expõe 80/443. Reservas: servidor/API em
`<LOOPBACK>:8100`, admin/health em `<LOOPBACK>:8101`, harness em `<LOOPBACK>:8110` e
memória, se separada, em `<LOOPBACK>:8150`. Toda reserva pertence à faixa operacional definida fora do repositório e
deve entrar em `/opt/infra/PORTS.md` antes da publicação. PostgreSQL não terá `ports:`.

```yaml
# Esquema conceitual; não é arquivo de deploy.
services:
  proxy:
    networks: [proxy, procedworld_internal]
    ports: ["80:80", "443:443"]
  server:
    image: registry/procedworld-server:${RELEASE_ID}
    networks: [procedworld_internal]
    read_only: true
  database:
    networks: [procedworld_internal]
    # sem ports:
networks: {proxy: {external: true}, procedworld_internal: {internal: true}}
```

Serviço não público usa bind de loopback, nunca publicação direta de porta, pois
Docker pode contornar firewall do host. Proxy e emissor de TLS permanecem intercambiáveis.

## Contratos operacionais

Os contratos usam identificadores opacos e serialização canônica versionada. Podem ser
expostos por HTTP, arquivo ou fila sem mudar seu significado.

```text
ReleaseManifest {
  release_id: String, protocol_version: String, engine_version: String,
  image_digest: String?, catalog_hash: Hash, migration_version: String,
  created_at: Timestamp, source_revision: String
}

HealthReport {
  status: "ready" | "degraded" | "unready", release_id: String,
  protocol_version: String, database: Check, storage: Check,
  last_completed_turn: TurnId?, reason_code: String?
}

BackupManifest {
  backup_id: String, created_at: Timestamp, release_id: String,
  database_snapshot: ObjectRef, command_log_range: [LogSeq, LogSeq],
  game_snapshot_refs: [ObjectRef], checksums: [Hash], restore_verified_at: Timestamp?
}
```

`image_digest` é opcional para não acoplar o contrato a contêiner; se presente,
identifica bytes imutáveis. `catalog_hash`, `engine_version` e `protocol_version`
permitem recusar combinação incompatível antes de aceitar tráfego.

```text
deploy(manifest) -> DeployResult
  pre: manifesto validado e artefato disponível
  post: ready serve manifest.release_id, ou o release anterior continua ativo

backup(scope) -> BackupManifest
  post: objetos referidos existem, têm checksum e podem ser restaurados

restore(manifest, isolated_target) -> RestoreReport
  post: sequência de log, hashes e metadados coincidem com o manifesto
```

Promoção não constrói código no host. CI cria/publica a imagem;
a VPS verifica digest, baixa e inicia. Build em contêiner na VPS fica apenas para
desenvolvimento/diagnóstico, sem concorrer com a partida: o spike mostrou compilação
apertada em 2 GB. Swap, se necessário, requer decisão explícita do usuário.

## Dados persistentes e invariantes

PostgreSQL guarda estes
registros transacionais. Volumes e cópias externas são réplicas por `ObjectRef` e checksum.

| Registro | Chave/versão | Retenção e uso |
|---|---|---|
| `release_manifest` | `release_id` imutável | auditoria de promoção e compatibilidade |
| `deployment_event` | sequência monotônica | tentativa, resultado e rollback |
| `command_log` | `world_id`, `log_seq` | fonte de replay; nunca reordenar/sobrescrever |
| `game_snapshot` | mundo, turno, hash | acelera recuperação; deriva do log |
| `backup_manifest` | `backup_id` imutável | prova de completude e restauração |
| `ai_usage_ledger` | civilização, janela, chamada | custo/fallback; sem segredo ou prompt bruto |
| `operational_audit` | sequência monotônica | mudanças sem valores secretos |

Invariantes verificáveis:

1. Um `release_id` aponta a uma combinação única de versões e hashes; deploy não troca
   artefato mantendo o mesmo identificador.
2. `command_log.log_seq` cresce estritamente por mundo; registros aceitos são append-only.
   Snapshot inclui `log_seq` e `state_hash` correspondentes.
3. Restore começa isolado, verifica checksums e só substitui alvo após aprovação operacional.
   Falha preserva o alvo existente.
4. Banco, volumes e backups têm acesso restrito; API keys, tokens, senhas, cabeçalhos e
   conteúdo autenticado não entram em manifests, métricas ou logs.
5. Versão só recebe tráfego após `HealthReport.status == "ready"` e checks declarados.
6. Ledger de custo é observacional: teto de IA seleciona fallback, mas não edita mundo
   nem impede `step`.

## Falhas, fallback e determinismo

Infra nunca participa de `step(estado, comandos, seed)`. Não entram no `step`: relógio
de deploy/log, reinício, ordem de requisições, latência, CPU/RAM, endereço de rede,
disponibilidade de banco, resposta de IA, cache, TLS, proxy ou health check. Comando
aceito e catálogo aplicável já precisam estar no log antes de `step`.

| Falha | Comportamento proposto | Evidência |
|---|---|---|
| Proxy/TLS indisponível | rejeita tráfego; não altera mundo | código, janela, release |
| Banco indisponível | `unready`; não confirma comando novo; retoma depois | erro classificado, sem URI |
| IA lenta/inválida/sem teto | camada IA grava resultado e usa T0 determinístico | `call_id`, custo/tokens agregados, fallback |
| Imagem inválida/health falha | não promove; mantém release anterior | digest, etapa, motivo |
| Deploy interrompido | reconcilia para release anterior ou alvo declarado | evento e estado final |
| Backup corrompido | não elegível a restore; alerta e nova cópia | objeto, checksum, motivo |
| Pressão de recurso | limita jobs auxiliares, degrada e alerta | uso agregado e limiar |

Logs estruturados usam `request_id`, `world_id` quando autorizado, `turn_id`, `log_seq`,
`release_id`, `engine_version`, `catalog_hash`, erro, duração e contadores agregados.
Nunca incluem API key, token, senha, URI credenciada, corpo de autenticação, prompt/resposta
brutos, IP, hostname ou texto de jogador/narrativa. Para replay: hash anterior/posterior,
seed e versão, ids/ordem de comandos aceitos, catálogo e motor. Infra apenas registra
evidência: não fornece aleatoriedade nem ordenação.

## Orçamento e capacidade

Valores são limites iniciais a medir no harness antes de se tornarem gate vermelho. A
prioridade da VPS é servir e preservar dados, não compilar.

| Recurso | Proposta inicial | Ao exceder |
|---|---|---|
| Turno | medir p50/p95; orçamento inicial 1 s p95 no host | alerta, reduzir tarefas auxiliares, investigar |
| API | medir p50/p95 separado do turno | degradar endpoint não essencial |
| RAM | alerta 75%, crítico 85% da disponível | parar harness/backup concorrente |
| CPU | no máximo um build ou harness pesado | serializar trabalho auxiliar |
| Disco | alerta 70%, crítico 85% | bloquear promoção sem espaço seguro |
| IA | tokens/custo por chamada, civilização e janela | T0 em teto, timeout ou falha |

IA é orçamento do jogador; infraestrutura é do operador. Escassez de CPU, memória ou
rede nunca pode virar resultado de jogo. T0 continua sem chave, como decidido no GDD 11.

## CI, release e deploy

GitHub Actions em runners limpos deve:

1. Validar formatação, schemas, manifests e ausência de segredos conhecidos.
2. Compilar/testar núcleo e servidor, com propriedades e fixtures sem rede.
3. Rodar matriz Linux x86_64, Windows x86_64 e macOS ARM64 com seed, catálogo e replay
   iguais; comparar hashes por turno e final.
4. Construir artefato uma vez, guardar digest no `ReleaseManifest` e arquivar relatórios
   e diagnósticos; SBOM é opcional até decisão específica.
5. Em tag válida, promover somente o artefato já testado ao registry.
6. No host: baixar por digest, validar compatibilidade/migração, backup prévio, iniciar
   sem tráfego, esperar health e alternar proxy; em falha, voltar ao release anterior.

Tag não autoriza migração destrutiva. Migração precisa ser compatível ou ter plano de
backup, restore e rollback aprovado. Esta proposta não presume credenciais de produção
no CI ou no repositório.

## Backups, observabilidade e resposta

Backup proposto: cópia consistente de banco e snapshots, com manifesto de checksums e
intervalo exato de log. Rodar sem disputar o harness, reter ao menos uma cadeia
restaurável e testar restore periódico em alvo isolado. Frequência, destino, criptografia
e retenção dependem de decisão do usuário.

Métricas mínimas: release ativo, health/idade, turnos resolvidos, duração de turno,
falhas de comando, tamanho/idade de log/snapshot, conexões, CPU/RAM/disco, sucesso/idade
de backup e restore, chamadas/tokens/custo/fallback de IA agregados. Alertar em
indisponibilidade, backup falho, espaço/memória críticos, divergência de hash CI e
fallback recorrente acima de limiar futuro.

Runbooks identificam `release_id`, preservam logs e backup, isolam incidente e só então
restauram ou revertem. Nunca começam apagando volume, log ou snapshot.

## Estratégia de testes

- **Contrato:** schemas, release incompatível, bind público indevido, banco publicado e
  produção sem TLS.
- **Determinismo:** matriz CI reproduz fixtures e compara hashes por turno/final; data,
  rede e observabilidade são injetadas ou excluídas do motor.
- **Propriedades:** sequências de deploy/rollback/restore verificam idempotência,
  append-only, checksum e preservação de release em promoção falha.
- **Fixtures gravadas:** IA/serviços externos usam record/replay; CI nunca chama LLM,
  API paga, proxy real ou VPS. Fixture tem schema e conteúdo sanitizado, nunca segredo.
- **Harness:** simulações longas medem hash, p50/p95, RAM e disco; confirmam que
  `P`, `C`, `D`, `G`, `W` e `E` são do núcleo. Referência Fase 2: 1.000 turnos/8 bots.
- **Recuperação:** restore isolado, replay até `log_seq` e comparação de hash; meta futura
  não aprovada é aviso, não teste vermelho.

## Perguntas abertas

1. Qual proxy reverso e estratégia de certificados TLS serão usados?
   **Recomendação:** solução madura, renovação automática e health explícito; ADR após spike curto.
2. Qual registry, quem promove tag e como o host lê a imagem?
   **Recomendação:** acesso mínimo, digest imutável e credencial só de leitura no host.
3. Qual política de backup (frequência, destino externo, retenção, criptografia)?
   **Recomendação:** diário consistente, cópia externa criptografada e restore mensal isolado.
4. A VPS receberá swap de 2 GB para diagnóstico/build em contêiner?
   **Recomendação:** não depender dele em produção; só aprovar após medição e autorização explícita.
5. Que limites de p95, RAM, disco, retenção e fallback bloqueiam promoção?
   **Recomendação:** medir baseline no harness antes de tornar qualquer número um gate.
6. Qual formato de tag e qual homologação antecedem produção?
   **Recomendação:** SemVer com sufixo de fase e homologação que rode health, restore e replay.

## ADRs relacionados

- ADR-0001 — servidor autoritativo (**aceito**).
- ADR-0007 — stack tecnológica (aceito).
- ADR-0006 — motor determinístico e event sourcing (**aceito**; restrição aplicada sem
  reabrir suas decisões).
