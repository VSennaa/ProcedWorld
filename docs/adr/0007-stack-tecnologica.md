# ADR-0007 — Stack tecnológica

- **Status**: **Proposto** (ratificar na Fase 1 — SDD)
- **Data**: 2026-10-01
- **Decisores**: usuário (pendente)

## Contexto

Requisitos: determinismo bit-a-bit, desempenho em simulações longas, servidor auto-hospedável em VPS
pequena (2 vCPU / 2 GB), cliente mobile com tiles hexagonais e desenvolvimento majoritariamente
feito por agentes.

## Proposta

| Parte | Proposta | Motivo | Alternativa |
|---|---|---|---|
| Núcleo de simulação | **Rust** (crate puro, `no I/O`) | determinismo controlável, desempenho, tipos fortes | TypeScript |
| Servidor | Rust (axum + tokio), WebSocket + HTTP | mesmo idioma do núcleo, binário único e leve | Node/TS |
| Persistência | PostgreSQL (estado, log, snapshots) | confiável, sem porta publicada (ver `infra/`) | SQLite |
| Memória de IA | Markdown + SQLite FTS5 (no estilo ai-memory) | fonte legível + busca barata | só Postgres |
| Cliente | **Godot 4** (export Android/iOS, TileMap hexagonal) | engine 2D madura, MCP `godot-ai` disponível | Flutter/Flame |
| Infra | Docker Compose, proxy reverso com TLS, GitHub Actions | já padronizado na VPS | — |

## Pontos a validar antes de aceitar (spikes)

- Tempo e memória de compilação do Rust na VPS de 2 GB (provável necessidade de swap ou build no CI).
- Contrato do protocolo entre Godot e o servidor (JSON vs. binário; geração de tipos).
- Determinismo do núcleo compilado em x86_64 e ARM.
