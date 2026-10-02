# Resultados — spike de serialização do protocolo

> Medição local em build `--release`; não é benchmark Android, nem representa latência de rede.

## Cenários

- **Chunk 8×8:** 64 `KnownTileView` com as camadas do GDD 02 §2.
- **Diff típico:** 40 mudanças de tile, resumo de 8 civilizações e 20 comandos aceitos.
- Cada média mede 1.000 repetições; compressão faz parte do encode/decode quando indicada.

## Ambiente

- Rust 1.x estável no container oficial `rust:1-slim` (Linux x86_64), `cargo run --release`.
- Dependências: serde_json, rmp-serde, bincode, flate2/gzip e zstd.
- Máquina: VPS de referência (2 vCPU, 2 GB), dentro de Docker; no PC do desenvolvedor o Smart App
  Control do Windows bloqueia executáveis recém-compilados (build scripts). Tempos servem só para
  comparação relativa.

## Medições

| Payload | Formato | Compressão | Bytes | Encode médio | Decode médio |
| --- | --- | --- | ---: | ---: | ---: |
| Chunk 8×8 | JSON | sem | 12775 | 34 µs | 50 µs |
| Diff típico | JSON | sem | 8973 | 14 µs | 30 µs |
| Chunk 8×8 | JSON | gzip | 1503 | 381 µs | 70 µs |
| Diff típico | JSON | gzip | 1113 | 288 µs | 58 µs |
| Chunk 8×8 | JSON | zstd | 1352 | 101 µs | 69 µs |
| Diff típico | JSON | zstd | 1089 | 79 µs | 46 µs |
| Chunk 8×8 | MessagePack | sem | 9385 | 18 µs | 30 µs |
| Diff típico | MessagePack | sem | 6685 | 11 µs | 19 µs |
| Chunk 8×8 | MessagePack | gzip | 1475 | 180 µs | 55 µs |
| Diff típico | MessagePack | gzip | 1062 | 113 µs | 44 µs |
| Chunk 8×8 | MessagePack | zstd | 1225 | 95 µs | 47 µs |
| Diff típico | MessagePack | zstd | 909 | 74 µs | 31 µs |
| Chunk 8×8 | bincode | sem | 1780 | 2 µs | 4 µs |
| Diff típico | bincode | sem | 1550 | 1 µs | 3 µs |
| Chunk 8×8 | bincode | gzip | 841 | 97 µs | 22 µs |
| Diff típico | bincode | gzip | 644 | 118 µs | 20 µs |
| Chunk 8×8 | bincode | zstd | 782 | 61 µs | 15 µs |
| Diff típico | bincode | zstd | 594 | 55 µs | 12 µs |

## Recomendação

Manter **48 KiB descompactados** como limite experimental inicial para `MapChunk`: o cenário de
64 tiles deve ficar substancialmente abaixo dele em todos os formatos testados, preservando
margem para campos autorizados futuros. O servidor deve continuar a validar esse teto antes de
comprimir e paginar/reduzir o chunk quando ele for excedido.

Para a primeira implementação, usar **JSON UTF-8 com gzip negociado**: é o formato proposto no
SDD 10, é inspecionável e a compressão reduz repetição de nomes/campos. MessagePack e bincode
são candidatos de evolução somente com versionamento explícito do contrato e novo teste em
dispositivos Android; bincode não é formato de rede estável por si só.

O diff medido é uma amostra fixa, não uma distribuição de partidas reais. Recalibrar os limites
com traces autorizados do harness e hardware móvel de referência antes de ratificar o SDD.

## Achado de design (supervisor, 2026-10-02)

O primeiro rascunho usava `Option<Option<T>>` em `TileChange` para distinguir "não mudou" de "mudou
para nenhum". **Em JSON isso quebra**: `None` e `Some(None)` viram ambos `null`, e o round trip perde a
mudança silenciosamente (o teste pegou). O spike passou a usar um tipo explícito de três estados,
`Change<T> { Keep, Clear, Set(T) }`, que funciona em JSON, MessagePack e bincode. Recomendação para o
SDD 10: **proibir `Option<Option<_>>` em contratos de rede** e usar patch explícito.
