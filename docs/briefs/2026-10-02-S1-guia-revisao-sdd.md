# Brief S1 — Guia de revisão do SDD para o dono do projeto

## Tarefa
O dono vai revisar os SDDs de 2 em 2, respondendo perguntas de múltipla escolha (como fez com o GDD).
Escrever **somente** `docs/sdd/GUIA-DE-REVISAO.md`:
- Pares de revisão em ordem de dependência (ex.: 00+01, 02+16, 03+17, 04+08, 05+06, 07+15, 09+14,
  10+12, 11+13, 18 sozinho; inclua 19, 20 e 21 se existirem), com o porquê de cada par.
- Para cada par: resumo de 5 linhas do que o par define; **até 4 perguntas de múltipla escolha** (2–4
  opções cada, a recomendada marcada "(Recomendado)" em primeiro, uma linha de prós/contras por opção)
  sobre as escolhas que de fato precisam do dono; e a lista do que é só técnico e pode ser aceito em bloco.
- Inclua as perguntas abertas de `docs/PERGUNTAS-ABERTAS.md` e o top 10 de `docs/research/SINTESE.md`
  no par a que pertencem, sem duplicar.
Arquivo permitido: `docs/sdd/GUIA-DE-REVISAO.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
