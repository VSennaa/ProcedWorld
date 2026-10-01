# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

- **Fase**: 0 — Concepção (GDD)
- **Branch ativa**: `docs/gdd/draft-proposals` (rascunhos de pilares por subagentes; ainda não mergeada).
- **Pilares com proposta redigida** (aguardando revisão do usuário): `00-visao.md`, `02-mapa-e-tiles.md`.
- **Faltam**: 01, 03, 04, 05, 06, 07, 08, 09, 10, 11 (prompts em `.agent-runs/prompts/`, fora do git).
- **Próximo passo**: rodar `quota-audit`; se o OpenRouter tiver saldo ≥ US$ 1, relançar os pilares
  faltantes com no máximo 2 em paralelo e prompts mais enxutos; senão, usar Codex via ChatGPT.
- **Última auditoria** (2026-10-01): Claude 39% (5 h) / 67% (semana, reset 2026-10-03 06:00 UTC);
  Codex 31% semanal (snapshot de 4 dias); OpenRouter US$ 3,07 restantes de US$ 5.

## Feito

- 2026-10-01 — Bootstrap: CLAUDE.md, ADRs 0001–0007, esqueleto de GDD/SDD, ROADMAP, GLOSSARY,
  `infra/README.md`, higiene do repositório. `develop` e `docs/sdd/project-bootstrap` publicadas.
- 2026-10-01 — VPS: clone em `~deploy/ProcedWorld` (branch `develop`), identidade git configurada,
  push via SSH com a chave `~deploy/.ssh/id_ed25519_github` (**pendente: cadastrar como Deploy Key
  com escrita no GitHub**).
- 2026-10-01 — Subagentes Codex com modelo escolhido pelo Jev Router (`tools/agents/jev-codex.sh`,
  skill `jev-subagents`) e auto-auditoria de cotas (`tools/agents/quota-check.sh`, skill
  `quota-audit`, `docs/process/agentes-e-cotas.md`).
- 2026-10-01 — Primeiro lote de subagentes: 2 de 12 pilares concluídos; o resto falhou com 402 do
  OpenRouter (reserva de crédito em requisições paralelas). Ferramenta corrigida (ver
  `docs/process/agentes-e-cotas.md` §2).

## Perguntas abertas (para o usuário)

1. Chave do modelo de decisão (Jev/Runware/OpenRouter) no jogo: do operador do servidor ou do jogador? (ADR-0002)
2. Ordem de ação de bots/Governadores dentro do turno simultâneo (ADR-0003).
3. Ratificar a stack proposta (ADR-0007).
4. Hospedagem futura do Laya, dado que a VPS atual não tem GPU (ADR-0005).
5. Aumentar o limite da chave do OpenRouter (hoje US$ 5) para os lotes de subagentes?

## Riscos conhecidos

- VPS com 2 GB de RAM: compilação Rust + PostgreSQL + agente de código podem disputar memória.
  Considerar swap ou build no CI antes da Fase 2.
- Custo dos subagentes varia muito conforme o modelo que o Jev escolhe (Sonnet custa bem mais que
  `gpt-6-luna` ou DeepSeek).
