# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

Agente ativo: Claude (turno da noite 2026-10-01 22:30 → 2026-10-02 07:30; áreas: GDD, SDD, spikes)

- **Fase**: 0 — Concepção (GDD)
- **Branch ativa**: `docs/gdd/draft-proposals` (ainda não mergeada em `develop`).
- **Revisão 2 a 2 concluída (rodada 1)**: os 12 pilares foram revisados com o usuário em 2026-10-01;
  decisões registradas no "Decidido" de cada pilar e no ADR-0008 (turno sem relógio).
- **Próximo passo**: passada de consolidação (Claude) para alinhar fórmulas e termos entre pilares,
  depois pedir ao usuário a **aprovação do GDD** (critério de saída da Fase 0) e o merge em `develop`.
- **Turno da noite (autorizado pelo usuário em 2026-10-01 22:20)**: até 07:30, parando antes se a
  cota semanal do Claude chegar a 85%. Escopo autorizado: consolidação do GDD; rascunho do SDD
  (como proposta, sem aprovar nada); spikes técnicos em `spike/*` (Rust no PC via winget se não
  precisar do usuário; Rust no usuário `deploy` da VPS; GitHub Actions). Sem merge em main/develop.
- **Pendências de consolidação conhecidas**: peso de `S` em `P_cidade`; proporção em contratos
  mistos (03, SDD); perguntas abertas restantes em cada pilar.
- Incorporadas as lições do DynMagic (`docs/process/agentes-e-cotas.md` §4).
- **Última auditoria** (2026-10-02): Claude 7% (5 h) / 69% (semana); Codex 32% semanal; OpenRouter ~US$ 3,02.

## Feito

- 2026-10-01 — Bootstrap: CLAUDE.md, ADRs 0001–0007, esqueleto de GDD/SDD, ROADMAP, GLOSSARY,
  `infra/README.md`, higiene do repositório. `develop` e `docs/sdd/project-bootstrap` publicadas.
- 2026-10-01 — VPS: clone em `~deploy/ProcedWorld` (branch `develop`), identidade git configurada,
  push via SSH com a chave `~deploy/.ssh/id_ed25519_github`, cadastrada como Deploy Key "VPS"
  (leitura/escrita) e testada em 2026-10-02.
- 2026-10-01 — Subagentes Codex com modelo escolhido pelo Jev Router (`tools/agents/jev-codex.sh`,
  skill `jev-subagents`) e auto-auditoria de cotas (`tools/agents/quota-check.sh`, skill
  `quota-audit`, `docs/process/agentes-e-cotas.md`).
- 2026-10-01 — Primeiro lote de subagentes: 2 de 12 pilares concluídos; o resto falhou com 402 do
  OpenRouter (reserva de crédito em requisições paralelas). Execução movida para o Codex na conta do
  ChatGPT; OpenRouter ficou só com a decisão do Jev.
- 2026-10-01 — Mapeamento Jev → Codex pelo benchmark do Akita (`tools/agents/model-scores.json`).
- 2026-10-01/02 — Revisão 2 a 2 dos 12 pilares com o usuário via perguntas de múltipla escolha;
  ADR-0008 (turno sem relógio); regra "texto de jogador é dado não confiável" no CLAUDE.md;
  regra de execução de escolhas Anthropic do Jev no próprio Claude (com ≥ 20% de cota livre).
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
