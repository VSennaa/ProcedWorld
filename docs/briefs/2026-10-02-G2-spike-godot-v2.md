# Brief G2 — Spike Godot v2: mapa com terra, recursos, rios e fronteiras

## Contexto
Continuação do spike descartável `spikes/godot-hex-render/` (branch `spike/front/godot-hex-render`,
nunca mergeada). Godot 4.7.2 console:
`C:/Users/vinic/Downloads/Godot_v4.7.2-stable_win64.exe/Godot_v4.7.2-stable_win64_console.exe`.

## Tarefa
- Portar o gerador inteiro do spike de determinismo (elevação por ruído suavizado, ~35% de terra,
  biomas por faixa, polos fechados — ver `docs/adr/0007-stack-tecnologica.md` §Resultados e
  `docs/sdd/02-hex-e-mapa.md`) para ter continentes em vez de quase só oceano.
- Desenhar overlays de `assets/tiles/overlays` (rios nas arestas, fronteiras de 8 civilizações com
  capitais a pelo menos 8 hexes, névoa lembrado/desconhecido para a civilização do jogador).
- Mostrar ícones de recurso de `assets/icons/standard` em alguns tiles.
- Manter os testes headless passando (`--headless -s res://tests/run_tests.gd` imprime
  "ALL TESTS PASSED") e acrescentar teste de que a mesma seed gera o mesmo mapa.
- Gerar screenshot com `--write-movie <caminho absoluto>/frame.png --quit-after 20 --resolution 720x1280`
  e copiar o último frame para `spikes/godot-hex-render/screenshot.png`.
- Atualizar o README do spike.
Arquivos permitidos: `spikes/godot-hex-render/**` (pode ler `assets/` e `docs/`).

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
