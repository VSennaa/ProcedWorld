# ADR-0004 — Mapa em grid hexagonal

- **Status**: Aceito
- **Data**: 2026-10-01
- **Decisores**: usuário

## Contexto

O formato do tile afeta movimento, adjacência, geração procedural, renderização e UI de toque.

## Decisão

Mapa em **hexágonos**. Representação interna em coordenadas axiais/cúbicas (q, r), com orientação
(pointy-top vs. flat-top) e topologia de borda (wrap horizontal, cilindro, etc.) definidas no GDD/SDD.

## Consequências

- Distâncias e vizinhança uniformes (6 vizinhos), sem o problema das diagonais do grid quadrado.
- O motor expõe as primitivas de hex (vizinhos, distância, linha, anel, área) como código puro e testado.
- O cliente precisa de TileMap hexagonal e de seleção por toque com área confortável.
