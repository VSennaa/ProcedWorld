# Spike Godot — mapa hexagonal

Projeto independente em Godot 4.7 para avaliar a renderização de um mapa pointy-top 40×40, em offset **odd-r**, com cilindro horizontal. O mapa local é apenas uma visualização determinística de teste: não substitui o mapa autoritativo do servidor.

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

- Geração inteira estável com seed `20261001`, 40×40 e biomas locais simplificados.
- Vizinhança odd-r, polos fechados, distância com wrap horizontal e pixel→hex pelos testes GDScript.
- Arraste, zoom por roda, toque/clique de seleção e cópias desenhadas na costura horizontal.

## Limites e problemas encontrados

- A geração é deliberadamente simples e de apresentação; não inclui ainda placas, rios, recursos ou fairness do spike determinístico do motor.
- Pinça usa a distância entre dois toques; o emulador desktop valida a roda. Uma verificação física em Android continua necessária.
- Os sete SVGs em `assets/tiles/` foram copiados do catálogo padrão e importados pelo Godot no primeiro carregamento.
