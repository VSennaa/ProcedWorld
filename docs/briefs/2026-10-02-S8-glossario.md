# Brief S8 — Glossário sincronizado

## Tarefa
Atualizar `docs/GLOSSARY.md` com os termos novos dos SDDs e do GDD 12/13 (ex.: AcceptedCommand,
accepted_sequence, GroundingRef, DomainEvent, StateHash, WorldSnapshot, RulesetRef, IntentEvidence,
PresencePolicy, SnapshotPolicy, Legitimidade, Estabilidade local, Pressão de crise por cidade/civilização,
Prática, Legado, Fenômeno mágico, Interferência na Entropia), uma linha cada, em PT-BR, mantendo a tabela.
Arquivo permitido: `docs/GLOSSARY.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
