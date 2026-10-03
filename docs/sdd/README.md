# SDD — Software Design Document

> **Aprovado pelo usuário em 2026-10-02** (Fase 1 encerrada). As recomendações marcadas em
> `GUIA-DE-REVISAO.md` valem como escolhas aprovadas e podem ser revistas por ADR.
> Antes: Um arquivo por subsistema, cada um com:
> responsabilidades, interfaces/contratos, modelo de dados, invariantes, falhas e fallbacks,
> orçamento (tempo/tokens/custo), estratégia de testes e ADRs relacionados.

## Subsistemas (aprovados em 2026-10-02)

| Arquivo | Subsistema |
|---|---|
| [00-visao-geral.md](00-visao-geral.md) | Topologia, componentes, fluxo de um turno ponta a ponta |
| [01-nucleo-simulacao.md](01-nucleo-simulacao.md) | Estado, comandos, `step`, PRNG, hash, snapshots, catálogos |
| [02-hex-e-mapa.md](02-hex-e-mapa.md) | Hexágonos pointy-top em cilindro, geração procedural, chunks |
| [03-turnos-e-sessao.md](03-turnos-e-sessao.md) | Turno sem relógio, presença, Pronto, ordem por seed |
| [04-camadas-de-ia.md](04-camadas-de-ia.md) | Portas de IA, intenções, validação, fallback, prompt injection |
| [05-governador.md](05-governador.md) | Mandato, presets, ausência, utilidade `U` |
| [06-entropia.md](06-entropia.md) | Orçamento de tensão, templates, personalidade, interferência |
| [07-diplomacia.md](07-diplomacia.md) | Estados, Ledger, aceitação, negociação em dois modos |
| [08-memoria-e-contexto.md](08-memoria-e-contexto.md) | Documentos canônicos, camadas, saturação, reset |
| [09-persistencia.md](09-persistencia.md) | Log de comandos, snapshots, esquema, retenção |
| [10-protocolo.md](10-protocolo.md) | API cliente-servidor, versionamento, diffs, chunks |
| [11-seguranca-e-chaves.md](11-seguranca-e-chaves.md) | BYOK opcional, criptografia de envelope, tetos de custo |
| [12-cliente.md](12-cliente.md) | Cliente (Godot proposto), Android, retrato, telas |
| [13-infra-e-deploy.md](13-infra-e-deploy.md) | Compose, proxy, build em container, CI, backups |
| [14-testes.md](14-testes.md) | Determinismo, golden replays, fixtures de IA, harness |
| [15-regras-de-dominio.md](15-regras-de-dominio.md) | Economia, cidades, tecnologia, sociedade, combate no `step` |
| [16-visibilidade.md](16-visibilidade.md) | Névoa de guerra, memória de tile, grounding por ator |
| [17-identidade-e-contas.md](17-identidade-e-contas.md) | Contas, sessões, presença, entrada tardia |
| [18-dsl-catalogos.md](18-dsl-catalogos.md) | Linguagem fechada de predicados, seletores e efeitos |
| [19-retencao-de-dados.md](19-retencao-de-dados.md) | Classificação, prazos, expurgo e exportação de dados |
| [20-matriz-de-conflitos.md](20-matriz-de-conflitos.md) | Conflitos entre comandos, desempate e fases |
| [21-notificacoes.md](21-notificacoes.md) | Outbox, push no Android, deduplicação, preferências |

Guia para a revisão 2 a 2: [GUIA-DE-REVISAO.md](GUIA-DE-REVISAO.md). Revisão cruzada e correções mecânicas: [REVISAO-CRUZADA.md](REVISAO-CRUZADA.md). Perguntas em aberto: [../PERGUNTAS-ABERTAS.md](../PERGUNTAS-ABERTAS.md).
