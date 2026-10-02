# Brief Q1 — Sincronizar decisões de 2026-10-01 nos SDDs e nas perguntas abertas

## Tarefa
Decisões do usuário de 2026-10-01 ainda não refletidas nos SDDs (fonte: `docs/STATUS.md` seção "Agora",
`docs/adr/*.md`, seções "## Decidido" de `docs/gdd/*.md`, tabela "Decisões do usuário" no fim de
`docs/sdd/REVISAO-CRUZADA.md`):
- ADR-0007 **aceito** (Rust + Godot 4 + PostgreSQL): remover nos SDDs as ressalvas "se ADR-0007 for aceito".
- Chave do modelo de decisão (Jev; depois Laya) é **do operador**; Laya só ao final (ADR-0002, ADR-0005).
- Magia: 3–5 fenômenos raros e sistêmicos; interferir na Entropia custa rituais, sacrifícios, pesquisas
  proibidas (GDD 00, 08).
- Snapshots: início, fim de cada era, a cada 50 turnos; turno começa em 0; presença com janela técnica
  de reconexão de 60 s; chunk de 48 KiB até benchmark; leis podem restringir a migração automática.
- GDD aprovado; projeto na Fase 1.

1. Atualizar os SDDs afetados (`docs/sdd/*.md`) para refletir as decisões, trocando "pergunta aberta"
   por "decidido em 2026-10-01" onde couber. Não invente nada além das decisões.
2. Reescrever `docs/PERGUNTAS-ABERTAS.md`: remover o que foi decidido (listar no fim em
   "## Decididas em 2026-10-01"), manter o resto com a mesma estrutura e reordenar o topo pelo que
   **bloqueia a aprovação do SDD**.

Arquivos permitidos: `docs/sdd/*.md`, `docs/PERGUNTAS-ABERTAS.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada fora do que a tarefa pede. Nada de segredos, IPs ou hostnames.
- Ao final, rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
