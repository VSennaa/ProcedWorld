# ADR-0007 — Stack tecnológica

- **Status**: **Aceito** em 2026-10-01 (após os spikes abaixo)
- **Data**: 2026-10-01
- **Decisores**: usuário

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

## Resultados dos spikes (turno da noite, 2026-10-01)

Branch `spike/engine/determinism-hex` (descartável, nunca mergeada) e medições na VPS de referência.

| Pergunta | Resultado |
|---|---|
| Determinismo entre máquinas e arquiteturas | **Passou.** Simulação de brinquedo só com inteiros (xoshiro256**, hex pointy-top em cilindro, hash FNV-1a), seed 20261001, 1.000 turnos, 8 civilizações, 1.600 tiles: hash final `c14c36b8a8863813` idêntico na VPS (Linux x86_64/musl), GitHub Actions Linux x86_64, Windows x86_64 e **macOS ARM64** ([run](https://github.com/VSennaa/ProcedWorld/actions/runs/36951065546)). |
| Custo por turno | ~0,6 ms/turno na VPS (2 vCPU) para a simulação de brinquedo; dá margem grande para o motor real. |
| Rust no PC do desenvolvedor | **Bloqueado** pelo Controle de Aplicativos do Windows (Smart App Control) ao executar `cargo.exe`. Não foi contornado (configuração de segurança do usuário). Desenvolvimento Rust local no PC exige o usuário liberar ou usar WSL/VPS/CI. |
| Rust na VPS sem sudo | Funciona para crates puros (alvo musl + `rust-lld`, só no usuário `deploy`). **Não** funciona para um servidor real: build scripts e proc-macros precisam do linker do sistema (`cc`), ausente. |
| Compilar o servidor proposto na VPS (2 GB) | Em container `rust:1-slim` com `-j 2`: axum + tokio + serde + sqlx/postgres compilou em **~157 s**, memória disponível mínima de **316 MB**, binário release de **2,3 MB**, `target/` de 525 MB. Viável, mas apertado. |

**Consequências para a decisão**: a stack Rust é viável e o requisito central (determinismo entre
plataformas, inclusive ARM) foi demonstrado. Recomendação: builds de release no CI (GitHub Actions) e
imagem Docker publicada; na VPS, build em container só para desenvolvimento, preferencialmente com um
swap de 2 GB (exige `sudo` — decisão do usuário). Desenvolvimento local no PC depende de liberar o
`cargo` no Smart App Control ou usar a VPS/WSL.
