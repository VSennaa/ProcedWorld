# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

- **Fase**: 0 — Concepção (GDD)
- **Branch ativa**: `docs/gdd/draft-proposals` (ainda não mergeada em `develop`).
- **Todos os 12 pilares têm proposta redigida**, aguardando revisão do usuário. "Decidido" intacto em todos.
- **Próximo passo**: o usuário revisar os pilares; depois, uma passada de consolidação (Claude) para
  alinhar termos e fórmulas entre pilares e levar as perguntas abertas restantes ao usuário.
- **Revisão 2 a 2 com o usuário**: 00–05 revisados e decididos (ver "Decidido" de cada pilar e
  ADR-0008). Próximos: 06+07, 08+09, 10+11. Inconsistência 04×06 resolvida (grupos híbridos); o 06
  precisa ser ajustado ao modelo híbrido na sua revisão.
- **Última auditoria** (2026-10-01 ~23:40 UTC): Codex 32% semanal; OpenRouter ~US$ 3,02 (só decisões do Jev).

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
  OpenRouter (reserva de crédito em requisições paralelas). Execução movida para o Codex na conta do
  ChatGPT; OpenRouter ficou só com a decisão do Jev.
- 2026-10-01 — Mapeamento Jev → Codex pelo benchmark do Akita (`tools/agents/model-scores.json`).
- 2026-10-01 — Segundo lote: 10 pilares redigidos, todos com saída 0 (01, 03, 04, 05 com `gpt-6-sol`
  pela regra antiga de preço; 06, 08, 09, 10, 11 com `gpt-5.6-terra`; 07 com `gpt-6-luna`).

## Perguntas abertas (para o usuário)

1. Chave do modelo de decisão (Jev/Runware/OpenRouter) no jogo: do operador do servidor ou do jogador? (ADR-0002)
3. Ratificar a stack proposta (ADR-0007).
4. Hospedagem futura do Laya, dado que a VPS atual não tem GPU (ADR-0005).

## Riscos conhecidos

- VPS com 2 GB de RAM: compilação Rust + PostgreSQL + agente de código podem disputar memória.
  Considerar swap ou build no CI antes da Fase 2.
- Custo dos subagentes varia muito conforme o modelo que o Jev escolhe (Sonnet custa bem mais que
  `gpt-6-luna` ou DeepSeek).
