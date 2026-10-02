# Catálogos iniciais (rascunho)

Todo o conteúdo desta pasta é **rascunho sujeito a aprovação e balanceamento**. Os arquivos seguem a DSL fechada proposta em `docs/sdd/18-dsl-catalogos.md`; não são ainda um contrato de runtime aprovado.

- `biomes.json`: os 12 biomas iniciais, seus rendimentos-base e custos de movimento.
- `resources.json`: rendimentos negociáveis, renováveis, finitos, estratégicos e luxos; inclui metal.
- `tech_tree.json`: 17 tecnologias nos ramos sustento, organização e circulação, com custos, dependências e práticas limitadas.
- `units.json`: 10 unidades de exploração, defesa, ataque, colonização, trabalho e comércio; custo, manutenção, movimento, força e tecnologia são propostas iniciais, sem multiplicadores por era.
- `buildings.json`: 14 edifícios urbanos. Cada entrada exerce uma função básica; cadeias de nível substituem o edifício anterior, sem somar efeitos.
- `improvements.json`: 12 melhorias de tile, limitadas aos seus biomas e recursos compatíveis, com requisito tecnológico e manutenção proposta.
- `governments.json`: os três eixos de governo, suas três posições e políticas associadas.
- `magic_phenomena.json`: quatro fenômenos raros, com gatilho, reação política e práticas mágicas.
- `event_templates.json`: 24 eventos, incluindo os cinco de referência, clima, tecnologia, diplomacia, revolta, epidemia, magia, colapso/renascimento, relevo e três ações de interferência na Entropia. Todo evento oferece ao menos duas respostas mecânicas; colapso permanece consequência de pressão persistente, nunca de um template isolado.

`tools/catalogs/validate.py` valida a forma mecânica deste rascunho: JSON, IDs, referências cruzadas, faixas do GDD e operações fechadas da DSL. Ele confere tecnologias de unidades, edifícios e melhorias, custos por recurso, compatibilidade melhoria–bioma–recurso, substituições de edifícios e cobertura/justiça mínima dos eventos. Ele não substitui o futuro validador Rust nem decide balanceamento.
