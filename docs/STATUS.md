# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

Agente ativo: build loop agendado (Codex) em `feat/ai/phase3-core`, área `engine/` — começa 2026-10-03 02:20

- **Fase**: **3 — Alfa** (Fase 2 concluída e mergeada em `develop` em 2026-10-02: 1.000 turnos, 8 bots,
  replay idêntico, mesmo hash em Linux/Windows/macOS na CI). Tag `v0.1.0-indev.1` **aguarda o usuário**.
- **Fase 3 em `feat/ai/phase3-core`**: P9 (intenções e portas de IA) pronto e commitado. P10 (Governador)
  escrito, mas o Governador escolhia **um comando por turno** e o teste de saúde caiu para 8 cidades;
  o supervisor corrigiu um teste desatualizado e abriu o **P10b** (conjunto de comandos por turno).
  Árvore de trabalho com P10 **não commitado** (testes do motor ok; teste de saúde falha) até o P10b passar.
- **Fila agendada (02:20)**: P10b → P11 (diplomacia) → P12 (Entropia, devolve o piso de pressão) →
  P13 (Crônica e memória). Para na primeira falha ou se a cota do Codex acabar; cada tarefa aprovada é
  commitada e o quadro é atualizado.
- **Incidentes de 2026-10-02 22:50**: cota de 5 h do Codex esgotada (volta 02:12) e IP do PC bloqueado
  no SSH da VPS por rajada de conexões (`ufw limit`/fail2ban). Corrigido: o ciclo para na cota do Codex
  e o espelho do quadro sincroniza no máximo 1×/min.
- **Quadro**: `https://<hostname da VPS>/board.html` (HTTP Basic, senha rotacionada por sprint) ou
  `.agent-runs/board.html` local.
- **Para o usuário**: aprovar a tag `v0.1.0-indev.1`; revisar `feat/ai/phase3-core` antes do merge.

## Turno da noite (2026-10-01 22:30 → 23:10)

Encerrado cedo por falta de trabalho desbloqueado: o restante depende das decisões acima.
- **GDD**: consolidação com `docs/gdd/12-variaveis-e-formulas.md` (fonte única de `C`, `L`, `S`, `D`, `G`,
  `W`, `E`, `P`); unificadas privação (03×04), coesão (04×06) e pressão (00×06, `S` com peso reduzido).
- **SDD**: 19 rascunhos (`docs/sdd/00`–`18`) por subagentes Codex com modelo escolhido pelo Jev; duas
  revisões cruzadas com correções só mecânicas (`REVISAO-CRUZADA.md`); perguntas consolidadas em
  `docs/PERGUNTAS-ABERTAS.md`. Nada aprovado.
- **Spikes (ADR-0007)**: determinismo idêntico em VPS, Linux, Windows e macOS ARM (branch
  `spike/engine/determinism-hex`, nunca mergear); servidor axum+tokio+sqlx compila na VPS em Docker em
  ~157 s com mínimo de 316 MB livres; Rust no PC bloqueado pelo Smart App Control (não contornado).
- **Ferramentas**: `tools/agents/check-encoding.py` (pega acentos trocados por `?`, que escaparam numa
  primeira checagem do SDD 02 — restaurado e verificado).
- **Cotas ao encerrar (23:10)**: Claude 48% (5 h) / 75% (semana); Codex 59% (5 h) / 40% (semana);
  OpenRouter ~US$ 2,99 (só decisões do Jev).
- **VPS**: Rust instalado em `~deploy/.cargo` (musl + rust-lld); imagem Docker `rust:1-slim` mantida;
  temporários de build removidos. Nenhuma porta publicada, nenhum serviço criado.

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

- Itens restantes em `docs/PERGUNTAS-ABERTAS.md` (precisa ser atualizado com as decisões de 2026-10-01).

## Riscos conhecidos

- VPS com 2 GB de RAM: compilação Rust + PostgreSQL + agente de código podem disputar memória.
  Considerar swap ou build no CI antes da Fase 2.
- Custo dos subagentes varia muito conforme o modelo que o Jev escolhe (Sonnet custa bem mais que
  `gpt-6-luna` ou DeepSeek).
