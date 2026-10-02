# Brief S6 — SDD 21: notificações push

## Tarefa
Escrever `docs/sdd/21-notificacoes.md` (rascunho): outbox transacional fora do `step`, eventos que
podem notificar (GDD 09 e 11: escolha crítica que vence em poucos turnos, proposta que expira, gatilho
de Mandato, crise iminente; no máximo uma acionável a cada 6 h), deduplicação, preferências, payload
mínimo sem dados sensíveis, FCM no Android a partir de servidor auto-hospedável (leia
`docs/research/integracoes-tecnicas.md`), reconciliação ao abrir o app e testes.
Arquivo permitido: `docs/sdd/21-notificacoes.md`.

## Regras

- Português do Brasil em documentos; identificadores de código em inglês. UTF-8, LF.
- Grave arquivos só com a ferramenta de patch (apply_patch), nunca com PowerShell.
- Não rode git. Não instale nada. Sem rede (o sandbox não tem). Nada de segredos, IPs ou hostnames.
- Não decida pelo dono do projeto: o que não está em "Decidido" (GDD), ADR aceito ou na tabela
  "Decisões do usuário" de `docs/sdd/REVISAO-CRUZADA.md` é **proposta**.
- Use os nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` e as variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Ao final rode `python tools/agents/check-encoding.py` e só termine com "encoding ok"; responda com
  arquivos alterados e um resumo de 5 linhas.
