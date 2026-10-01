# SDD — Software Design Document

> **Fase 1.** Não iniciar antes da aprovação do GDD. Um arquivo por subsistema, cada um com:
> responsabilidades, interfaces/contratos, modelo de dados, invariantes, falhas e fallbacks,
> orçamento (tempo/tokens/custo), estratégia de testes e ADRs relacionados.

## Subsistemas previstos

| Arquivo | Subsistema |
|---|---|
| `00-visao-geral.md` | Topologia, componentes, fluxo de um turno ponta a ponta |
| `01-nucleo-simulacao.md` | Estado, comandos, `step`, PRNG, hash, snapshots, catálogos de dados |
| `02-hex-e-mapa.md` | Coordenadas hexagonais, geração procedural, topologia de borda |
| `03-turnos-e-sessao.md` | Turnos simultâneos, ordem de resolução, timers, prontidão |
| `04-camadas-de-ia.md` | `LLMPort`, `DecisionPort`, fallbacks, validação de intenções, gravação |
| `05-governador.md` | Mandato (schema), loop de decisão, limites aplicados pelo motor |
| `06-entropia.md` | Orçamento de tensão, seleção e parametrização de templates, narrativa |
| `07-diplomacia.md` | Máquina de estados, ledger, avaliação de propostas, geração de texto |
| `08-memoria-e-contexto.md` | Camadas de memória, documentos canônicos, saturação, compactação, reset |
| `09-persistencia.md` | Esquema PostgreSQL, log de comandos, snapshots, migrações |
| `10-protocolo.md` | API cliente-servidor, versionamento, autenticação |
| `11-seguranca-e-chaves.md` | BYOK, criptografia de envelope, tetos de custo |
| `12-cliente.md` | Arquitetura do cliente Godot, renderização hexagonal, UX mobile |
| `13-infra-e-deploy.md` | Compose, proxy, CI/CD, observabilidade, backups |
| `14-testes.md` | Determinismo, golden replays, propriedades, fixtures de IA, harness |
