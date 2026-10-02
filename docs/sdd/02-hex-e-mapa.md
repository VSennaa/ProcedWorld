# SDD 02 — Hexágonos e mapa

- **Status:** proposta para revisão do SDD; GDD aprovado em 2026-10-01 e ADR-0007 aceito.
- **Data:** 2026-10-01.
- **Escopo relacionado:** ADR-0004, ADR-0006 e ADR-0007 (aceitos); GDD 02.
- **Regra de leitura:** os contratos e invariantes abaixo são propostas para revisão. Decisões do GDD e ADRs aceitos estão explicitamente identificados.

## 1. Objetivo e fronteiras

Este subsistema define identidade, topologia, armazenamento lógico e consulta de tiles; gera um mapa inicial reproduzível; calcula primitivas geométricas; e publica fatias do mapa para clientes.

Responsabilidades:

- Normalizar coordenadas axiais em um mapa finito de cilindro com largura horizontal e polos fechados.
- Fornecer vizinhança, distância, linha, anel, área e iteração canônica.
- Construir e validar o mapa por etapas determinísticas a partir de parâmetros e seed.
- Manter dados geográficos e seus índices estáveis para comandos, snapshots, hashes e replay.
- Converter estado autoritativo em chunks de leitura adequados ao cliente, respeitando fog of war fornecido pelo domínio de visibilidade.

Fora de escopo:

- Regras econômicas, população, propriedade, cidades, movimento e custo de travessia; consomem consultas do mapa.
- Descoberta e controle de visibilidade por civilização; o mapa apenas recebe uma projeção de visibilidade para serialização.
- Decisão da Entropia, seleção de eventos e narrativa. Um evento validado pode emitir comando que altera dados do mapa.
- Renderização, zoom, seleção por toque e armazenamento local do cliente; pertencem ao SDD do cliente.
- Persistência física, autenticação, protocolo de transporte e gestão de chaves; pertencem a outros subsistemas.

O servidor é autoridade sobre o mapa (ADR-0001, via CLAUDE.md). Cliente nunca envia alteração direta de tile.

## 2. Convenções geométricas

### 2.1 Coordenadas e topologia

**Decidido:** hexágonos, representação axial/cúbica (ADR-0004); pointy-top e mundo cilíndrico, wrap horizontal e polos fechados (GDD 02 — Decidido).

A proposta usa coordenadas axiais inteiras `(q, r)`; a terceira coordenada cúbica é `s = -q-r`, derivada e não armazenada. `q` percorre `[0, width)` e `r` percorre `[0, height)`. A orientação pointy-top afeta somente projeção visual, nunca a semântica axial.

`r` fora do intervalo é inválido: não há wrap vertical. `q` é normalizado por módulo euclidiano para o intervalo do mapa. A geometria é definida por uma largura axial retangular e linhas de `r` fixas; essa convenção, incluindo eventuais efeitos de borda em mapas de altura par/ímpar, precisa de fixtures antes de ser ratificada.

Uma coordenada recebida de fora é validada/normalizada uma vez na fronteira. Coordenadas internas de tile são canônicas. Funções que exigem tile válido retornam erro para `r` inválido; não devem silenciosamente projetar polos para uma linha válida.

```text
type Axial = { q: Int, r: Int }
type MapShape = { width: PositiveInt, height: PositiveInt }
type TileId = UInt32  // proposta: índice denso estável

normalize_q(q, width) = ((q % width) + width) % width
is_valid({q,r}, shape) = 0 <= r < height
canonical({q,r}, shape) = { normalize_q(q,width), r } se is_valid; erro caso contrário
```

### 2.2 Direções e vizinhos

Proposta de ordem canônica fixa das seis direções axiais; essa ordem é parte do contrato e deve ser versionada se mudar:

```text
DIRECTIONS = [(+1,0), (+1,-1), (0,-1), (-1,0), (-1,+1), (0,+1)]
neighbors(c) = para d em DIRECTIONS: canonical(c+d) se r válido
```

Vizinhos que cruzam o polo são omitidos, não duplicados nem refletidos. Cada resultado é único; ordenação acompanha `DIRECTIONS`. Isso deixa o grau de tiles de borda menor que seis.

### 2.3 Distância e primitivas

Distância plana entre axiais é `(abs(dq)+abs(dr)+abs(ds))/2`. No cilindro, escolher entre imagens horizontais equivalentes:

```text
cylinder_distance(a,b) = min_k hex_distance(a, {b.q + k*width, b.r})
```

A proposta limita `k` a candidatos próximos de `round((a.q-b.q)/width)` e testa ambos os vizinhos inteiros; polos continuam fechados. Para qualquer uso que precise de caminho, desempate deve ser canônico. A definição deve ser validada para larguras pequenas, costuras e bordas.

`line(a,b)` retorna sequência inclusiva de tiles que aproxima a reta hexagonal, com tie-break por direção canônica; wrap escolhe a imagem de `b` com menor distância e empate pela menor translação assinada. Se houver empate geométrico, selecionar resultado lexicograficamente pela sequência de `TileId`.

`ring(center, radius)` retorna tiles a distância exata `radius`; `area(center, radius)` inclui distâncias de zero até o raio, sem duplicatas após wrap e sem tiles fora dos polos. Para raio zero, linha/anel/área contém somente o centro válido. Ordenação de `ring` e `area`: crescente por distância e depois `TileId` crescente. Limitar raio ao diâmetro máximo do mapa ou retornar resultados únicos; nunca fazer loop proporcional a um raio arbitrário sem limite.

Essas consultas são puras, não acessam rede, relógio, PRNG ou armazenamento externo. Distâncias, índices e desempates usam inteiros.

## 3. Contratos do subsistema

Os nomes são ilustrativos; contratos são proposta independente de linguagem.

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

`WorldParams` contém shape, abundância de terra e personalidade climática; valores e catálogo pertencem à criação de mundo. Parâmetros inválidos falham antes da geração. Generator recebe versão explícita para preservar mapas antigos quando algoritmo/catálogos mudarem.

`GeneratedMap` inclui shape, tiles, metadados de geração (versão, tentativa, identificador do algoritmo de PRNG) e pontos de partida validados. `GenerationFailure` é tipado: parâmetro inválido, limite de tentativas excedido, conectividade insuficiente, equilíbrio mínimo não atingido, overflow ou catálogo incompatível.

Mudanças permanentes/temporárias do mapa ocorrem por comandos tipados aceitos pelo motor: por exemplo `ApplyMapEvent`, `ChangeImprovement`, `DepleteDeposit`. Validadores aplicam pré-condições. `step` aplica comandos em ordem canônica; API nenhuma altera tile fora dele.

## 4. Modelo de dados

### 4.1 Estado canônico

Proposta de armazenamento lógico: vetor denso row-major, `index = r * width + q`. O vetor e sua ordem fazem parte do estado hasheado; `TileId` é o índice. O wrap ocorre na geometria, não por duplicação de tiles nas bordas.

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

Campos e faixas são propostas do GDD 02, não requisitos fechados. Visibilidade por civilização é estado separado, pois tem múltiplos observadores e não define geografia. Bioma efetivo é derivado de tabela catalogada sobre elevação/temperatura/umidade, conforme GDD; pode ser materializado para consulta, mas deve ser regenerável e coerente com a versão do catálogo.

Rios correm em arestas, então sua representação proposta é um conjunto de `EdgeFeature` associado a cada par não ordenado de tiles adjacentes, normalizado pelo menor par de `TileId`. Não usar flag de rio por tile como fonte autoritativa. A decisão detalhada deve alinhar-se ao GDD antes da implementação.

Índices auxiliares (por bioma, dono, recurso ou chunks) são derivados e reconstruíveis. Não entram no hash canônico, a menos que sejam explicitamente definidos como estado; resultados de consulta não podem depender da ordem de um índice hash.

### 4.2 Invariantes verificáveis

- `width > 0`, `height > 0`, produto cabe no limite configurado e no tipo de `TileId`.
- Exatamente `width * height` tiles; `tile_id(coord(r,q)) == r*width+q` para toda coordenada válida.
- Cada valor enumerado/ID referencia catálogo disponível na versão gravada.
- Temperatura, umidade e fertilidade respeitam faixas definidas pela versão do schema.
- Depósito ausente equivale a nenhum depósito; depósito presente tem estoque não negativo e extração máxima não negativa.
- Aresta de rio conecta tiles adjacentes, existe uma vez e ordena endpoints canonicamente.
- Dono aponta para entidade existente ou ausência; mudanças de entidade não deixam referência inválida.
- Nenhuma consulta retorna coordenada fora dos polos; wrap preserva cardinalidade sem duplicatas.
- Geração com mesmas entradas, versões e catálogos produz bytes/hash canônicos iguais.
- Aplicar o mesmo comando validado ao mesmo estado produz mesmo estado; comando inválido não causa mutação parcial.

## 5. Geração procedural

**Decidido:** mapa procedural a partir de seed; personalidade climática escolhida na criação; mudanças permanentes de relevo permitidas via evento validado; depósitos finitos com renovação lenta por era (GDD 02 — Decidido). A descrição de pipeline e parâmetros abaixo é proposta do GDD, ainda sujeita a revisão.

Pipeline determinístico proposto:

1. Validar parâmetros, dimensões, versão de algoritmo e catálogo.
2. Derivar sub-seeds independentes por domínio estável: `derive(world_seed, generator_version, stage_id, attempt)`. Não derivar por ordem de execução ou endereço de memória.
3. Criar massas/placas; calcular elevação; calcular temperatura e umidade; gerar rios; classificar biomas e depósitos; escolher pontos iniciais; validar conectividade e justiça.
4. Em falha recuperável, incrementar tentativa explicitamente e repetir todas as etapas dependentes daquela tentativa. Ao exceder limite, retornar erro tipado, sem mapa parcial.
5. Gravar seed inicial, versões, identificadores de etapas e tentativa escolhida como metadados do evento de criação de mundo. Segredo não é usado como seed.

Cada etapa consome somente seu PRNG derivado e dados de entrada imutáveis. Uma mudança em distribuição de uma etapa não desloca sequências aleatórias das demais. Loops usam ordem de `TileId`; redução paralela, se adotada, precisa produzir o mesmo resultado e tie-break em todas as máquinas.

Sub-seeds não substituem a seed-mestra gravada. O mapa é gerado uma vez e armazenado como estado; replay normal reexecuta comandos gravados, não regenera mundo usando código novo. Para auditoria de geração, guardar parâmetros, seed, versões de algoritmo/PRNG/catálogos, tentativa e hash do resultado.

Com ADR-0007 aceito, implementar geração no crate Rust puro do núcleo; PRNG versionado e aritmética inteira/fixa. Ruído procedural pode usar algoritmo próprio inteiro ou biblioteca somente após spike de determinismo em arquiteturas alvo. Nada de ponto flutuante dependente de plataforma no estado canônico.

## 6. Falhas, fallbacks e determinismo

Falhas de parâmetro, catálogo ausente, coordenada inválida, corrupção de shape, tentativa esgotada e overflow são resultados explícitos. Cliente pode pedir novamente uma leitura; isso não altera mundo. Erro de geração impede criação do mundo até o chamador apresentar parâmetros válidos ou escolher novo seed, ação externa registrada.

Não criar mapa “de emergência” silenciosamente: um fallback mudaria justiça e identidade do mundo. Um fallback determinístico de algoritmo só é permitido se for versão/catalogado, registrado como entrada e exposto ao chamador.

Log de comandos/eventos deve registrar: parâmetros de criação, seed do mundo (não secreta), versões de geração/PRNG/catálogos, número da tentativa, comandos que mudam mapa e IDs/valores de seus alvos. Hash por turno inclui estado geográfico canônico. Falhas de validação externas podem entrar em log operacional/auditoria, mas não como comando de simulação se não alterarem estado.

Nunca entram no `step`: I/O de disco/rede, relógio, geração sob demanda, chamada de IA, ordem de iteração de mapa hash, float não determinístico, RNG global, consulta à UI. `step` recebe comandos já ordenados e seed/stream explícitos conforme SDD 01; mapa puro não define a sequência global de resolução de turnos.

IA pode sugerir intenção de evento; intenção validada pelo motor gera comando gravável. A saída bruta do provedor não é lida pelo gerador nem pelo `step`.

## 7. Chunks para cliente

Chunk é projeção de leitura, não unidade de simulação. Proposta: janela retangular de coordenadas canônicas, com `chunk_id`, `map_revision`, versão de schema e lista ordenada de tiles. Cliente pode solicitar um chunk e sua revisão; servidor pode responder completo ou delta se protocolo suportar.

Chunks devem incluir apenas campos que o viewer pode conhecer. Tile desconhecido não revela bioma, recursos, relevo nem dono; tile lembrado usa última observação autorizada; visível usa projeção atual. Política exata de fog of war pertence ao domínio de visão.

Proposta de conteúdo: coordenada/TileId estável, elevação/bioma/features visíveis, proprietário visível e indicação de estado de visibilidade; IDs de catálogo em vez de texto localizado. Nunca enviar seed-mestra como parte de resposta de jogo. A geometria e os dados privados do mapa permanecem autoritativos no servidor.

Chunk boundary não altera vizinhança: primitivas consultam mapa completo. Requisições perto da costura horizontal normalizam coordenadas e não duplicam TileId na resposta. Para cache, revisão muda somente quando estado representado muda; uma política de revisão global versus por chunk segue em aberto.

Godot 4 converte axial para pixel pointy-top e renderiza chunks em TileMap; essa conversão é apresentação, não regra do motor. PostgreSQL armazena snapshot do vetor ou blocos compactados, mas o formato físico não faz parte deste contrato.

## 8. Orçamento proposto

Não há meta de desempenho aprovada. Proposta inicial para benchmark, em servidor de referência a definir:

- Consultas de vizinhos/distância: O(1), meta p95 < 1 ms por lote de 10 mil consultas.
- Linha, anel e área: O(n) no número de tiles retornados, com teto explícito de raio.
- Geração inicial: O(N) por etapa, memória transitória O(N); meta p95 < 5 s para o maior mapa inicial configurado.
- Estado: alvo de 16–32 bytes por tile no formato lógico compacto, excluídos índices e overhead de persistência; medir antes de escolher representação binária.
- Chunk: limite único de 48 KiB descompactado até benchmark no Android; valor final definido pelo benchmark (decidido em 2026-10-01), com paginação para áreas maiores.

São metas para spike/benchmark, não garantias de produto. Corrigir/aceitar os limites após tamanhos de mundo, dispositivos e hardware do servidor serem definidos. Nenhuma IA é necessária na geometria ou geração; custo de tokens é zero. Se um agente de IA auxiliar criação de narrativa ou sugerir parâmetros, a geração mecânica não depende disso; qualquer uso pertence aos SDDs de IA e deve ter teto/fallback próprio.

## 9. Estratégia de testes

- Fixtures gravadas de mapas pequenos: cilindro mínimo válido, larguras par/ímpar, costura horizontal, linhas junto aos dois polos, mapa de teste com rio/ilha; guardar entradas, versão e hash esperado.
- Propriedades: normalização horizontal idempotente; simetria de distância; `distance(a,a)=0`; vizinho válido é adjacente; ausência de wrap vertical; `area` contém centro e não duplica; todos os itens do anel têm distância exata; `line` termina nas extremidades e cada par consecutivo é adjacente na imagem escolhida.
- Propriedades do armazenamento: bijeção coord/TileId; tamanho constante; serialização canônica round-trip; índices derivados reconstruídos equivalem aos originais.
- Determinismo: mesma seed/params/versões em execuções repetidas dá mesmo hash; fixtures de geração devem ser verificadas em arquiteturas suportadas. Mudança intencional de algoritmo exige nova versão e fixture nova, sem reescrever replays antigos.
- Testes de geração: conectividade de regiões caminháveis conforme regra configurada, fairness por métrica versionada, rios adjacentes e descendentes conforme algoritmo, tentativa máxima determinística e erro sem estado parcial.
- Testes de comandos: comando válido altera apenas campos previstos; comando inválido deixa hash intacto; replay do log reconstrói hash por turno.
- Harness deve fixar seed, PRNG, catálogos, versão de algoritmo e parâmetros; imprimir caso mínimo reproduzível em falha. Testes de propriedade podem gerar seeds aleatórias apenas no harness e registrar a seed que falhou; produção nunca depende de seed implícita.

## 10. Perguntas abertas

1. A convenção `q` horizontal com `r` limitado produz exatamente a topologia visual pretendida nas linhas polares, para alturas pares e ímpares? **Recomendação:** aprovar uma fixture visual/geométrica de cada caso antes de fechar os contratos.
2. Qual o tamanho mínimo/máximo de mundo e limite de tiles? **Recomendação:** definir antes do benchmark e escolher `TileId`/limites de raio com prova de overflow.
3. Quais dados de tile são canônicos e quais são derivados, especialmente bioma, fertilidade e clima? **Recomendação:** manter dados-base canônicos e derivar bioma por catálogo versionado, salvo necessidade comprovada de estado histórico.
4. Qual codec/forma de chunk e granularidade de revisão? **Recomendação:** começar com chunks completos versionados e medir; adicionar delta apenas quando custo de rede justificar.
5. Qual formato de rio em arestas e quais regras de conectividade/fairness definem mapa válido? **Recomendação:** fechar no GDD de mapa antes da implementação, com fixtures que cubram desembocadura, lago e wrap.
6. Quais metas reais de tempo e memória valem para servidor e cliente? **Recomendação:** manter as metas deste documento como hipóteses de spike até o tamanho máximo de mundo e hardware serem decididos.
7. Como preservar saves se catálogo ou algoritmo mudar? **Recomendação:** persistir versões de schema, gerador e catálogo no snapshot; migrações devem ser explícitas e cobertas por ADR quando alterarem semântica.
