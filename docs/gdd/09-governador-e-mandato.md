# 09 — Governador e Mandato

## Decidido
- O Governador é um bot do jogo, roda no servidor, e o jogador o configura pelo Mandato (ADR-0002).
- O motor bloqueia ações que violem o Mandato.

## Proposta
- **Mandato** em camadas: prioridades ponderadas (crescer, defender, pesquisar, comerciar...),
  proibições (não declarar guerra, não vender o recurso X), limites (gasto máximo, tropas mínimas),
  postura diplomática, gatilhos de notificação ("me avise se houver guerra").
- **Escopo de delegação**: tudo, ou só algumas áreas (ex.: cidades sim, diplomacia não).
- **Relatório do Governador**: ao voltar, o jogador lê o que foi feito e por quê (grounding).

## Perguntas abertas
- Presets de Mandato para quem não quer configurar nada?
- O Governador pode sugerir mudanças no Mandato?
