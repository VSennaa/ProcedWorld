# 12 — Variáveis e fórmulas compartilhadas

> Fonte única das variáveis que atravessam pilares. Quando um pilar e este arquivo divergirem,
> **vale este arquivo**; o pilar deve ser corrigido. Todos os números são **valores iniciais,
> sujeitos a balanceamento** no harness de simulação, exceto onde o "Decidido" de um pilar diz outra coisa.
> Aritmética inteira; `limitar(a, b, x)` = `min(b, max(a, x))`; `teto` = arredondar para cima.

## Por que este arquivo existe

Os pilares foram redigidos em paralelo e definiram as mesmas grandezas de formas diferentes
(privação em 03 e 04; atualização de coesão em 04 e 06; pressão de crise em 00 e 06). A consolidação
de 2026-10-01 escolheu uma definição para cada uma, preservando as decisões do usuário.

## Tabela de variáveis

| Símbolo | Nome | Escala | Escopo | Definida em | Usada em |
|---|---|---|---|---|---|
| `C` | Coesão social | 0–100 | civilização | 06 (regra), 00 (conceito) | 00, 01, 04, 08, 09, 10, 11 |
| `L` | Legitimidade | 0–100 | civilização | 06 | 06, 09 |
| `S` | Estabilidade local | 0–100 | cidade | 04 | 04, 06, P |
| `A_g` / `T_g` | Satisfação / tensão do grupo (`T_g = 100 − A_g`) | 0–100 | grupo por cidade | 04, 06 | 04, 06, G |
| `D` | Privação | 0–20 | cidade e civilização | este arquivo (unifica 03 e 04) | P, 05, 08 |
| `G` | Tensão de grupos | 0–20 | cidade e civilização | este arquivo (de 04) | P |
| `W` | Ameaça de guerra | 0–20 | cidade e civilização | este arquivo (provisório) | P, 07 |
| `E` | Exposição ambiental | 0–20 | cidade e civilização | este arquivo (provisório) | P, 02, 05, 08 |
| `P` | Pressão de crise | 0–100 | cidade e civilização | este arquivo (unifica 00 e 06) | 01, 04, 05, 06, 08, 09, 10, 11 |
| `Cf`, `R`, `Dv` | Confiança, ressentimento, dívida (por direção) | 0–100, 0–100, −100..100 | par de civilizações | 07 | 03, 05, 07, 08 |
| `A` | Aceitação de proposta | 0–100 | proposta | 07 | 07 |
| `U` | Utilidade de ação do Governador | inteiro | ação candidata | 09 | 09 |

Letras não podem ser reaproveitadas com outro sentido em novos pilares; use sufixos (`Cop`, `Bseg`...).

## Privação `D` (unifica 03 e 04)

- Por cidade: `D_c = min(20, 5 × unidades_sem_comida_c + M_c)`, com `M_c = 5` se alguma manutenção
  essencial da cidade não foi paga no turno, senão `0`.
- Por civilização: `D_civ = limitar(0, 20, teto(Σ(pop_c × D_c) / Σ pop_c))`; população zero ⇒ `0`.
- Substitui a fórmula por falta total de 03 e a fórmula sem manutenção de 04. A falta de comida
  continua com causa numérica rastreável (03) e por cidade (04).

## Tensão de grupos `G`

- Por cidade: `G_c = limitar(0, 20, teto(20 × pop_em_grupos_com_A_g<40_c / pop_c))` (de 04).
- Por civilização: média ponderada por população de `G_c`.

## Ameaça de guerra `W` e exposição ambiental `E` (provisórias)

Nenhum pilar fechou essas fórmulas. Proposta para o SDD validar no harness:
- `W_c = min(20, 4 × confrontos_vizinhos_ou_no_raio_3 + 6 × [guerra ativa com quem faz fronteira com a cidade])`.
- `E_c = min(20, soma, nos tiles trabalhados, de modificadores climáticos negativos ativos ÷ 5)`;
  práticas de 05 reduzem `E` (teto de −4 por turno).

## Pressão de crise `P` (unifica 00 e 06; decisões de 2026-10-01)

- Por cidade: `P_c = limitar(0, 100, 2·D_c + 2·G_c + W_c + E_c + (100 − S_c) / 10 − (C − 50) / 5)`.
  **Decidido em 2026-10-02**: o termo de coesão é **centrado em 50** (antes `− C / 5`, que tirava 10 pontos
  com coesão média e impedia crises mesmo com a Entropia ativa — medido no P12). Coesão 50 é neutra;
  80 alivia 6; 20 soma 6.
  - O termo `(100 − S_c)/10` (0–10) é a estabilidade local com **peso reduzido**, decidido pelo usuário.
    Ele pesa até metade de `W` ou `E` porque `S` já reflete a satisfação dos grupos que entra por `G`.
  - `C` é a coesão da civilização (0–100); `(C − 50)/5` vai de −10 a +10.
- Por civilização: `P_civ = média ponderada por população de P_c`.
- Limiares (00, 01, 06): `P` 40–69 ⇒ aviso e decisão de mitigação; `P ≥ 70` por 2 turnos ⇒ crise
  (local para `P_c`, nacional para `P_civ`); `P_c ≥ 85` com `L < 30` ⇒ revolta elegível (06).

## Coesão `C` (unifica 04 e 06)

Regra única, a de 06, por turno:
`C' = limitar(0, 100, C + 4·compromissos_cumpridos − 5·compromissos_quebrados − média_ponderada(T_g)/10 − 2·conflitos_ativos + Δ_luxo)`,
com cada termo limitado por turno (06) e `Δ_luxo` vindo de 03 (+2 pela primeira origem de luxo
consumida, +1 pela segunda, suspenso com `D_civ ≥ 15`).

- A regra de 04 que somava −2/+1 à coesão pela estabilidade urbana média **foi removida**: ela contava
  de novo a satisfação dos grupos, que já entra por `média_ponderada(T_g)`.
- Colapso: `C = 0` por 2 turnos, ou crise grave validada que deixe o governo incapaz por 3 turnos (10).

## Eras e legados (resumo das decisões)

- Era por marcos, de 4 a 12 turnos (média esperada 6–8) — 10.
- No máximo 2 legados ativos no total (instituição, patrimônio, memória social ou tecnologia
  preservada, que conta como instituição) — 05, 10, 11.

## Pendências para o SDD

- Validar no harness se os pesos de `P` produzem crises na frequência desejada (nem crise
  permanente, nem mundo sem pressão).
- Fechar `W` e `E` com dados de simulação.
- Fórmula de proporção em contratos mistos (03).
