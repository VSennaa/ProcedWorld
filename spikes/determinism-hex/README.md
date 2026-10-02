# Spike: determinismo entre máquinas (ADR-0007)

Spike descartável (CLAUDE.md §4): vive só na branch `spike/engine/determinism-hex` e nunca é mergeado.
O aprendizado vai para o ADR-0007 e para `docs/sdd/14-testes.md`.

## O que testa

- PRNG xoshiro256** com seed via splitmix64 e sub-seeds por etapa (`Rng::derive`).
- Grid hexagonal pointy-top em cilindro (offset odd-r, wrap horizontal, polos fechados) com
  vizinhança em ordem canônica e distância com wrap.
- Mapa por ruído inteiro e uma simulação simplificada (rendimento, crescimento, expansão, "seca"
  sorteada) só com inteiros e ordem canônica.
- Hash FNV-1a de uma serialização canônica (sem `DefaultHasher`, sem floats, sem mapas).

## Como rodar

```bash
cargo test
cargo run --release -- 20261001 1000
```

## Resultados

Ver a seção de resultados no ADR-0007 (preenchida ao final do spike).
