# SDD 02 ? Hex?gonos e mapa

- **Status:** rascunho de proposta; n?o aprovado.
- **Data:** 2026-10-01.
- **Escopo relacionado:** ADR-0004 (aceito), ADR-0006 (aceito), ADR-0007 (proposto), GDD 02 (decis?es parciais).
- **Regra de leitura:** os contratos e invariantes abaixo s?o propostas para revis?o. Decis?es do GDD e ADRs aceitos est?o explicitamente identificados.

## 1. Objetivo e fronteiras

Este subsistema define identidade, topologia, armazenamento l?gico e consulta de tiles; gera um mapa inicial reproduz?vel; calcula primitivas geom?tricas; e publica fatias do mapa para clientes.

Responsabilidades:

- Normalizar coordenadas axiais em um mapa finito de cilindro com largura horizontal e polos fechados.
- Fornecer vizinhan?a, dist?ncia, linha, anel, ?rea e itera??o can?nica.
- Construir e validar o mapa por etapas determin?sticas a partir de par?metros e seed.
- Manter dados geogr?ficos e seus ?ndices est?veis para comandos, snapshots, hashes e replay.
- Converter estado autoritativo em chunks de leitura adequados ao cliente, respeitando fog of war fornecido pelo dom?nio de visibilidade.

Fora de escopo:

- Regras econ?micas, popula??o, propriedade, cidades, movimento e custo de travessia; consomem consultas do mapa.
- Descoberta e controle de visibilidade por civiliza??o; o mapa apenas recebe uma proje??o de visibilidade para serializa??o.
- Decis?o da Entropia, sele??o de eventos e narrativa. Um evento validado pode emitir comando que altera dados do mapa.
- Renderiza??o, zoom, sele??o por toque e armazenamento local do cliente; pertencem ao SDD do cliente.
- Persist?ncia f?sica, autentica??o, protocolo de transporte e gest?o de chaves; pertencem a outros subsistemas.

O servidor ? autoridade sobre o mapa (ADR-0001, via CLAUDE.md). Cliente nunca envia altera??o direta de tile.

## 2. Conven??es geom?tricas

### 2.1 Coordenadas e topologia

**Decidido:** hex?gonos, representa??o axial/c?bica (ADR-0004); pointy-top e mundo cil?ndrico, wrap horizontal e polos fechados (GDD 02 ? Decidido).

A proposta usa coordenadas axiais inteiras `(q, r)`; a terceira coordenada c?bica ? `s = -q-r`, derivada e n?o armazenada. `q` percorre `[0, width)` e `r` percorre `[0, height)`. A orienta??o pointy-top afeta somente proje??o visual, nunca a sem?ntica axial.

`r` fora do intervalo ? inv?lido: n?o h? wrap vertical. `q` ? normalizado por m?dulo euclidiano para o intervalo do mapa. A geometria ? definida por uma largura axial retangular e linhas de `r` fixas; essa conven??o, incluindo eventuais efeitos de borda em mapas de altura par/?mpar, precisa de fixtures antes de ser ratificada.

Uma coordenada recebida de fora ? validada/normalizada uma vez na fronteira. Coordenadas internas de tile s?o can?nicas. Fun??es que exigem tile v?lido retornam erro para `r` inv?lido; n?o devem silenciosamente projetar polos para uma linha v?lida.

```text
type Axial = { q: Int, r: Int }
type MapShape = { width: PositiveInt, height: PositiveInt }
type TileId = UInt32  // proposta: ?ndice denso est?vel

normalize_q(q, width) = ((q % width) + width) % width
is_valid({q,r}, shape) = 0 <= r < height
canonical({q,r}, shape) = { normalize_q(q,width), r } se is_valid; erro caso contr?rio
```

### 2.2 Dire??es e vizinhos

Proposta de ordem can?nica fixa das seis dire??es axiais; essa ordem ? parte do contrato e deve ser versionada se mudar:

```text
DIRECTIONS = [(+1,0), (+1,-1), (0,-1), (-1,0), (-1,+1), (0,+1)]
neighbors(c) = para d em DIRECTIONS: canonical(c+d) se r v?lido
```

Vizinhos que cruzam o polo s?o omitidos, n?o duplicados nem refletidos. Cada resultado ? ?nico; ordena??o acompanha `DIRECTIONS`. Isso deixa o grau de tiles de borda menor que seis.

### 2.3 Dist?ncia e primitivas

Dist?ncia plana entre axiais ? `(abs(dq)+abs(dr)+abs(ds))/2`. No cilindro, escolher entre imagens horizontais equivalentes:

```text
cylinder_distance(a,b) = min_k hex_distance(a, {b.q + k*width, b.r})
```

A proposta limita `k` a candidatos pr?ximos de `round((a.q-b.q)/width)` e testa ambos os vizinhos inteiros; polos continuam fechados. Para qualquer uso que precise de caminho, desempate deve ser can?nico. A defini??o deve ser validada para larguras pequenas, costuras e bordas.

`line(a,b)` retorna sequ?ncia inclusiva de tiles que aproxima a reta hexagonal, com tie-break por dire??o can?nica; wrap escolhe a imagem de `b` com menor dist?ncia e empate pela menor transla??o assinada. Se houver empate geom?trico, selecionar resultado lexicograficamente pela sequ?ncia de `TileId`.

`ring(center, radius)` retorna tiles a dist?ncia exata `radius`; `area(center, radius)` inclui dist?ncias de zero at? o raio, sem duplicatas ap?s wrap e sem tiles fora dos polos. Para raio zero, linha/anel/?rea cont?m somente o centro v?lido. Ordena??o de `ring` e `area`: crescente por dist?ncia e depois `TileId` crescente. Limitar raio ao di?metro m?ximo do mapa ou retornar resultados ?nicos; nunca fazer loop proporcional a um raio arbitr?rio sem limite.

Essas consultas s?o puras, n?o acessam rede, rel?gio, PRNG ou armazenamento externo. Dist?ncias, ?ndices e desempates usam inteiros.

## 3. Contratos do subsistema

Os nomes s?o ilustrativos; contratos s?o proposta independente de linguagem.

```text
MapGeometry
  create(shape: MapShape) -> Result<Geometry, MapError>
  normalize(coord: Axial) -> Result<Axial, MapError>
  tile_id(coord: Axial) -> Result<TileId, MapError>
  coord(tile_id: TileId) -> Result<Axial, MapError>
  neighbors(coord: Axial) -> Result<List<Axial>, MapError>
  distance(a: Axial, b: Axial) -> Result<UInt32, MapError>
  line(a: Axial, b: Axial) -> Result<List<Axial>, MapError>
  ring(center: Axial, radius: UInt32) -> Result<List<Axial>, MapError>
  area(center: Axial, radius: UInt32) -> Result<List<Axial>, MapError>

MapGenerator
  generate(params: WorldParams, seed: Seed, generator_version: Version)
    -> Result<GeneratedMap, GenerationFailure>

MapQuery
  get_tile(state: WorldState, coord: Axial) -> Result<Tile, MapError>
  get_chunk(state: WorldState, request: ChunkRequest, viewer: ViewerId)
    -> Result<MapChunk, MapError>
```

`WorldParams` cont?m shape, abund?ncia de terra e personalidade clim?tica; valores e cat?logo pertencem ? cria??o de mundo. Par?metros inv?lidos falham antes da gera??o. Generator recebe vers?o expl?cita para preservar mapas antigos quando algoritmo/cat?logos mudarem.

`GeneratedMap` inclui shape, tiles, metadados de gera??o (vers?o, tentativa, identificador do algoritmo de PRNG) e pontos de partida validados. `GenerationFailure` ? tipado: par?metro inv?lido, limite de tentativas excedido, conectividade insuficiente, equil?brio m?nimo n?o atingido, overflow ou cat?logo incompat?vel.

Mudan?as permanentes/tempor?rias do mapa ocorrem por comandos tipados aceitos pelo motor: por exemplo `ApplyMapEvent`, `ChangeImprovement`, `DepleteDeposit`. Validadores aplicam pr?-condi??es. `step` aplica comandos em ordem can?nica; API nenhuma altera tile fora dele.

## 4. Modelo de dados

### 4.1 Estado can?nico

Proposta de armazenamento l?gico: vetor denso row-major, `index = r * width + q`. O vetor e sua ordem fazem parte do estado hasheado; `TileId` ? o ?ndice. O wrap ocorre na geometria, n?o por duplica??o de tiles nas bordas.

```text
MapState {
  shape: MapShape,
  generator_version: Version,
  generation_attempt: UInt32,
  tiles: Tile[width * height],
  climate: ClimateState,
  map_revision: UInt64
}
Tile {
  elevation: Int8,
  temperature: UInt8,
  moisture: UInt8,
  biome_id: CatalogId,
  fertility: UInt8,
  deposit: Optional<Deposit { kind_id, stock, max_extraction }>,
  feature_bits: UInt32,
  improvement_id: Optional<CatalogId>,
  owner_id: Optional<EntityId>
}
```

Campos e faixas s?o propostas do GDD 02, n?o requisitos fechados. Visibilidade por civiliza??o ? estado separado, pois tem m?ltiplos observadores e n?o define geografia. Bioma efetivo ? derivado de tabela catalogada sobre eleva??o/temperatura/umidade, conforme GDD; pode ser materializado para consulta, mas deve ser regener?vel e coerente com a vers?o do cat?logo.

Rios correm em arestas, ent?o sua representa??o proposta ? um conjunto de `EdgeFeature` associado a cada par n?o ordenado de tiles adjacentes, normalizado pelo menor par de `TileId`. N?o usar flag de rio por tile como fonte autoritativa. A decis?o detalhada deve alinhar-se ao GDD antes da implementa??o.

?ndices auxiliares (por bioma, dono, recurso ou chunks) s?o derivados e reconstru?veis. N?o entram no hash can?nico, a menos que sejam explicitamente definidos como estado; resultados de consulta n?o podem depender da ordem de um ?ndice hash.

### 4.2 Invariantes verific?veis

- `width > 0`, `height > 0`, produto cabe no limite configurado e no tipo de `TileId`.
- Exatamente `width * height` tiles; `tile_id(coord(r,q)) == r*width+q` para toda coordenada v?lida.
- Cada valor enumerado/ID referencia cat?logo dispon?vel na vers?o gravada.
- Temperatura, umidade e fertilidade respeitam faixas definidas pela vers?o do schema.
- Dep?sito ausente equivale a nenhum dep?sito; dep?sito presente tem estoque n?o negativo e extra??o m?xima n?o negativa.
- Aresta de rio conecta tiles adjacentes, existe uma vez e ordena endpoints canonicamente.
- Dono aponta para entidade existente ou aus?ncia; mudan?as de entidade n?o deixam refer?ncia inv?lida.
- Nenhuma consulta retorna coordenada fora dos polos; wrap preserva cardinalidade sem duplicatas.
- Gera??o com mesmas entradas, vers?es e cat?logos produz bytes/hash can?nicos iguais.
- Aplicar o mesmo comando validado ao mesmo estado produz mesmo estado; comando inv?lido n?o causa muta??o parcial.

## 5. Gera??o procedural

**Decidido:** mapa procedural a partir de seed; personalidade clim?tica escolhida na cria??o; mudan?as permanentes de relevo permitidas via evento validado; dep?sitos finitos com renova??o lenta por era (GDD 02 ? Decidido). A descri??o de pipeline e par?metros abaixo ? proposta do GDD, ainda sujeita a revis?o.

Pipeline determin?stico proposto:

1. Validar par?metros, dimens?es, vers?o de algoritmo e cat?logo.
2. Derivar sub-seeds independentes por dom?nio est?vel: `derive(world_seed, generator_version, stage_id, attempt)`. N?o derivar por ordem de execu??o ou endere?o de mem?ria.
3. Criar massas/placas; calcular eleva??o; calcular temperatura e umidade; gerar rios; classificar biomas e dep?sitos; escolher pontos iniciais; validar conectividade e justi?a.
4. Em falha recuper?vel, incrementar tentativa explicitamente e repetir todas as etapas dependentes daquela tentativa. Ao exceder limite, retornar erro tipado, sem mapa parcial.
5. Gravar seed inicial, vers?es, identificadores de etapas e tentativa escolhida como metadados do evento de cria??o de mundo. Segredo n?o ? usado como seed.

Cada etapa consome somente seu PRNG derivado e dados de entrada imut?veis. Uma mudan?a em distribui??o de uma etapa n?o desloca sequ?ncias aleat?rias das demais. Loops usam ordem de `TileId`; redu??o paralela, se adotada, precisa produzir o mesmo resultado e tie-break em todas as m?quinas.

Sub-seeds n?o substituem a seed-mestra gravada. O mapa ? gerado uma vez e armazenado como estado; replay normal reexecuta comandos gravados, n?o regenera mundo usando c?digo novo. Para auditoria de gera??o, guardar par?metros, seed, vers?es de algoritmo/PRNG/cat?logos, tentativa e hash do resultado.

**Se ADR-0007 for aceito:** implementar gera??o no crate Rust puro do n?cleo; PRNG versionado e aritm?tica inteira/fixa. Ru?do procedural pode usar algoritmo pr?prio inteiro ou biblioteca somente ap?s spike de determinismo em arquiteturas alvo. Nada de ponto flutuante dependente de plataforma no estado can?nico.

## 6. Falhas, fallbacks e determinismo

Falhas de par?metro, cat?logo ausente, coordenada inv?lida, corrup??o de shape, tentativa esgotada e overflow s?o resultados expl?citos. Cliente pode pedir novamente uma leitura; isso n?o altera mundo. Erro de gera??o impede cria??o do mundo at? o chamador apresentar par?metros v?lidos ou escolher novo seed, a??o externa registrada.

N?o criar mapa ?de emerg?ncia? silenciosamente: um fallback mudaria justi?a e identidade do mundo. Um fallback determin?stico de algoritmo s? ? permitido se for vers?o/catalogado, registrado como entrada e exposto ao chamador.

Log de comandos/eventos deve registrar: par?metros de cria??o, seed do mundo (n?o secreta), vers?es de gera??o/PRNG/cat?logos, n?mero da tentativa, comandos que mudam mapa e IDs/valores de seus alvos. Hash por turno inclui estado geogr?fico can?nico. Falhas de valida??o externas podem entrar em log operacional/auditoria, mas n?o como comando de simula??o se n?o alterarem estado.

Nunca entram no `step`: I/O de disco/rede, rel?gio, gera??o sob demanda, chamada de IA, ordem de itera??o de mapa hash, float n?o determin?stico, RNG global, consulta ? UI. `step` recebe comandos j? ordenados e seed/stream expl?citos conforme SDD 01; mapa puro n?o define a sequ?ncia global de resolu??o de turnos.

IA pode sugerir inten??o de evento; inten??o validada pelo motor gera comando grav?vel. A sa?da bruta do provedor n?o ? lida pelo gerador nem pelo `step`.

## 7. Chunks para cliente

Chunk ? proje??o de leitura, n?o unidade de simula??o. Proposta: janela retangular de coordenadas can?nicas, com `chunk_id`, `map_revision`, vers?o de schema e lista ordenada de tiles. Cliente pode solicitar um chunk e sua revis?o; servidor pode responder completo ou delta se protocolo suportar.

Chunks devem incluir apenas campos que o viewer pode conhecer. Tile desconhecido n?o revela bioma, recursos, relevo nem dono; tile lembrado usa ?ltima observa??o autorizada; vis?vel usa proje??o atual. Pol?tica exata de fog of war pertence ao dom?nio de vis?o.

Proposta de conte?do: coordenada/TileId est?vel, eleva??o/bioma/features vis?veis, propriet?rio vis?vel e indica??o de estado de visibilidade; IDs de cat?logo em vez de texto localizado. Nunca enviar seed-mestra como parte de resposta de jogo. A geometria e os dados privados do mapa permanecem autoritativos no servidor.

Chunk boundary n?o altera vizinhan?a: primitivas consultam mapa completo. Requisi??es perto da costura horizontal normalizam coordenadas e n?o duplicam TileId na resposta. Para cache, revis?o muda somente quando estado representado muda; uma pol?tica de revis?o global versus por chunk segue em aberto.

**Se ADR-0007 for aceito:** Godot 4 pode converter axial para pixel pointy-top e renderizar chunks em TileMap; essa convers?o ? apresenta??o, n?o regra do motor. PostgreSQL pode armazenar snapshot do vetor ou blocos compactados, mas o formato f?sico n?o faz parte deste contrato.

## 8. Or?amento proposto

N?o h? meta de desempenho aprovada. Proposta inicial para benchmark, em servidor de refer?ncia a definir:

- Consultas de vizinhos/dist?ncia: O(1), meta p95 < 1 ms por lote de 10 mil consultas.
- Linha, anel e ?rea: O(n) no n?mero de tiles retornados, com teto expl?cito de raio.
- Gera??o inicial: O(N) por etapa, mem?ria transit?ria O(N); meta p95 < 5 s para o maior mapa inicial configurado.
- Estado: alvo de 16?32 bytes por tile no formato l?gico compacto, exclu?dos ?ndices e overhead de persist?ncia; medir antes de escolher representa??o bin?ria.
- Chunk: limitar resposta por configura??o; meta inicial < 256 KiB descompactado por p?gina, pagina??o para ?reas maiores.

S?o metas para spike/benchmark, n?o garantias de produto. Corrigir/aceitar os limites ap?s tamanhos de mundo, dispositivos e hardware do servidor serem definidos. Nenhuma IA ? necess?ria na geometria ou gera??o; custo de tokens ? zero. Se um agente de IA auxiliar cria??o de narrativa ou sugerir par?metros, a gera??o mec?nica n?o depende disso; qualquer uso pertence aos SDDs de IA e deve ter teto/fallback pr?prio.

## 9. Estrat?gia de testes

- Fixtures gravadas de mapas pequenos: cilindro m?nimo v?lido, larguras par/?mpar, costura horizontal, linhas junto aos dois polos, mapa de teste com rio/ilha; guardar entradas, vers?o e hash esperado.
- Propriedades: normaliza??o horizontal idempotente; simetria de dist?ncia; `distance(a,a)=0`; vizinho v?lido ? adjacente; aus?ncia de wrap vertical; `area` cont?m centro e n?o duplica; todos os itens do anel t?m dist?ncia exata; `line` termina nas extremidades e cada par consecutivo ? adjacente na imagem escolhida.
- Propriedades do armazenamento: bije??o coord/TileId; tamanho constante; serializa??o can?nica round-trip; ?ndices derivados reconstru?dos equivalem aos originais.
- Determinismo: mesma seed/params/vers?es em execu??es repetidas d? mesmo hash; fixtures de gera??o devem ser verificadas em arquiteturas suportadas. Mudan?a intencional de algoritmo exige nova vers?o e fixture nova, sem reescrever replays antigos.
- Testes de gera??o: conectividade de regi?es caminh?veis conforme regra configurada, fairness por m?trica versionada, rios adjacentes e descendentes conforme algoritmo, tentativa m?xima determin?stica e erro sem estado parcial.
- Testes de comandos: comando v?lido altera apenas campos previstos; comando inv?lido deixa hash intacto; replay do log reconstr?i hash por turno.
- Harness deve fixar seed, PRNG, cat?logos, vers?o de algoritmo e par?metros; imprimir caso m?nimo reproduz?vel em falha. Testes de propriedade podem gerar seeds aleat?rias apenas no harness e registrar a seed que falhou; produ??o nunca depende de seed impl?cita.

## 10. Perguntas abertas

1. A conven??o `q` horizontal com `r` limitado produz exatamente a topologia visual pretendida nas linhas polares, para alturas pares e ?mpares? **Recomenda??o:** aprovar uma fixture visual/geom?trica de cada caso antes de fechar os contratos.
2. Qual o tamanho m?nimo/m?ximo de mundo e limite de tiles? **Recomenda??o:** definir antes do benchmark e escolher `TileId`/limites de raio com prova de overflow.
3. Quais dados de tile s?o can?nicos e quais s?o derivados, especialmente bioma, fertilidade e clima? **Recomenda??o:** manter dados-base can?nicos e derivar bioma por cat?logo versionado, salvo necessidade comprovada de estado hist?rico.
4. Qual codec/forma de chunk e granularidade de revis?o? **Recomenda??o:** come?ar com chunks completos versionados e medir; adicionar delta apenas quando custo de rede justificar.
5. Qual formato de rio em arestas e quais regras de conectividade/fairness definem mapa v?lido? **Recomenda??o:** fechar no GDD de mapa antes da implementa??o, com fixtures que cubram desembocadura, lago e wrap.
6. Quais metas reais de tempo e mem?ria valem para servidor e cliente? **Recomenda??o:** manter as metas deste documento como hip?teses de spike at? o tamanho m?ximo de mundo e hardware serem decididos.
7. Como preservar saves se cat?logo ou algoritmo mudar? **Recomenda??o:** persistir vers?es de schema, gerador e cat?logo no snapshot; migra??es devem ser expl?citas e cobertas por ADR quando alterarem sem?ntica.
