# ADR-0003 — Turnos simultâneos com resolução sequencial

- **Status**: Aceito; condição de fechamento do turno **substituída pelo ADR-0008** (sem relógio)
- **Data**: 2026-10-01
- **Decisores**: usuário

## Contexto

Mundo compartilhado com vários jogadores e bots. O modelo de turnos define ritmo, justiça e
como o determinismo é mantido.

## Decisão

- Igual ao multiplayer do *Civilization*: **todos jogam o mesmo turno ao mesmo tempo**.
- As ações são **aplicadas sequencialmente**, na ordem em que o servidor as aceita; essa ordem é gravada
  no log de comandos, o que mantém o replay determinístico.
- O turno termina quando todos os participantes marcam "pronto" ou quando o tempo do turno expira.
  Participantes que não agiram têm o turno jogado pelo Governador (ADR-0002).
- Processamento de fim de turno (produção, crescimento, Entropia, consolidação de memória) acontece
  numa fase própria, depois que as ações de todos foram aplicadas.

## Consequências

- Conflitos (duas unidades indo para o mesmo tile) são resolvidos pela ordem de chegada; o GDD precisa
  definir se isso é aceitável em combate ou se algumas ações (ex.: ataques) são resolvidas em fase própria.
- Bots e Governadores não podem ganhar sempre por agir primeiro: o GDD/SDD deve definir quando eles
  agem (ex.: ordem rotativa com seed, ou janela no fim do turno).
- Tempo de turno é configuração do mundo.
