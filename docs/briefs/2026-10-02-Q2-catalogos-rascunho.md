# Brief Q2 — Catálogos de conteúdo em dados (rascunho)

## Tarefa
Criar os catálogos iniciais em JSON, seguindo a linguagem fechada proposta em `docs/sdd/18-dsl-catalogos.md`
(predicados, seletores, efeitos) e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`. Fonte de
conteúdo: GDD 02 (biomas, recursos), 03 (economia), 05 (tecnologia), 06 (governo por 3 eixos), 08
(Entropia, 5 eventos de referência), 00/05/06/08 (magia).

Gerar em `data/catalogs/`:
- `biomes.json` (12 biomas do GDD 02 com rendimentos-base e custo de movimento),
- `resources.json` (renováveis, finitos, estratégicos/luxo; metal),
- `tech_tree.json` (árvore base nos ramos sustento, organização, circulação; 15–25 tecnologias com
  pré-requisitos, custo 12–30, prática e efeito limitado aos tetos do GDD 05),
- `governments.json` (3 eixos × 3 posições e políticas),
- `magic_phenomena.json` (**3–5** fenômenos raros e sistêmicos: gatilho da Entropia, como aderir,
  proibir, regulamentar, práticas mágicas, riscos),
- `event_templates.json` (os 5 eventos de referência do GDD 08 + pelo menos 7 novos, incluindo
  **3 ações de interferir na Entropia** que custam rituais, sacrifícios ou pesquisas proibidas, com risco
  catastrófico sorteado pela seed e crescente com a ambição),
- `README.md` explicando cada arquivo, que tudo é **rascunho sujeito a aprovação e balanceamento**.

E criar `tools/catalogs/validate.py` (Python 3, só biblioteca padrão) que valida: JSON bem formado,
ids únicos, referências existentes (pré-requisitos, recursos, biomas), faixas numéricas dentro dos
tetos do GDD e que efeitos só usam operações da DSL. Rode o validador até passar.

Arquivos permitidos: `data/catalogs/**`, `tools/catalogs/**`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada fora do que a tarefa pede. Nada de segredos, IPs ou hostnames.
- Ao final, rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
