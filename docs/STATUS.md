# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

Agente ativo: nenhum

- **Fase**: 0 — Concepção. GDD consolidado e pronto para **aprovação formal**; SDD inteiro em rascunho.
- **Branch ativa**: `docs/gdd/draft-proposals` (ainda não mergeada em `develop`).
- **Para o usuário de manhã (2026-10-02), em ordem:**
  1. Ler `docs/PERGUNTAS-ABERTAS.md` — topo com as 10 perguntas que bloqueiam a próxima fase.
  2. Aprovar (ou pedir ajustes) o **GDD** — `docs/gdd/` incluindo o novo `12-variaveis-e-formulas.md`.
     Aprovação encerra a Fase 0 e libera o merge em `develop`.
  3. Decidir o **ADR-0007 (stack)** — agora com resultados de spike (determinismo passou em 4 plataformas).
  4. Revisar os itens "precisa do usuário: sim" em `docs/sdd/REVISAO-CRUZADA.md` (snapshots, origem do
     número do turno, política de presença, limite de chunk, sucessão/entrada tardia).
  5. Opcional: liberar o `cargo` no Smart App Control do Windows (ou usar WSL) para desenvolver Rust no PC;
     swap de 2 GB na VPS (exige sudo).

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

1. Chave do modelo de decisão (Jev/Runware/OpenRouter) no jogo: do operador do servidor ou do jogador? (ADR-0002)
3. Ratificar a stack proposta (ADR-0007).
4. Hospedagem futura do Laya, dado que a VPS atual não tem GPU (ADR-0005).

## Riscos conhecidos

- VPS com 2 GB de RAM: compilação Rust + PostgreSQL + agente de código podem disputar memória.
  Considerar swap ou build no CI antes da Fase 2.
- Custo dos subagentes varia muito conforme o modelo que o Jev escolhe (Sonnet custa bem mais que
  `gpt-6-luna` ou DeepSeek).
