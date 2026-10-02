# Brief — Assets: tiles hexagonais por bioma

- **Data**: 2026-10-01 · **Executor**: subagente Codex · **Status**: pendente
- **Branch**: `feat/front/asset-prototype` (worktree separado)

## Contexto

Leia `docs/gdd/13-direcao-de-arte.md` (2D estilizado, hexágonos ilustrados, ícones planos, assets
**gerados por código/SVG**, sem licença de terceiros) e `docs/gdd/02-mapa-e-tiles.md` §2 (biomas).

## Tarefa

Criar `tools/assets/gen_tiles.py` que gera em `assets/tiles/` um SVG por bioma do GDD 02 (oceano,
costa, planície, floresta, selva, savana, deserto, estepe, tundra, pântano, montanha, geleira), em hexágono
**pointy-top** (largura `sqrt(3)·r`, altura `2·r`, r = 64), com textura ilustrada por formas simples
(ondas, árvores, dunas, picos), **3 variações** por bioma escolhidas pela seed, e overlays separados:
rio na aresta (um SVG por aresta), borda de fronteira (cor de civilização como parâmetro) e névoa de
guerra (lembrado/desconhecido). Os tiles devem encaixar sem frestas num grid odd-r: inclua no preview um
mini-mapa 10×8 montado com eles.

## Requisitos comuns

- Gerador em **Python 3 puro** (só biblioteca padrão), determinístico: mesma seed ⇒ mesmos SVGs.
- SVGs limpos, sem dependências externas, sem fontes externas, `viewBox` definido.
- Paleta compartilhada em `assets/palette.json` (crie se não existir; se existir, reutilize), com
  contraste legível em celular e variante para daltonismo onde a cor carrega significado.
- Página `assets/preview.html` (crie ou acrescente uma seção) mostrando tudo lado a lado, em fundo
  claro e escuro, sem JS externo.
- `assets/README.md`: como gerar, o que cada arquivo é, licença **CC0** (obra própria).

## Regras

- Arquivos permitidos: `tools/assets/**`, `assets/**`. Não rode git. Não instale nada.
- Rode o gerador ao final e confira que os arquivos saíram. Grave código e docs com apply_patch.
- Rode `python tools/agents/check-encoding.py assets/README.md` e só termine com "encoding ok".
