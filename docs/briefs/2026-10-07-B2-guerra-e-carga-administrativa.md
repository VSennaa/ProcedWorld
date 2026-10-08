# Brief B2 — Bots que guerreiam e carga administrativa

Decisão (2026-10-07, delegada pelo usuário: "a partir de agora pode tomar essas decisões"): o mundo
dos bots está pacífico demais (B1: 0 tensões, 0 guerras em 3 seeds; pressão 4 só da coesão). Entram:

## 1. Bots que podem guerrear (GDD 07, GDD 09)
- **Personalidades** de bot (só bots e Governadores de bots, nunca o Mandato de um jogador humano):
  derivadas da seed do mundo, de forma reprodutível, ex.: cauteloso, mercantil, expansionista,
  belicoso. Só as cautelosas mantêm a linha vermelha NoStartWar; as outras podem declarar guerra
  **com casus belli registrado no Ledger** (grounding obrigatório, CLAUDE.md §2.5).
- **Disputas de fronteira** como fonte de tensão: territórios que se tocam e tile reivindicado por
  duas civilizações geram entrada `border.disputed` no Ledger e tensão; agressão, quebra de tratado e
  disputa não resolvida por N turnos levam a tensão → guerra, sempre pela máquina de estados existente.
- Guerra com objetivo (já existe em `diplomacy.rs`); paz por cansaço/negociação para não virar guerra
  eterna; o termo `−2·conflitos_ativos` da coesão (GDD 12) passa a valer.
- A auditoria diplomática do harness continua exigindo 0 ações sem grounding.

## 2. Carga administrativa (GDD 06, SDD 15)
- A estabilidade `S` da cidade cai com a distância (hexes) até a capital e com o número de cidades da
  civilização acima de um limite; regra inteira, determinística, com pesos provisórios documentados.
- Capital fixa = primeira cidade (ou a escolhida por regra existente, se houver).

## Critérios
- Harness 8 civs × 300 turnos em 3 seeds (20261001, 42, 7): pressão média ≥ 10 e ≤ 80, ≤ 2 colapsos
  por seed, guerras e tensões > 0, 0 ações diplomáticas sem grounding. Se ≥ 10, o piso volta a ser
  asserção. Registrar antes/depois em `engine/DIAGNOSTICO-P7.md` ("B2") e as regras no GDD 06/07/12 e
  SDD 15 como "decidido em 2026-10-07 por delegação do usuário".
- Testes de unidade, determinismo e replay verdes. Rust só na VPS; não reformatar arquivos.
