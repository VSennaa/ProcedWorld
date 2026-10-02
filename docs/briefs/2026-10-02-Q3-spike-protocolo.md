# Brief Q3 — Spike: serialização do protocolo (Rust)

## Contexto
Branch descartável `spike/back/protocol-serialization` (nunca mergeada). Responde à pergunta do limite de
chunk (48 KiB provisório) e ao formato do protocolo (`docs/sdd/10-protocolo.md`, `docs/sdd/02-hex-e-mapa.md`).
O `cargo` funciona nesta máquina (Rust 1.99, MSVC).

## Tarefa
Criar `spikes/protocol-serialization/` (crate Rust com `Cargo.toml` próprio, sem workspace) que:
- modela um chunk de mapa 8×8 hexágonos com as camadas do GDD 02 §2 e um diff de turno típico
  (20–60 mudanças de tile, 8 civilizações, 10–30 comandos aceitos);
- serializa com **JSON (serde_json)**, **MessagePack (rmp-serde)** e **bincode**, com e sem compressão
  (`flate2`/gzip e `zstd` se compilar sem problemas);
- mede tamanho em bytes e tempo médio de encode/decode (1.000 repetições, build release);
- grava `spikes/protocol-serialization/RESULTS.md` com tabela, ambiente e recomendação (formato e limite
  de chunk sugerido), deixando claro que é medição local e não benchmark Android.
Inclua `.gitignore` com `target/`. Rode `cargo run --release` e `cargo test` até passar.

Arquivos permitidos: `spikes/protocol-serialization/**`. Pode baixar crates do crates.io via cargo.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada fora do que a tarefa pede. Nada de segredos, IPs ou hostnames.
- Ao final, rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
