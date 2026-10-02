# Brief Q4 — Síntese dos estudos em propostas

## Tarefa
Ler os 4 estudos em `docs/research/` e escrever **somente** `docs/research/SINTESE.md`:
- Tabela: recomendação | estudo(s) de origem | pilar do GDD e/ou SDD afetado | mudança proposta
  (concreta) | impacto (alto/médio/baixo) | contradiz algo decidido? (sim/não, qual).
- Separe "reforça o que já está decidido" de "propõe mudança" de "lacuna nova".
- Top 10 propostas para o usuário avaliar, em ordem de impacto, cada uma com a pergunta a fazer a ele.
- Não altere GDD, SDD nem ADR. Não repita os estudos; aponte seções.

Arquivo permitido: `docs/research/SINTESE.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada fora do que a tarefa pede. Nada de segredos, IPs ou hostnames.
- Ao final, rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
