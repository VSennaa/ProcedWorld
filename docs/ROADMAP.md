# ROADMAP

Mudança de fase só com aprovação explícita do usuário. Critérios detalhados podem ser refinados no SDD.

## Fase 0 — Concepção · **concluída em 2026-10-01**
- [x] GDD completo
- [x] GDD aprovado pelo usuário

## Fase 1 — Especificação · *atual*
- [ ] SDD de cada subsistema (`docs/sdd/`)
- [x] ADR-0007 (stack) aceito, com spikes de validação
- [ ] Protocolo cliente-servidor versionado
- [ ] Estratégia de testes e orçamentos (tempo por turno, tokens, custo) definidos
- [ ] SDD aprovado pelo usuário

## Fase 2 — Indev · tags `v0.1.0-indev.N`
- [ ] Núcleo determinístico: estado, comandos, `step`, PRNG, hash, snapshots
- [ ] Primitivas hexagonais e geração procedural de mapa
- [ ] Cidades, recursos/rendimentos, tecnologia básica
- [ ] Bots T0 (utility AI)
- [ ] Harness de simulação headless + replay
- [ ] CI: build, testes, teste de determinismo
- **Saída**: 1.000 turnos com 8 bots sem crash; replay idêntico; hash estável entre máquinas

## Fase 3 — Alfa · tags `v0.x.0-alpha.N`
- [ ] Portas de IA (`LLMPort`, `DecisionPort`, `MemoryPort`) com fixtures gravadas
- [ ] Governador + Mandato
- [ ] Entropia (orçamento de tensão, catálogo de eventos)
- [ ] Diplomacia (máquina de estados + ledger de relações)
- [ ] Memória em camadas, previsão de saturação, compactação e reset
- [ ] Servidor na VPS (stack Docker na faixa 8100–8199) e turnos simultâneos
- [ ] Cliente mínimo em Godot com mapa hexagonal
- **Saída**: partida longa com IA real sem intervenção; contexto nunca estoura; custo por turno medido

## Fase 4 — Beta / MVP · tags `v0.x.0-beta.N`
- [ ] Cliente mobile jogável (Android primeiro)
- [ ] Onboarding BYOK e armazenamento seguro de chaves
- [ ] Contas, persistência, notificações
- [ ] Telemetria e tetos de custo
- [ ] Proxy reverso com TLS
- **Saída**: testadores externos jogam por dias; sem perda de save; diplomacia avaliada como coerente

## Fase 5 — 1.0 · tag `v1.0.0`
- [ ] Balanceamento, polimento, documentação de usuário e de auto-hospedagem
