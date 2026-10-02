# ADR-0008 — Turno sem relógio: avança quando todos os humanos presentes jogaram

- **Status**: Aceito
- **Data**: 2026-10-01
- **Decisores**: usuário
- **Substitui**: a condição de fechamento do turno do ADR-0003 ("ou quando o tempo do turno expira")
  e a menção a "tempo do turno expira" no ADR-0002.

## Contexto

Os ADRs 0002 e 0003 e o primeiro rascunho do pilar 01 assumiram turnos com prazo em tempo real
(ex.: 12 horas). O usuário esclareceu que o jogo **não é em tempo real**: o turno é a jogada.

## Decisão

- Não existe relógio de turno nem prazo em tempo real.
- Todos jogam o mesmo turno ao mesmo tempo, com ações aplicadas na ordem aceita (ADR-0003 continua
  valendo nisso).
- O turno avança assim que **todos os humanos presentes** jogaram (marcaram Pronto).
- **Humano ausente** (não conectado ao mundo) não é esperado: o **Governador joga por ele** naquele
  turno, dentro do Mandato.
- Bots e Governadores jogam no mesmo turno, em ordem rotativa com seed (pilar 01).

## Consequências

- O mundo anda no ritmo de quem está jogando; ninguém trava o mundo.
- "Presente" precisa de definição exata no SDD (sessão ativa no mundo; o que acontece se a conexão cair
  no meio do turno).
- Um jogador que volta encontra turnos jogados pelo Governador; o relatório de retorno é essencial
  (pilares 09 e 11).
- Prazos dentro do jogo (tratados, propostas, crises) são contados em turnos, nunca em horas.
- Em mundos onde um humano joga sozinho com bots, o mundo só avança quando ele joga.
