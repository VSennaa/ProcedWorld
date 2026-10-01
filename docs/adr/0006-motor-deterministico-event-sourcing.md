# ADR-0006 — Motor determinístico com event sourcing; IA só propõe

- **Status**: Aceito
- **Data**: 2026-10-01
- **Decisores**: usuário (via CLAUDE.md)

## Contexto

O jogo precisa ser previsível e resiliente (sem travar, reproduzível, auditável) e ao mesmo tempo
maleável (eventos e comportamento gerados por IA). IA generativa é não determinística e falha.

## Decisão

- `estado[n+1] = step(estado[n], comandos[n], seed)`, com função pura, PRNG com seed versionada e
  aritmética inteira/ponto fixo. Sem I/O, relógio ou IA dentro do `step`.
- Toda entrada externa (ação de jogador, resposta de IA, evento da Entropia) vira **comando gravado**.
- Snapshots periódicos e hash do estado por turno.
- A IA produz **intenções** tipadas; o motor valida e converte em comandos; falhas caem em fallback T0.

## Consequências

- Replays e testes de regressão são exatos, e bugs são reproduzíveis a partir do log.
- Toda mecânica precisa ser expressa como regras do motor e catálogos de dados; a IA não tem atalho.
- O CI roda testes de determinismo (mesmo hash em máquinas diferentes).
