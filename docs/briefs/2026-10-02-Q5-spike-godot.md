# Brief Q5 — Spike: cliente Godot renderizando o mapa hexagonal

## Contexto
Branch descartável `spike/front/godot-hex-render` (nunca mergeada). Godot 4.7.2 está em
`C:/Users/vinic/Downloads/Godot_v4.7.2-stable_win64.exe/Godot_v4.7.2-stable_win64_console.exe`
(use o executável `_console` para rodar sem janela). Leia `docs/gdd/02-mapa-e-tiles.md` §2–3,
`docs/gdd/13-direcao-de-arte.md`, `docs/sdd/02-hex-e-mapa.md` e `docs/sdd/12-cliente.md`; os tiles SVG
estão em `assets/tiles/` (veja `assets/tiles/manifest.json`).

## Tarefa
Criar `spikes/godot-hex-render/` (projeto Godot 4.7 independente) que:
- copia/importa os SVGs de bioma necessários para dentro do projeto;
- gera um mapa 40×40 com seed fixa (porte simples do gerador inteiro do spike de determinismo: grid
  pointy-top odd-r, wrap horizontal) e o desenha com os tiles;
- câmera para celular em retrato (arrastar, pinçar/zoom com roda, wrap horizontal contínuo);
- toque/clique seleciona um hexágono e mostra coordenadas e bioma num rótulo;
- testes em GDScript rodáveis sem janela: vizinhança, distância com wrap e conversão pixel→hex
  (`--headless -s res://tests/run_tests.gd`), que imprimem "ALL TESTS PASSED";
- se possível, gerar um screenshot `spikes/godot-hex-render/screenshot.png` (ex.: `--write-movie` com
  `--quit-after`), sem bloquear se não der;
- `README.md` com como rodar, o que foi validado e problemas encontrados.

Arquivos permitidos: `spikes/godot-hex-render/**` (pode ler `assets/`).

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada fora do que a tarefa pede. Nada de segredos, IPs ou hostnames.
- Ao final, rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
