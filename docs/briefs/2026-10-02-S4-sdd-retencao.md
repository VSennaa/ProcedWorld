# Brief S4 — SDD 19: retenção e eliminação de dados

## Tarefa
Escrever `docs/sdd/19-retencao-de-dados.md` (rascunho), fechando a lacuna apontada em
`docs/sdd/REVISAO-CRUZADA.md` e em `docs/research/SINTESE.md`: classificação de dados (log de comandos,
snapshots, texto de jogador, evidências e respostas de IA, Crônica, memória, chaves BYOK, telemetria,
backups), prazos, expurgo verificável, exportação, efeito em replay (o log mecânico não pode perder
determinismo), backups e exclusão de conta. Formato igual aos outros SDDs (responsabilidades,
contratos, invariantes, falhas, testes, perguntas com "Recomendação:").
Arquivo permitido: `docs/sdd/19-retencao-de-dados.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
