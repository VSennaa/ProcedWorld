# Brief — Restaurar acentos do SDD 02

- **Data**: 2026-10-01 (turno da noite) · **Executor**: subagente Codex · **Status**: pendente

## Tarefa

`docs/sdd/02-hex-e-mapa.md` foi gravado por uma ferramenta em code page ANSI e todos os caracteres
acentuados do português viraram `?` (ex.: `Hex?gonos`, `n?o`, `Conven??es`). Restaurar o texto em
PT-BR correto, trocando cada `?` indevido pela letra acentuada certa pelo contexto (á, à, â, ã, é, ê,
í, ó, ô, õ, ú, ç, —). **Não mude mais nada**: nem conteúdo, nem números, nem estrutura, nem código.
Pontos de interrogação legítimos (fim de pergunta) ficam.

## Como gravar (obrigatório)

Edite o arquivo **somente com a ferramenta de patch do Codex (apply_patch)**. Não use PowerShell
`Set-Content`/`Out-File`/redirecionamento para gravar. Ao final rode
`python tools/agents/check-encoding.py docs/sdd/02-hex-e-mapa.md` e só termine com "encoding ok".

## Regras

- Arquivo permitido: apenas `docs/sdd/02-hex-e-mapa.md`. Não rode git. Não instale nada.
- Ao terminar, responda com o resultado do check-encoding.

<!-- encoding-check: quotes damaged text -->
