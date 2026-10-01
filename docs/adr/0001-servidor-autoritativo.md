# ADR-0001 — Servidor autoritativo, distribuído no próprio repositório

- **Status**: Aceito
- **Data**: 2026-10-01
- **Decisores**: usuário

## Contexto

O jogo tem mundo persistente, várias civilizações (jogadores e bots), Governadores que agem quando o
jogador não está jogando e uma Entropia por mundo. Tudo isso exige processamento contínuo e chamadas
de IA que um celular não sustenta. Um modo totalmente local no celular não é objetivo do projeto.

## Decisão

- A simulação roda em um **servidor autoritativo**. O cliente (celular) apenas exibe o estado e envia
  ações; nunca é fonte da verdade.
- O servidor faz parte deste repositório e é **auto-hospedável**: qualquer pessoa (ou o próprio
  desenvolvedor, em dev) pode subir o servidor numa VPS ou PC e apontar o cliente para ele.
- Cliente e servidor são artefatos separados, com versionamento de protocolo explícito.

## Consequências

- O núcleo de simulação fica isolado do transporte (biblioteca pura) e é usado pelo servidor e pelo
  harness de testes.
- O protocolo cliente-servidor precisa de versionamento e checagem de compatibilidade.
- Há um host de referência para dev/testes (VPS do projeto; ver `infra/README.md`). Em dev, o servidor
  também deve subir localmente com um único comando (Docker Compose).
- Não haverá modo offline jogável no celular.
