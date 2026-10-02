# Spike Godot — mapa hexagonal

Projeto independente em Godot 4.7 para avaliar a renderização de um mapa pointy-top 40×40, em offset **odd-r**, com cilindro horizontal e polos fechados. O mapa local é apenas uma visualização determinística de teste: não substitui o mapa autoritativo do servidor.

## Como rodar

Abra `project.godot` no Godot 4.7 ou execute:

```powershell
& 'C:/Users/vinic/Downloads/Godot_v4.7.2-stable_win64.exe/Godot_v4.7.2-stable_win64_console.exe' --path spikes/godot-hex-render
```

Para os testes sem janela:

```powershell
& 'C:/Users/vinic/Downloads/Godot_v4.7.2-stable_win64.exe/Godot_v4.7.2-stable_win64_console.exe' --headless --path spikes/godot-hex-render -s res://tests/run_tests.gd
```

## Validado

- Geração inteira estável com seed `20261001`, elevação suavizada, exatamente 35% de terra, continentes e biomas por latitude/umidade.
- Vizinhança odd-r, polos fechados, distância com wrap horizontal e pixel→hex pelos testes GDScript.
- Rios em arestas, fronteiras territoriais de oito civilizações (capitais a oito hexes ou mais), recursos e névoa visível/lembrada/desconhecida para a civilização do jogador.
- Arraste, zoom por roda, toque/clique de seleção e cópias desenhadas na costura horizontal.

## Limites e problemas encontrados

- A geração é deliberadamente de apresentação: rios, fronteiras, recursos e névoa são dados de renderização, não contratos do motor autoritativo; justiça de início, placas e persistência ainda não foram avaliadas.
- Pinça usa a distância entre dois toques; o emulador desktop valida a roda. Uma verificação física em Android continua necessária.
- Os sete SVGs em `assets/tiles/` foram copiados do catálogo padrão e importados pelo Godot no primeiro carregamento.
