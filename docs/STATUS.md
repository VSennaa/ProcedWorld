# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

Agente ativo: Sonnet P16 (`feat/engine/unit-orders`, área `engine/`) e Sonnet P17 (`feat/front/live-client`, área `client/`), desde 2026-10-03

- **Fase**: **3 — Alfa**. Fase 2 mergeada em `develop` (tag `v0.1.0-indev.1` aguarda o usuário).
- **`feat/ai/phase3-core`** (98 testes na VPS: 81 motor, 11 harness, 6 servidor), feito em 2026-10-02 com
  subagentes Sonnet 5.5 em paralelo (worktrees) e Codex:
  - P9 intenções e portas de IA (T0/T1/T2, fixtures, texto de jogador não confiável);
  - P10/P10b Governador e Mandato (conjunto de comandos por turno, regra de ausência, presets);
  - P11 diplomacia (8 estados, Ledger Cf/R/Dv, traição sem decaimento, aceitação A, guerra com
    objetivo, auditoria por ação de bot — o harness falha se alguma ação não tiver grounding);
  - P12 Entropia (interpretador fechado da DSL, orçamento de tensão, personalidade, cooldowns,
    resposta útil, ~419 eventos em 300 turnos);
  - P14 servidor axum + WebSocket (create/join/ready, Governador pelo ausente, log reproduzível);
  - P15 cliente Godot (Pauta e Mapa sobre fixture JSON; 242 verificações headless).
- **Decisão do usuário (2026-10-02)**: pressão de crise com coesão **centrada em 50** (GDD 12). Com a
  fórmula antiga a pressão média ficava 0 mesmo com a Entropia; P12b implementa e devolve o piso ≥ 10.
- **Parcial (registrar antes do merge em develop)**: contrapropostas, agregação do Ledger por era,
  liquidação de comércio/ajuda, vassalagem/cessão/troca de tecnologia; templates de interferência e de
  colapso nunca são autoescolhidos; era fixa de 8 turnos na Entropia; `turn_diff` do servidor é visão
  completa (não delta); sem autenticação real nem PostgreSQL no servidor; cliente não fala com o servidor
  ainda (só fixture); sem `Cargo.lock` versionado.
- **2026-10-03**: decisão do usuário — só unidades ociosas bloqueiam Pronto (GDD 01); contrato no SDD
  03/10/12/15 (`bb4b322`); árvore de pesquisa só leitura no cliente. P16/P17 em andamento.
- **Para o usuário**: aprovar a tag `v0.1.0-indev.1`; revisar `feat/ai/phase3-core` antes do merge em `develop`.
- **Quadro**: `https://<hostname da VPS>/board.html` (HTTP Basic).

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
