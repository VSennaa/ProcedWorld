# 02 — Mapa e tiles

## Decidido
- Grid hexagonal, coordenadas axiais/cúbicas (ADR-0004).
- Mapa gerado proceduralmente a partir de uma seed.

## Proposta
- Camadas por tile: elevação, bioma, umidade, temperatura, recurso, feature (rio, floresta), melhoria, dono.
- Clima que varia ao longo das eras (alimenta eventos da Entropia: secas, glaciações, cheias).

## Perguntas abertas
- Orientação (pointy-top ou flat-top) e borda (wrap horizontal, cilindro, ilha fechada).
- Tamanho do mapa por número de civilizações.
- Névoa de guerra e exploração: como funcionam para o Governador?
