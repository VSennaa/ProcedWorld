# Catálogos iniciais (rascunho)

Todo o conteúdo desta pasta é **rascunho sujeito a aprovação e balanceamento**. Os arquivos seguem a DSL fechada proposta em `docs/sdd/18-dsl-catalogos.md`; não são ainda um contrato de runtime aprovado.

- `biomes.json`: os 12 biomas iniciais, seus rendimentos-base e custos de movimento.
- `resources.json`: rendimentos negociáveis, renováveis, finitos, estratégicos e luxos; inclui metal.
- `tech_tree.json`: 17 tecnologias nos ramos sustento, organização e circulação, com custos, dependências e práticas limitadas.
- `governments.json`: os três eixos de governo, suas três posições e políticas associadas.
- `magic_phenomena.json`: quatro fenômenos raros, com gatilho, reação política e práticas mágicas.
- `event_templates.json`: os cinco eventos de referência, sete novos templates e três ações de interferência na Entropia.

`tools/catalogs/validate.py` valida a forma mecânica deste rascunho: JSON, IDs, referências, faixas do GDD e operações fechadas da DSL. Ele não substitui o futuro validador Rust nem decide balanceamento.
