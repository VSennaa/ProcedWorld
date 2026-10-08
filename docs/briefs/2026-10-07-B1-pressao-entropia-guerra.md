# Brief B1 — Pressão de crise: Entropia + guerra

Decisão do usuário (2026-10-07, `docs/PERGUNTAS-ABERTAS.md`, "Meta de pressão de crise"): opção (4)
— mais eventos de pressão da Entropia **e** ameaça de guerra (`W`) vinda da diplomacia entrando na
pressão, medindo no harness **antes** de mexer em pesos. Meta: pressão média ≥ 10 no cenário de saúde
(seed 20261001, 8 civilizações, 300 turnos), sem colapso sistêmico (≤ 2 colapsos) e sem estourar 80.

## Entregas

1. **Medir primeiro**: linha de métricas atual do harness e a distribuição dos termos de `P_c`
   (`D`, `G`, `W`, `E`, `S`, `C`) ao longo da partida — registrar em `engine/DIAGNOSTICO-P7.md`,
   seção "B1" (só acrescentar).
2. **`W` pela diplomacia**: a ameaça de guerra da cidade deriva de guerras ativas, tensão e
   proximidade de civilizações hostis no Ledger/máquina de estados (GDD 07, GDD 12), com regra inteira,
   determinística e documentada no SDD 15; hoje `W` fica perto de 0.
3. **Entropia**: eventos que pressionam (`D`, `G`, `E`) escolhidos com a frequência certa dentro do
   orçamento de tensão (GDD 08: justiça e alívio continuam valendo); ajustar parâmetros de catálogo ou
   do diretor, não inventar efeitos fora dos templates validados.
4. **Teste de saúde**: quando a média chegar a ≥ 10 de forma legítima, o piso volta a ser asserção
   (hoje é aviso). Registrar antes/depois no DIAGNOSTICO.
5. Testes de unidade para as regras novas; replay e determinismo intactos.

## Regras

- Rust só na VPS (`tools/dev/vps-test.sh`, `tools/dev/vps-harness.sh`), SSH espaçado ≥ 60 s, builds
  serializados por `flock`. Nunca IP/host em arquivo. Não reformatar arquivos inteiros.
