use std::error::Error;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const ITERATIONS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MapChunk {
    chunk_id: ChunkId,
    revision: u64,
    schema_version: u16,
    tiles: Vec<KnownTileView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ChunkId {
    q: i16,
    r: i16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct KnownTileView {
    tile_id: u32,
    q: i16,
    r: i16,
    elevation: i8,
    temperature: u8,
    moisture: u8,
    biome_id: u16,
    fertility: u8,
    resource: Option<ResourceView>,
    feature_bits: u32,
    improvement_id: Option<u16>,
    owner_id: Option<u16>,
    visibility: Visibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ResourceView {
    kind_id: u16,
    stock: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Visibility {
    Unknown,
    Remembered,
    Visible,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TurnDiff {
    from_cursor: u64,
    to_cursor: u64,
    turn: u32,
    changed_tiles: Vec<TileChange>,
    civilizations: Vec<CivilizationSummary>,
    accepted_commands: Vec<AcceptedCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TileChange {
    tile_id: u32,
    temperature: Option<u8>,
    moisture: Option<u8>,
    biome_id: Option<u16>,
    fertility: Option<u8>,
    feature_bits: Option<u32>,
    improvement_id: Change<u16>,
    owner_id: Change<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CivilizationSummary {
    civilization_id: u16,
    food: i32,
    production: i32,
    wealth: i32,
    science: i32,
    culture: i32,
    stability: i16,
    known_tiles: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AcceptedCommand {
    command_id: u64,
    accepted_sequence: u64,
    civilization_id: u16,
    kind: CommandKind,
    target_tile_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum CommandKind {
    MoveUnit,
    ImproveTile,
    FoundCity,
    Research,
    Diplomacy,
}

/// Tri-state field patch. `Option<Option<T>>` cannot be used in diffs: JSON encodes both
/// "unchanged" (None) and "cleared" (Some(None)) as null, so the round trip silently loses a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Change<T> {
    Keep,
    Clear,
    Set(T),
}

#[derive(Clone, Copy, Debug)]
enum Codec {
    Json,
    MessagePack,
    Bincode,
}

#[derive(Clone, Copy, Debug)]
enum Compression {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug)]
struct Measurement {
    payload: &'static str,
    codec: Codec,
    compression: Compression,
    bytes: usize,
    encode_average: Duration,
    decode_average: Duration,
}

fn main() -> Result<(), Box<dyn Error>> {
    let chunk = representative_chunk();
    let diff = representative_diff();
    let combinations = [
        (Codec::Json, Compression::None),
        (Codec::Json, Compression::Gzip),
        (Codec::Json, Compression::Zstd),
        (Codec::MessagePack, Compression::None),
        (Codec::MessagePack, Compression::Gzip),
        (Codec::MessagePack, Compression::Zstd),
        (Codec::Bincode, Compression::None),
        (Codec::Bincode, Compression::Gzip),
        (Codec::Bincode, Compression::Zstd),
    ];

    let mut measurements = Vec::new();
    for (codec, compression) in combinations {
        measurements.push(measure("Chunk 8×8", &chunk, codec, compression)?);
        measurements.push(measure("Diff típico", &diff, codec, compression)?);
    }

    let report = results_markdown(&measurements);
    std::fs::write("RESULTS.md", &report)?;
    print!("{report}");
    Ok(())
}

fn representative_chunk() -> MapChunk {
    let tiles = (0..64)
        .map(|index| {
            let q = index % 8;
            let r = index / 8;
            KnownTileView {
                tile_id: 1_024 + index,
                q: q as i16,
                r: r as i16,
                elevation: ((index % 11) as i8) - 2,
                temperature: 35 + ((index * 7) % 60) as u8,
                moisture: 20 + ((index * 11) % 75) as u8,
                biome_id: (index % 12) as u16,
                fertility: 15 + ((index * 13) % 80) as u8,
                resource: (index % 5 == 0).then(|| ResourceView {
                    kind_id: (index % 9) as u16,
                    stock: 100 + (index * 3) as u16,
                }),
                feature_bits: if index % 7 == 0 { 0b101 } else { (index % 4) as u32 },
                improvement_id: (index % 6 == 0).then_some((index % 10) as u16),
                owner_id: (index % 3 != 0).then_some((index % 8) as u16),
                visibility: if index % 9 == 0 {
                    Visibility::Remembered
                } else {
                    Visibility::Visible
                },
            }
        })
        .collect();

    MapChunk {
        chunk_id: ChunkId { q: 16, r: 24 },
        revision: 42,
        schema_version: 1,
        tiles,
    }
}

fn representative_diff() -> TurnDiff {
    let changed_tiles = (0..40)
        .map(|index| TileChange {
            tile_id: 1_024 + index * 3,
            temperature: (index % 2 == 0).then_some(40 + index as u8),
            moisture: (index % 3 == 0).then_some(55 + index as u8),
            biome_id: (index % 5 == 0).then_some((index % 12) as u16),
            fertility: (index % 4 == 0).then_some(60 - index as u8),
            feature_bits: (index % 7 == 0).then_some(0b1_001),
            improvement_id: if index % 8 == 0 { Change::Set((index % 10) as u16) } else { Change::Keep },
            owner_id: if index % 9 == 0 { Change::Clear } else { Change::Keep },
        })
        .collect();
    let civilizations = (0..8)
        .map(|civilization_id| CivilizationSummary {
            civilization_id,
            food: 120 + civilization_id as i32 * 9,
            production: 95 + civilization_id as i32 * 7,
            wealth: 80 + civilization_id as i32 * 11,
            science: 45 + civilization_id as i32 * 3,
            culture: 30 + civilization_id as i32 * 5,
            stability: 70 + civilization_id as i16,
            known_tiles: 180 + civilization_id * 12,
        })
        .collect();
    let accepted_commands = (0..20)
        .map(|index| AcceptedCommand {
            command_id: 9_000 + index,
            accepted_sequence: 7_000 + index,
            civilization_id: (index % 8) as u16,
            kind: match index % 5 {
                0 => CommandKind::MoveUnit,
                1 => CommandKind::ImproveTile,
                2 => CommandKind::FoundCity,
                3 => CommandKind::Research,
                _ => CommandKind::Diplomacy,
            },
            target_tile_id: (index % 5 != 3).then_some((1_024 + index * 2) as u32),
        })
        .collect();

    TurnDiff {
        from_cursor: 71_000,
        to_cursor: 71_068,
        turn: 128,
        changed_tiles,
        civilizations,
        accepted_commands,
    }
}

fn measure<T>(
    payload: &'static str,
    value: &T,
    codec: Codec,
    compression: Compression,
) -> Result<Measurement, Box<dyn Error>>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let encoded = encode(value, codec, compression)?;
    let encode_started = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = encode(value, codec, compression)?;
    }
    let encode_average = encode_started.elapsed() / ITERATIONS as u32;

    let decode_started = Instant::now();
    for _ in 0..ITERATIONS {
        let decoded: T = decode(&encoded, codec, compression)?;
        assert_eq!(&decoded, value);
    }
    let decode_average = decode_started.elapsed() / ITERATIONS as u32;

    Ok(Measurement {
        payload,
        codec,
        compression,
        bytes: encoded.len(),
        encode_average,
        decode_average,
    })
}

fn encode<T: Serialize>(
    value: &T,
    codec: Codec,
    compression: Compression,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let serialized = match codec {
        Codec::Json => serde_json::to_vec(value)?,
        Codec::MessagePack => rmp_serde::to_vec_named(value)?,
        Codec::Bincode => bincode::serialize(value)?,
    };
    match compression {
        Compression::None => Ok(serialized),
        Compression::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&serialized)?;
            Ok(encoder.finish()?)
        }
        Compression::Zstd => Ok(zstd::stream::encode_all(serialized.as_slice(), 0)?),
    }
}

fn decode<T>(
    encoded: &[u8],
    codec: Codec,
    compression: Compression,
) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let serialized = match compression {
        Compression::None => encoded.to_vec(),
        Compression::Gzip => {
            let mut decoder = flate2::read::GzDecoder::new(encoded);
            let mut output = Vec::new();
            decoder.read_to_end(&mut output)?;
            output
        }
        Compression::Zstd => zstd::stream::decode_all(encoded)?,
    };
    Ok(match codec {
        Codec::Json => serde_json::from_slice(&serialized)?,
        Codec::MessagePack => rmp_serde::from_slice(&serialized)?,
        Codec::Bincode => bincode::deserialize(&serialized)?,
    })
}

fn results_markdown(measurements: &[Measurement]) -> String {
    let mut output = String::from(
        "# Resultados — spike de serialização do protocolo\n\n\
        > Medição local em build `--release`; não é benchmark Android, nem representa latência de rede.\n\n\
        ## Cenários\n\n\
        - **Chunk 8×8:** 64 `KnownTileView` com as camadas do GDD 02 §2.\n\
        - **Diff típico:** 40 mudanças de tile, resumo de 8 civilizações e 20 comandos aceitos.\n\
        - Cada média mede 1.000 repetições; compressão faz parte do encode/decode quando indicada.\n\n\
        ## Ambiente\n\n\
        - Rust: 1.99 (MSVC), `cargo run --release`.\n\
        - Dependências: serde_json, rmp-serde, bincode, flate2/gzip e zstd.\n\
        - Máquina: execução local Windows; números de tempo servem apenas para comparação relativa.\n\n\
        ## Medições\n\n\
        | Payload | Formato | Compressão | Bytes | Encode médio | Decode médio |\n\
        | --- | --- | --- | ---: | ---: | ---: |\n",
    );
    for measurement in measurements {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            measurement.payload,
            codec_name(measurement.codec),
            compression_name(measurement.compression),
            measurement.bytes,
            duration_name(measurement.encode_average),
            duration_name(measurement.decode_average),
        ));
    }
    output.push_str(
        "\n## Recomendação\n\n\
        Manter **48 KiB descompactados** como limite experimental inicial para `MapChunk`: o cenário de\n\
        64 tiles deve ficar substancialmente abaixo dele em todos os formatos testados, preservando\n\
        margem para campos autorizados futuros. O servidor deve continuar a validar esse teto antes de\n\
        comprimir e paginar/reduzir o chunk quando ele for excedido.\n\n\
        Para a primeira implementação, usar **JSON UTF-8 com gzip negociado**: é o formato proposto no\n\
        SDD 10, é inspecionável e a compressão reduz repetição de nomes/campos. MessagePack e bincode\n\
        são candidatos de evolução somente com versionamento explícito do contrato e novo teste em\n\
        dispositivos Android; bincode não é formato de rede estável por si só.\n\n\
        O diff medido é uma amostra fixa, não uma distribuição de partidas reais. Recalibrar os limites\n\
        com traces autorizados do harness e hardware móvel de referência antes de ratificar o SDD.\n",
    );
    output
}

fn codec_name(codec: Codec) -> &'static str {
    match codec {
        Codec::Json => "JSON",
        Codec::MessagePack => "MessagePack",
        Codec::Bincode => "bincode",
    }
}

fn compression_name(compression: Compression) -> &'static str {
    match compression {
        Compression::None => "sem",
        Compression::Gzip => "gzip",
        Compression::Zstd => "zstd",
    }
}

fn duration_name(duration: Duration) -> String {
    if duration.as_nanos() < 1_000 {
        format!("{} ns", duration.as_nanos())
    } else if duration.as_micros() < 1_000 {
        format!("{} µs", duration.as_micros())
    } else {
        format!("{} ms", duration.as_millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codecs_round_trip_chunk_and_diff() {
        for codec in [Codec::Json, Codec::MessagePack, Codec::Bincode] {
            for compression in [Compression::None, Compression::Gzip, Compression::Zstd] {
                let chunk = representative_chunk();
                let encoded = encode(&chunk, codec, compression).unwrap();
                let decoded: MapChunk = decode(&encoded, codec, compression).unwrap();
                assert_eq!(decoded, chunk);

                let diff = representative_diff();
                let encoded = encode(&diff, codec, compression).unwrap();
                let decoded: TurnDiff = decode(&encoded, codec, compression).unwrap();
                assert_eq!(decoded, diff, "diff round trip failed for {codec:?} {compression:?}");
            }
        }
    }

    #[test]
    fn representative_payload_counts_match_the_brief() {
        assert_eq!(representative_chunk().tiles.len(), 64);
        let diff = representative_diff();
        assert!((20..=60).contains(&diff.changed_tiles.len()));
        assert_eq!(diff.civilizations.len(), 8);
        assert!((10..=30).contains(&diff.accepted_commands.len()));
    }
}
