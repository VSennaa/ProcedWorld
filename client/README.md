# Cliente ProcedWorld (Godot 4.7)

Cliente mínimo, em retrato (720x1280), seguindo `docs/sdd/12-cliente.md` e `docs/gdd/11-experiencia-mobile.md`.
O cliente só exibe a visão enviada pelo servidor e monta intenções; não executa regras nem é fonte de verdade.

## Como rodar

```sh
GODOT=Godot_v4.7.2-stable_win64_console.exe   # ajuste o caminho

# importar assets (necessário uma vez, e depois de adicionar SVGs)
$GODOT --headless --path client --import

# testes headless (devem terminar com ALL TESTS PASSED)
$GODOT --headless --path client -s res://tests/run_tests.gd

# abrir o jogo (janela)
$GODOT --path client
# argumentos úteis, depois de "--": --screen=mapa  --select-capital  --connect
```

`--connect` tenta abrir `ws://127.0.0.1:8100/ws`. Sem ele (padrão) o cliente usa só o fixture.

Capturas: `client/docs/pauta.png` e `client/docs/mapa.png`, geradas com
`--write-movie <caminho>/frame.png --quit-after 20 --resolution 720x1280`.

## O que é real

- Matemática hexagonal (pointy-top, odd-r, cilindro horizontal): vizinhos, distância, distância com wrap, pixel para hex (`scripts/hex.gd`).
- Envelope do protocolo (`protocol_version`, `request_id`, `type`, `payload`) com checagem de versão maior (`scripts/protocol.gd`).
- Projeção do snapshot (`scripts/world_view.gd`): valida o mapa (tiles desconhecidos não carregam bioma) e limita a pauta a 1 decisão crítica + 2 importantes.
- Mapa renderizado com os SVGs de `assets/` (tiles, rios, fronteiras, névoa, ícones, cidade), arrastar, zoom (roda/pinça) e toque para selecionar. A pinça não foi testada em aparelho.
- Pauta com cartões de decisão (causa, efeito imediato, risco futuro, prazo, opções com sacrifício) e botão "Pronto".
- Barra inferior com as cinco superfícies.
- Código de WebSocket (`scripts/net_client.gd`); os testes nunca o usam.

## O que é placeholder ou limitado

- Os dados vêm de `fixtures/state_snapshot.json`, um exemplo escrito à mão; não refletem nenhum servidor real. O formato do payload `state_snapshot` ainda não está fixado no SDD 10 e pode mudar.
- Sociedade, Relações e Crônica mostram "em breve" e não fazem nada.
- "Pronto" sem conexão marca só um rascunho local e avisa que nada foi enviado. Com `--connect` e socket aberto, envia `ready_set` (mensagem ainda sem contrato fechado no servidor).
- Não há login, onboarding BYOK, cache local, diffs nem recuperação por cursor.
- Sem texto de tela para tiles em modo daltônico/monocromático; apenas os assets "standard" foram copiados.
- Exportação para Android ainda não foi configurada.
