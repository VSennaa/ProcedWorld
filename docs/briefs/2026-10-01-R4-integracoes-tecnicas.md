# Brief — Estudo: Integrações técnicas

- **Data**: 2026-10-01 · **Executor**: subagente Codex **com busca na web** · **Status**: pendente

## Tarefa

Escrever **somente** `docs/research/integracoes-tecnicas.md`: um estudo para orientar o design e a engenharia do
ProcedWorld (leia antes `CLAUDE.md` §1–3 e `docs/gdd/README.md`; consulte pontualmente os pilares citados).

Temas: (1) Godot 4 ↔ servidor Rust: WebSocket no Godot, serialização (JSON, MessagePack, Protobuf,
FlatBuffers), geração de tipos compartilhados, godot-rust/gdext e quando usar; (2) como Unciv, Freeciv e
outros fazem multiplayer por turnos e sincronização; (3) Jev e Laya via Runware (endpoint `/v1/systemone`,
primitivas, identificadores) e o Jev Router no OpenRouter; (4) DeepSeek: API compatível com OpenAI,
cache de contexto e preços atuais; (5) notificações push no Android (FCM) a partir de servidor
auto-hospedado; (6) guarda de chaves no Android (Keystore) e criptografia de envelope no servidor;
(7) axum + tokio + sqlx: padrões para servidor de jogo por turnos. Inclua um diagrama em texto do fluxo
cliente → servidor → núcleo → portas de IA.

## Formato

- Para cada jogo/projeto/tecnologia: o que é (2 linhas), **como funciona** o mecanismo relevante,
  o que **aproveitar** e o que **evitar** no ProcedWorld, e **qual pilar/SDD** isso afeta
  (`docs/gdd/NN-*.md`, `docs/sdd/NN-*.md`).
- Feche com "## Recomendações para o ProcedWorld" (5–10 itens acionáveis) e "## Fontes".
- **Toda afirmação factual precisa de fonte** (URL) na seção Fontes, preferindo documentação oficial,
  devlogs, wikis e papers. Se não achar fonte, diga "não verificado". Não invente números.
- Não copie trechos longos: resuma com suas palavras; no máximo uma citação curta por fonte.
- Português do Brasil, Markdown, 200–450 linhas, UTF-8, LF.

## Regras

- Arquivo permitido: apenas `docs/research/integracoes-tecnicas.md`. Não rode git. Não instale nada.
- Grave só com a ferramenta de patch (apply_patch), nunca com PowerShell. Ao final rode
  `python tools/agents/check-encoding.py docs/research/integracoes-tecnicas.md` e só termine com "encoding ok".
