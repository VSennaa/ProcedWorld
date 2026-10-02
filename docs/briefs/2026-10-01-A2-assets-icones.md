# Brief — Assets: ícones de recursos e UI

- **Data**: 2026-10-01 · **Executor**: subagente Codex · **Status**: pendente
- **Branch**: `feat/front/asset-prototype` (worktree separado)

## Contexto

Leia `docs/gdd/13-direcao-de-arte.md` (2D estilizado, hexágonos ilustrados, ícones planos, assets
**gerados por código/SVG**, sem licença de terceiros) e `docs/gdd/02-mapa-e-tiles.md` §2 (biomas).

## Tarefa

Criar `tools/assets/gen_icons.py` que gera em `assets/icons/` ícones planos 48×48 (legíveis em 24×24):
comida, produção, riqueza, conhecimento, cultura, metal, luxo, coesão, legitimidade, estabilidade,
pressão de crise, ameaça de guerra, exposição ambiental, população, turno, era, Mandato, Governador,
Entropia, Ledger (confiança/ressentimento/dívida), tratado, guerra, Pronto, custo de IA. Estilo único
(traço e cantos consistentes), cor da paleta compartilhada, e versão monocromática para status.

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
