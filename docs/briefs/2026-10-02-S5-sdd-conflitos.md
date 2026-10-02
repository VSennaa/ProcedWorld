# Brief S5 — SDD 20: matriz de conflitos de comandos

## Tarefa
Escrever `docs/sdd/20-matriz-de-conflitos.md` (rascunho): por classe de comando (movimento, obra,
gasto de reserva, comércio/contrato, diplomacia, política, pesquisa, ataque), quais conflitam entre si,
como são resolvidos (ordem de aceitação do ADR-0003; ataque em fase própria com perdas simultâneas,
decidido no GDD 01), desempate verificável por seed, rejeição sem custo e como o cliente comunica.
Inclua a ordem canônica das fases e exemplos trabalhados. Não contradiga ADR-0003/0008 nem o GDD 01.
Arquivo permitido: `docs/sdd/20-matriz-de-conflitos.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
