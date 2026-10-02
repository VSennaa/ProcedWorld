# SDD 20 — Matriz de conflitos de comandos

> **Status: rascunho para revisão do SDD.** Este documento não altera regras de
> domínio, catálogos ou balanceamento. Onde indicado como **proposta**, a regra
> ainda depende de aprovação do usuário e de validação no harness.

## 1. Objetivo e base normativa

Este SDD define como o servidor explica e resolve comandos concorrentes no mesmo
turno. Ele complementa, sem substituir, os contratos de `AcceptedCommand`, o
resolvedor de domínio e o protocolo de sessão.

São decisões já aceitas:

- todos atuam no mesmo turno e as ações são aceitas sequencialmente pelo
  servidor; `accepted_sequence` é global, estritamente crescente e gravado no
  log (ADR-0003 e `REVISAO-CRUZADA.md`);
- não há relógio de jogo: o turno fecha quando os humanos presentes marcam
  Pronto, com automação para ausentes (ADR-0008);
- `DeclareAttack` reserva custo e unidade ao ser aceito; combate ocorre em fase
  própria, com perdas simultâneas por confronto (GDD 01);
- o motor é determinístico: IDs estáveis ordenam coleções e um PRNG explícito,
  versionado e derivado da seed só resolve empates declarados.

Uma rejeição não cria `AcceptedCommand`, não consome ponto, recurso, vaga nem
reserva e não altera o estado. A tentativa e seu motivo podem ser auditados como
telemetria/protocolo, mas não são uma transição mecânica do replay.

## 2. Ordem canônica das fases

| Ordem | Fase | Papel para conflitos |
|---:|---|---|
| 1 | Abertura | Publica o estado e congela os presentes/agendamento do turno. |
| 2 | Entrada | Valida pedidos um a um e, se válidos, atribui `accepted_sequence`; aplica reservas e efeitos administrativos imediatos. |
| 3 | Fechamento | Registra Pronto, ausência, intenções automatizadas e fallback; não reordena comandos já aceitos. |
| 4 | Conflitos | Resolve movimento pendente, tratados que afetam hostilidade e confrontos de ataques declarados. |
| 5 | Sustento | Processa manutenção, produção, consumo e obrigações de contratos. |
| 6 | Entropia | Aplica somente comando de evento já aceito e ainda elegível contra o estado pós-sustento. |
| 7 | Síntese | Atualiza variáveis e elegibilidades, consolida memória e gera relatório/Crônica. |
| 8 | Publicação | Grava hash, snapshot quando aplicável e expõe o próximo estado. |

Na fase 4, a ordem interna proposta é: (a) efeitos diplomáticos pendentes que
alteram hostilidade; (b) movimentos válidos; (c) agrupamento de ataques por
confronto; (d) dano e perdas simultâneas; (e) ocupação, retirada e eventos. Essa
ordem detalhada é **proposta**, exceto a fase própria e a simultaneidade das
perdas. Ela evita que a ordem de chegada transforme um ataque em dano antecipado.

`C`, `L`, `S`, `D`, `G`, `W`, `E`, `P`, `Cf`, `R`, `Dv` e `A` mantêm exatamente
os nomes, escalas e fórmulas de `gdd/12-variaveis-e-formulas.md`; esta matriz não
os recalcula nem introduz variável nova.

## 3. Regra geral de aceitação e desempate

1. O servidor valida um pedido contra o estado e as reservas já aceitas no turno.
2. Se for elegível, cria um `AcceptedCommand` com `command_id`, `turn`,
   `actor_id`, `origin`, `kind`, `payload_canonical`, `grounding`,
   `ruleset_ref` e o próximo `accepted_sequence`.
3. Se dois pedidos exigirem a mesma capacidade exclusiva, o primeiro
   `accepted_sequence` que a reservar vence; o posterior é rejeitado sem custo.
   Não há prioridade por civilização, dispositivo, bot, Governador ou latência
   fora da ordem efetivamente aceita.
4. Se a disputa só puder ser conhecida na resolução (por exemplo, sobreviventes
   de um confronto), o resolvedor usa a chave canônica definida para aquele
   conflito. Empate de valores usa PRNG versionado com seed derivada de
   `SeedRef`, `turn`, `phase` e IDs estáveis das entidades; nunca usa relógio,
   ordem de pacote, thread ou iteração de mapa.

A função exata de derivação, o algoritmo de PRNG e as chaves de cada catálogo são
**proposta de implementação**; devem ser versionados no `RulesetRef` e cobertos
por replay golden. A rotação e o momento de envio de bots/Governadores continuam
determinados pela seed do mundo, como decidido no GDD 01; ela não reescreve a
ordem de comandos humanos já aceita.

## 4. Matriz resumida

Legenda: **X** = pode disputar o mesmo recurso, estado ou exclusividade;
**A** = interação especial com ataque; **—** = sem conflito intrínseco, salvo
referência explícita à mesma capacidade do catálogo. A metade espelhada é igual.

| Classe \ Classe | Movimento | Obra | Reserva | Comércio/contrato | Diplomacia | Política | Pesquisa | Ataque |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Movimento | X | — | X | — | — | — | — | A |
| Obra | — | X | X | X | — | X | — | — |
| Reserva | X | X | X | X | X | X | X | A |
| Comércio/contrato | — | X | X | X | X | — | X | — |
| Diplomacia | — | — | X | X | X | X | — | A |
| Política | — | X | X | — | X | X | X | — |
| Pesquisa | — | — | X | X | — | X | X | — |
| Ataque | A | — | A | — | A | — | — | A |

O traço não autoriza duas ações incompatíveis por uma regra ainda não catalogada:
o validador deve rejeitar a referência ausente ou a pré-condição falsa. A matriz
não cria custos, capacidades ou ações além das já definidas por catálogo.

## 5. Resolução por classe

| Classe | Conflita quando | Resolução proposta e verificável |
|---|---|---|
| Movimento | A mesma unidade recebe mais de uma ordem; dois movimentos reclamam ocupação/capacidade exclusiva do mesmo hex; ou a unidade foi reservada para ataque. | A primeira ordem aceita reserva a ação da unidade. Conflito de destino é resolvido na fase 4 por ID estável do hex e da unidade; se houver empate de regra, usa seed. Uma unidade com `reserved_attack` não recebe movimento. |
| Obra | Duas obras usam a mesma fila, vaga, trabalhador/tile ou insumo reservado; uma obra e contrato disputam o mesmo bem. | A primeira reserva válida da vaga/insumo prevalece. O segundo pedido é rejeitado sem custo; produção posterior só progride a obra ainda válida. Quais filas e vagas são exclusivas vêm do catálogo. |
| Gasto de reserva | Dois comandos consomem o mesmo saldo de riqueza, bem, ponto de ação, capacidade de rota ou outro recurso reservável. | Reserva atômica na entrada, em `accepted_sequence`. Saldo disponível exclui toda reserva anterior. Falta de saldo/capacidade devolve `insufficient_reserved_capacity` (nome de código **proposto**). |
| Comércio/contrato | Propostas/aceites usam estoque, riqueza, rota, prazo ou `Contract` já comprometido; ou termos mudam depois de contraproposta. | O contrato/proposta tem ID e revisão. Só a revisão vigente pode ser aceita; a primeira aceitação válida fixa obrigações. Estoque e rota disputados seguem a regra de reserva. Entrega e inadimplência são resolvidas no sustento pelo dono Economia. |
| Diplomacia | Duas transições disputam o mesmo par de civilizações, tratado ou obrigação; diplomacia muda a hostilidade de ataques declarados. | Aplicar em `accepted_sequence` na entrada e registrar `LedgerEntry`/fato causal. Efeito pendente de paz, trégua ou guerra é revalidado antes do combate. Paz/trégua aceita pode tornar ataque inelegível: não há dano/ocupação e o custo de preparação já reservado permanece, como decidido no GDD 01. |
| Política | Duas escolhas alteram o mesmo eixo, política ativa, reforma em curso ou recurso reservado por efeito. | Um eixo tem posição válida e há no máximo uma reforma institucional em curso. A primeira mudança válida que ocupar exclusividade prevalece; a posterior é rejeitada sem custo. Esta política de “primeira ocupa” é **proposta**; as pré-condições e efeitos pertencem ao catálogo. |
| Pesquisa | Dois projetos usam a mesma capacidade/investimento; ativação de prática disputa vaga ou manutenção; pesquisa e contrato disputam recurso catalogado. | Reserva de orçamento/prática em `accepted_sequence`; a segunda seleção incompatível é rejeitada sem custo. Há no máximo três práticas ativas, conforme SDD 15. A semântica de trocar projeto antes do fechamento é **proposta** e deve ser decidida no catálogo. |
| Ataque | A mesma unidade ataca/move mais de uma vez; ataques e paz/trégua divergem; múltiplos lados disputam o mesmo confronto ou ocupação. | `DeclareAttack` verifica alcance, hostilidade, Mandato, ponto e reserva a unidade/custo na entrada. Na fase 4, participantes válidos são congelados por confronto, forças e perdas são calculadas simultaneamente; ocupação só é avaliada depois. Empate de ocupação usa seed e IDs estáveis. |

## 6. Comunicação ao cliente

O recibo de uma aceitação deve espelhar `accepted_sequence`; ele não inventa outra
ordem. Uma rejeição deve ser localizável e acionável, sem expor estado oculto:

```text
CommandReceipt {
  command_id,
  status: accepted | rejected,
  accepted_sequence?,
  reason_code?,
  subject_refs: List<GroundingRef>,
  visible_summary
}
```

`reason_code`, textos e os campos opcionais acima são **proposta de protocolo**.
Para cada rejeição, o cliente mostra: comando afetado, causa legível, recurso/vaga
visível que já estava reservado quando puder ser revelado, confirmação de que não
houve custo e uma alternativa válida conhecida. Para conflito posterior à entrada,
o relatório de turno diferencia “aceito e resolvido sem efeito” de “rejeitado” e
cita comando causal, fase e desempate, sem revelar informação de fog of war.

Ataque invalidado por paz/trégua deve dizer claramente: “preparação consumida;
sem dano porque a hostilidade mudou antes do confronto”. A tela de previsão pode
marcar capacidades já reservadas no estado conhecido, mas nunca promete resultado
de combate nem revela ordens alheias ocultas.

## 7. Exemplos trabalhados

### Exemplo A — obra e comércio disputam metal

No turno 18, uma civilização tem 10 unidades de metal disponível. O comando
`StartConstruction` da muralha reserva 8 e é aceito como `accepted_sequence = 41`.
Depois, `AcceptContract` tenta reservar 5 do mesmo metal e é rejeitado por saldo
disponível insuficiente. A muralha continua para a fase de produção; o contrato
não existe, não há transferência, `LedgerEntry` de obrigação nem custo. O cliente
mostra que o metal já estava comprometido pela obra, pois essa informação pertence
ao próprio ator.

### Exemplo B — paz após dois ataques declarados

No turno 24, duas unidades de lados hostis declaram ataque para o mesmo hex. Os
dois `DeclareAttack` passam na entrada, reservam suas unidades e custos, e entram
no log em sequências 88 e 89. Um `AcceptContract` de trégua posterior é aceito na
sequência 90. Na fase de conflitos, a trégua altera a hostilidade antes de congelar
o confronto: os dois ataques tornam-se inelegíveis, não causam dano nem ocupação,
mas mantêm o custo de preparação. A ordem 88/89 não dá dano antecipado a ninguém.

### Exemplo C — empate de ocupação verificável

Após perdas simultâneas, duas coalizões continuam elegíveis para ocupar o mesmo
hex e a regra de catálogo as pontua igualmente. O resolvedor ordena o hex e os
IDs de coalizão de modo canônico, deriva a amostra do PRNG versionado da seed do
mundo, turno, fase e esses IDs, e grava no `DomainEvent` a versão da regra e a
chave de desempate. O replay com os mesmos `AcceptedCommand`, `RulesetRef` e seed
produz o mesmo ocupante e `StateHash`.

## 8. Testes de aceitação propostos

- propriedades: pedido rejeitado não muda recursos, reservas, `AcceptedCommand`
  nem `StateHash`; pedido duplicado por `command_id` é idempotente;
- golden replays: destino disputado, duas obras na mesma vaga, saldo esgotado,
  revisão de contrato obsoleta, trégua após ataque e empate de ocupação;
- determinismo: permutar a ordem de inserção de coleções não altera a resolução;
  repetir seed, comandos e `RulesetRef` reproduz eventos e hash;
- projeção: todo `CommandReceipt` aceito contém a sequência canônica, e toda
  rejeição apresenta motivo sem revelar entidade ou ordem não visível.

## Perguntas abertas para o usuário

1. A política proposta de primeira reserva/primeira escolha deve valer para
   política e pesquisa, ou o jogador pode substituir sua própria escolha antes
   do fechamento?
2. Quais capacidades de movimento e ocupação são exclusivas no catálogo inicial?
3. Qual algoritmo versionado de PRNG e qual formato de chave de desempate serão
   adotados pelo núcleo?
