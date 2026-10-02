# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

Agente ativo: nenhum

- **Fase**: **1 — Especificação** (GDD aprovado pelo usuário em 2026-10-01; Fase 0 encerrada).
- **Branch**: `develop` recebeu o GDD aprovado e os rascunhos de SDD. Próximo trabalho em `docs/sdd/*`,
  `docs/research/*` e `feat/front/asset-prototype`.
- **Decidido em 2026-10-01 (lote final)**: stack aceita (ADR-0007); chave T1 do operador (ADR-0002);
  Laya só no final (ADR-0005); magia com 3–5 fenômenos sistêmicos; interferir na Entropia custa rituais,
  sacrifícios e pesquisas proibidas; direção de arte 2D estilizada (GDD 13); itens técnicos da revisão
  cruzada decididos (ver `docs/sdd/REVISAO-CRUZADA.md`).
- **Próximo passo**: revisão do SDD com o usuário (2 a 2, como no GDD) e aprovação → Fase 2.
- **Feito na madrugada de 2026-10-02**: 4 estudos em `docs/research/` (outros jogos, sandbox e mundos
  emergentes, IA com LLM em jogos, integrações técnicas) com fontes via busca na web do Codex; protótipo de
  assets em `assets/` (89 SVGs de tiles por bioma com overlays, 26 ícones × 3 variantes, gerados por
  `tools/assets/*.py`, determinísticos, CC0) — ver `assets/preview.html`.
- **Feito em 2026-10-02 06:15–07:00 (fila sequencial no Codex)**: decisões sincronizadas nos SDDs e
  `PERGUNTAS-ABERTAS.md` atualizado; catálogos-rascunho em `data/catalogs/` com validador
  (`tools/catalogs/validate.py`); síntese dos estudos em `docs/research/SINTESE.md` (top 10 propostas);
  spike `spike/back/protocol-serialization` (chunk 8×8 = 12,5 KiB JSON / 1,5 KiB gzip; `Option<Option>`
  quebra em JSON → usar patch de três estados); spike `spike/front/godot-hex-render` (Godot 4.7, testes
  headless passam, screenshot). No PC, o Smart App Control bloqueia executáveis recém-compilados
  (build scripts do cargo); compilação Rust roda na VPS em Docker.
- **Feito em 2026-10-02 11:30–12:05 (fila 2 no Codex, autônomo)**: `docs/sdd/GUIA-DE-REVISAO.md` (pares de
  revisão com 39 perguntas de múltipla escolha); catálogos v2 (10 unidades, 14 edifícios, 12 melhorias,
  24 eventos; validador ok); assets v2 em `assets/entities/` (138 SVGs, determinísticos); sucessão e
  entrada tardia como comandos (SDD 15/17, RC2-03); SDDs 19 (retenção), 20 (conflitos), 21
  (notificações); glossário sincronizado; spike Godot v2 (continentes, rios, fronteiras, névoa,
  recursos; testes headless ok).
- **Próximo passo**: revisão do SDD 2 a 2 seguindo `docs/sdd/GUIA-DE-REVISAO.md` → aprovação → Fase 2.
- **Pendente para o usuário**: avaliar o visual em `assets/preview.html`; atualizar
  `docs/PERGUNTAS-ABERTAS.md` com as decisões de 2026-10-01; revisão do SDD 2 a 2.
- **Cotas (00:10)**: Claude 68% (5 h) / 77% (semana); Codex 95% (5 h) / 46% (semana) — Codex esgotado
  até o reset da janela de 5 h.

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
