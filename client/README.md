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

# fluxo ao vivo contra um servidor simulado em processo (socket em 127.0.0.1:18123)
$GODOT --headless --path client -s res://tests/live_flow.gd

# abrir o jogo (janela): começa na tela inicial (servidor, criar mundo, entrar, demonstração)
$GODOT --path client
# argumentos úteis, depois de "--": --server=ws://host:porta/ws  --demo=pauta|servidor
#   --screen=pauta|mapa|pesquisa  --select-capital  --select-unit=<id>  --no-decisions
```

A tela inicial pede a URL do servidor (padrão `ws://127.0.0.1:8100/ws`). **Criar mundo** envia
`create_world` e entra como civilização 0; **Entrar** usa o token de sessão guardado em
`user://sessions.cfg` (reconexão). As duas **Demonstrações** não usam rede: "pauta de exemplo" abre
`fixtures/state_snapshot.json` (escrito à mão) e "visão do servidor" abre `fixtures/server_view.json`
e `fixtures/server_catalog.json`, no formato real de `view_for` e do frame `catalog`.

Capturas em `client/docs/` (`inicio`, `pauta`, `pauta-unidades`, `mapa`, `mapa-pronto`, `pesquisa`), geradas com
`--write-movie <pasta existente>/frame.png --quit-after 20 --resolution 720x1280` e pegando o último quadro.

## O que é real

- Matemática hexagonal (pointy-top, odd-r, cilindro horizontal): vizinhos, distância, distância com wrap, pixel para hex (`scripts/hex.gd`).
- Envelope do protocolo (`protocol_version`, `request_id`, `type`, `payload`) com checagem de versão maior (`scripts/protocol.gd`).
- Projeção do snapshot (`scripts/world_view.gd`): valida o mapa (tiles desconhecidos não carregam bioma) e limita a pauta a 1 decisão crítica + 2 importantes.
- Mapa renderizado com os SVGs de `assets/` (tiles, rios, fronteiras, névoa, ícones, cidade), arrastar, zoom (roda/pinça) e toque para selecionar. A pinça não foi testada em aparelho.
- Pauta com cartões de decisão (causa, efeito imediato, risco futuro, prazo, opções com sacrifício) e botão "Pronto".
- Barra inferior com as superfícies (Pesquisa só aparece quando há catálogo).
- Cliente WebSocket (`scripts/net_client.gd`) e fluxo ao vivo em `main.gd`: `create_world`, `join` (com token guardado), `state_snapshot`, `turn_diff`, `catalog`, `ready_state`, `submit_command`, `ready`, `get_snapshot` e erros do servidor em PT-BR. `run_tests.gd` nunca abre rede; `live_flow.gd` usa um servidor simulado local.
- Adaptador da visão do servidor (`scripts/server_view.gd`): `tile` row-major para célula odd-r, terreno numérico para bioma, fronteiras derivadas de `owner`, cidades, unidades (`order`, `skipped_turn`), `idle_units` e eventos pendentes da Entropia como cartões da Pauta. Campos opcionais ausentes são tolerados.
- Unidades no mapa (ícone por tipo/papel, cor da civilização, marca âmbar quando ociosa), cartão da unidade com **Mover** (toque no destino, `MoveTo`), **Explorar**, **Fortificar** e **Pular**. A ordem só aparece na tela depois de `command_accepted`.
- Pauta com a fila de unidades ociosas (toque centraliza o mapa) e **Pronto** desabilitado com "N unidades aguardam ordem"; `units_awaiting_orders` pede novo snapshot.
- Botão **Pronto** compacto (`scripts/ready_fab.gd`) fora da Pauta: selo com as decisões restantes (capital por fundar, eventos sem escolha, unidades ociosas). Com decisões, leva à próxima (capital, eventos na Pauta, unidades por id); com zero, envia `ready` pelo mesmo caminho do botão da Pauta e depois espera o turno avançar. Só conta eventos que aparecem na Pauta (limite de 3 cartões).
- Pesquisa (só leitura) a partir do frame `catalog`: colunas por profundidade de pré-requisito, dominadas, atual e progresso (`scripts/catalog_view.gd`, `scripts/tech_view.gd`).

## O que é placeholder ou limitado

- A demonstração "pauta de exemplo" usa `fixtures/state_snapshot.json`, escrito à mão; não reflete nenhum servidor real.
- Sociedade, Relações e Crônica mostram "em breve" e não fazem nada.
- Em demonstração, "Pronto" marca só um rascunho local e avisa que nada foi enviado; ordens de unidade valem só no cliente. onboarding BYOK, cache local, diffs nem recuperação por cursor.
- Contrato assumido do P16 (ainda não verificado contra o servidor real): `protocol_version: "1.0"`, `order`/`skipped_turn` por unidade, `idle_units`, frame `catalog` (`technologies` e `unit_types`) e comandos `set_unit_order`/`skip_unit` no formato `{"type": ..., "data": {...}}` do `CommandPayload`.
- O servidor não envia nomes de cidade, civilização nem prosa de eventos: o cliente mostra "Cidade N", "Civilização N" e ids de evento/escolha humanizados (em inglês). Rios chegam só como booleano por tile, sem arestas, e por isso não são desenhados no modo ao vivo.
- Escolher pesquisa, desfazer Pronto e agrupar muitas unidades no mesmo tile ainda não existem na interface.
- O mapa ao vivo infere a altura pelo maior índice de tile conhecido (a visão traz só `map_width`).
- Não há login, onboarding BYOK, cache local nem recuperação por cursor.
- Sem texto de tela para tiles em modo daltônico/monocromático; apenas os assets "standard" foram copiados.
- Exportação para Android ainda não foi configurada.
