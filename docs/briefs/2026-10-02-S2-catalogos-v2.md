# Brief S2 — Catálogos v2: unidades, edifícios, melhorias e mais eventos

## Tarefa
Ampliar `data/catalogs/` (rascunho) seguindo a DSL de `docs/sdd/18-dsl-catalogos.md` e o validador
existente `tools/catalogs/validate.py`:
- `units.json` (8–14 unidades: exploração, defesa, ataque, colono, trabalhador, comércio; custo,
  manutenção, movimento, força, requisito tecnológico; sem multiplicadores por era — GDD 01, 03, 05);
- `buildings.json` (12–20 edifícios: uma função básica por cidade, nível substitui o anterior — GDD 04);
- `improvements.json` (melhorias de tile por bioma e recurso — GDD 02, 03);
- `event_templates.json`: chegar a **24 eventos** (hoje 12), cobrindo clima, tecnologia, diplomacia,
  revolta, epidemia, magia, colapso/renascimento e mudança de relevo, respeitando as regras de justiça
  do GDD 08 (sem colapso por evento, resposta útil obrigatória).
- Atualizar `tools/catalogs/validate.py` para os novos arquivos e referências cruzadas, e
  `data/catalogs/README.md`. Rode o validador até passar.
Arquivos permitidos: `data/catalogs/**`, `tools/catalogs/**`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
