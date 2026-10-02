# 08 — Entropia e eventos

## Decidido

- A Entropia governa eventos procedurais: catástrofes climáticas, tecnologias surgindo, incidentes
  diplomáticos, revoltas etc.
- Efeitos mecânicos sempre vêm de templates validados; a Entropia escolhe e parametriza dentro de limites.
- **Personalidade, sem nível de dificuldade**: cada mundo tem uma personalidade da Entropia (cíclica,
  contenciosa, transformadora); não existe configuração de dificuldade separada. Decidido em 2026-10-01.
- **Personalidade sugere o clima**: cíclica → sazonal; contenciosa → estável (o drama vem da política);
  transformadora → instável. O jogador pode trocar o clima na criação (02-mapa-e-tiles.md). Decidido em 2026-10-01.
- **A Entropia é oculta**: o jogador vê a causa e o risco de cada evento, nunca o orçamento nem a
  lógica de seleção. As nações **podem tentar convencer, manipular ou prever a Entropia** (rituais,
  oráculos, magia, ciência proibida), mas o custo tende a ser catastrófico frente à recompensa — como
  o outro lado de *Call of Cthulhu*. Analisar riscos e responder aos eventos deve ser quase sempre a
  escolha preferível. Decidido em 2026-10-01.
- **Mudanças permanentes de relevo** e **fenômenos mágicos** são templates da Entropia; ela julga se
  seriam exagerados pelo orçamento de tensão e pelas regras de justiça (00, 02, 06). Decidido em 2026-10-01.

## Proposta

### Papel e leitura

- A Entropia é a diretora de ritmo do mundo: revela uma vulnerabilidade real — abastecimento,
  coesão, exposição ambiental ou compromisso diplomático — e abre uma decisão com custo. Ela não
  cria falhas sem causa legível nem escolhe vencedores.
- A pergunta recorrente é: **o que proteger agora e que risco aceitar depois?** Uma seca pode pedir
  alimento, uma descoberta investimento e um incidente honra ou paz. A escolha desloca pressão entre
  pilares, em vez de apenas somar bônus.
- Cada cartão mostra causa, alcance, em quantos turnos vence e até três respostas; um toque abre fatos que o habilitaram
  e dois toques bastam para decidir. Eventos sem prazo entram na Crônica, não na fila de alertas.

### Orçamento de tensão e curva por era

- No começo da era, o motor cria orçamento mundial `B` e reserva `b[c]` por civilização `c`. `B`
  limita quantidade e severidade; `b[c]` impede que o mundo pressione somente quem já está fraco.
- Valor inicial, sujeito a balanceamento: `B = 12 + 2 × civilizações vivas`, em pontos de tensão por
  era; `b[c] = limitar(2, 6, 2 + piso(P[c] / 25))`. `P` é a pressão de crise de
  [00-visao.md](00-visao.md). O motor só gasta ambos em templates elegíveis.
- Cada template custa 1 (sinal), 2–3 (crise localizada) ou 4 (crise de mundo). Custo mede tensão
  dramática, não dano: um evento caro também pode ser uma oportunidade tecnológica.
- A curva distribui gasto em **leitura** (25%), **crise** (55%) e **respiro** (20%). Valor inicial,
  sujeito a balanceamento: numa era de 8 turnos, leitura ocupa 1–2, crise 3–6 e respiro 7–8. Respiro
  só privilegia recuperação, consequência positiva ou aviso longo; nunca inicia catástrofe grave.
- Sem template justo e elegível, pontos ficam reservados para a próxima janela ou expiram. Assim, a
  prosperidade pode ser uma conquista e o diretor não fabrica desgraça para cumprir uma cota.
- O jogador escolhe investir em resiliência, reduzindo elegibilidade e efeitos, ou explorar uma
  vulnerabilidade para crescer agora. Estoque reduz seca, mas compete com expansão e pesquisa; ver
  [03-economia.md](03-economia.md) e [04-cidades-e-populacao.md](04-cidades-e-populacao.md).

### Personalidade do mundo

- Personalidade só pondera templates já válidos; não altera limites, regras de justiça ou dificuldade.
  A identidade narrativa, portanto, não é uma penalidade escondida.

| Personalidade | Preferência | Leitura estratégica |
|---|---|---|
| Cíclica | clima, recuperação, cadeias sazonais | preparar padrões recorrentes compensa |
| Contenciosa | incidentes, promessas, fronteiras | relações mal cuidadas voltam a cobrar preço |
| Transformadora | descoberta, reforma, adaptação | risco pode abrir mudança institucional |

- Valor inicial, sujeito a balanceamento: a categoria preferida recebe peso 2, as demais peso 1.
  Custo, proteção e sorteio com seed continuam iguais. A personalidade é anunciada ao criar o mundo
  e entra na Crônica de cada era.

### Elegibilidade, justiça e cadeias

- Após produção e crescimento, na fase de resolução de [01-loop-e-turnos.md](01-loop-e-turnos.md),
  o motor avalia gatilhos. Eles exigem fatos concretos: ids de tile, cidade, grupo, tecnologia,
  estado diplomático ou entrada do Ledger; não existe alvo abstrato.
- Valor inicial, sujeito a balanceamento: uma crise de custo 2+ concede proteção de 3 turnos contra a
  mesma categoria e de 2 contra outra crise de custo 3+. Sinais e consequências já anunciadas podem
  ocorrer; o sistema bloqueia repetição de catástrofe, não causalidade.
- Evento algum pode reduzir coesão a zero, remover a última cidade controlada ou causar colapso por
  si. Pode agravar causas que levem à transição jogável pelos limiares de [00-visao.md](00-visao.md)
  e [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md).
- Antes do sorteio, o motor exige resposta útil: uma escolha reduz risco por custo viável, ou uma
  alternativa diplomática está aberta. Se não houver, o template é inelegível. Perdas são possíveis,
  mas precisam ser previsíveis, respondíveis e não repetir o mesmo golpe.
- Uma cadeia tem nó inicial, prazo, estado pendente e no máximo duas bifurcações. Valor inicial,
  sujeito a balanceamento: resolve em 2–4 turnos e custa até 5 pontos somados. O jogador decide no
  nó inicial; os nós seguintes dependem da escolha e do estado, nunca de azar opaco.
- Exemplo: seca → racionamento reduz alimento e eleva tensão de grupos → se persistir, o motor abre
  pedido de ajuda ou disputa local. Economia, sociedade e diplomacia formam uma história única.

### Contrato de template

```text
id e versão do catálogo
categoria; custo de tensão; peso por personalidade
gatilhos determinísticos; alvos permitidos e regra de seleção
parâmetros: tipo, mínimo, máximo e fonte
efeitos: operações do motor, magnitude limitada e duração
escolhas: 2–3 comandos, custos, pré-requisitos e prazo
consequências: nós condicionais, duração máxima e sucessores permitidos
proteções: cooldown, exclusão de colapso e condição de resposta útil
texto: chaves narrativas e fatos obrigatórios a citar
```

- O catálogo define intervalos e operações; IA nunca escreve efeito livre. Por exemplo,
  `perda_alimento` aceita apenas inteiro de 1–3 por dois turnos, não “a colheita foi destruída”. O
  validador rejeita parâmetro fora de faixa, alvo inexistente ou escolha sem recurso para seu custo.
- Alvos são ordenados por id e sorteados por PRNG com seed versionada. A Entropia pode propor
  `{template_id, alvo_id, parâmetros, fatos_citados}`; o motor revalida gatilhos, orçamento,
  proteção e limites antes de gravar o comando.
- Texto pode nomear enchente, emissário ou doença, mas cita os fatos do template. Se a narrativa
  falhar, texto canônico do catálogo ocupa o cartão sem alterar a resolução.

### Eventos completos de referência

#### 1. Clima — Estiagem do vale

- **Gatilhos e parâmetros:** cidade com 2+ tiles agrícolas expostos e privação `D >= 8`; custo 2,
  duração 2, perda de alimento 1–2 por turno. Valores iniciais, sujeitos a balanceamento. Cooldown
  climático impede novo alvo protegido.
- **Escolhas:** racionar (perda −1, tensão de grupos `+3` por turno); importar (gasta reserva ou abre
  pedido comercial); preservar colheita (mantém alimento, perde uma ação produtiva). O jogador pesa
  sobrevivência, coesão e capacidade futura.
- **Consequência:** `D >= 12` abre ajuda diplomática por 2 turnos; alimento recuperado gera respiro
  “safra compartilhada”. Ver [03-economia.md](03-economia.md), [04-cidades-e-populacao.md](04-cidades-e-populacao.md)
  e [07-diplomacia.md](07-diplomacia.md).

#### 2. Tecnologia — Método de conservação contestado

- **Gatilhos e parâmetros:** privação recente, cidade com produção estável e tecnologia adjacente
  disponível; custo 2, investimento 2–4, prazo 2 turnos. Valores iniciais, sujeitos a balanceamento.
  Só seleciona tecnologia explicitamente elegível no catálogo; não concede pesquisa livre.
- **Escolhas:** financiar oficina (paga e abre eficiência limitada de estoque); publicar método
  (benefício menor, negociável em tratado); arquivar (sem custo, sem avanço). Recuperação vira
  vantagem privada, vínculo externo ou prudência.
- **Consequência:** oficina reduz em 1 a perda do primeiro evento climático da era; publicação aceita
  cria promessa no Ledger. Ver [05-tecnologia.md](05-tecnologia.md) e [07-diplomacia.md](07-diplomacia.md).

#### 3. Diplomacia — Caravana retida na fronteira

- **Gatilhos e parâmetros:** rota comercial entre dois povos e tensão ou ofensa recente no Ledger;
  custo 2, carga 1–3, prazo 1 turno. Valores iniciais, sujeitos a balanceamento. Sem entrada
  rastreável de tratado, fronteira ou histórico, o incidente não é elegível.
- **Escolhas:** liberar com reparação (paga e melhora confiança); exigir investigação conjunta
  (atrasa carga e cria compromisso); confiscar (recebe carga e registra ofensa). Lucro imediato pode
  significar isolamento ou escalada.
- **Consequência:** investigação cumprida reduz ofensa; promessa quebrada aumenta dívida e habilita
  sanção ou mediação, não guerra automática. Motor calcula aceitação e Ledger; bots propõem somente
  texto e intenção fundamentada. Ver [07-diplomacia.md](07-diplomacia.md).

#### 4. Revolta — Conselho local recusa o tributo

- **Gatilhos e parâmetros:** tensão alta e coesão baixa na cidade, ou imposto mantido após aviso;
  custo 3, duração 2, grupo afetado com apoio 20–40. Valores iniciais, sujeitos a balanceamento.
  Inelegível por 3 turnos após revolta grave na mesma civilização.
- **Escolhas:** negociar autonomia (tensão −, rendimento central temporariamente −); aliviar tributo
  (gasta reserva, preserva controle); impor guarda (mantém rendimento, perde coesão e pode bloquear
  comércio). O jogador define quem absorve o custo da unidade social.
- **Consequência:** ignorada com coesão crítica, vira transição política local por regras sociais;
  jamais elimina por sorteio. Autonomia cumprida vira regra institucional futura. Ver
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md) e [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md).

#### 5. Epidemia — Febre das rotas

- **Gatilhos e parâmetros:** cidade densa, rota ativa e estoque sanitário insuficiente; custo 3,
  duração 3, produtividade −1–2 e crescimento suspenso por 1 turno. Valores iniciais, sujeitos a
  balanceamento. Escolhe uma rota, não várias cidades simultaneamente.
- **Escolhas:** quarentena (reduz propagação, suspende comércio); cuidar localmente (gasta recurso,
  reduz perda); pedir socorro (abre ajuda com dívida explícita). Economia, população e confiança
  externa entram na mesma decisão.
- **Consequência:** rota mantida sem cuidado pode levar o template uma vez ao parceiro; ajuda entregue
  aumenta confiança e cria crédito. A cadeia termina em recuperação ou tensão social, nunca em
  eliminação direta. Ver [04-cidades-e-populacao.md](04-cidades-e-populacao.md),
  [03-economia.md](03-economia.md) e [07-diplomacia.md](07-diplomacia.md).

### Governador, bots e IA

- **Determinístico no motor:** `P`, orçamento, curva, elegibilidade, alvo, sorteio com seed,
  cooldown, escolhas válidas, efeitos, cadeia, Ledger, colapso e comando gravado. Replays aplicam
  comandos sem IA, conforme ADR-0006.
- **Proposta da Entropia:** escolha entre templates elegíveis, parâmetros em faixa e texto com fatos
  citados. Intenção inválida, atrasada ou sem grounding é descartada; fallback T0 escolhe peso fixo e
  ordem determinística.
- **Governador:** ao delegar, recebe o cartão e propõe resposta compatível com Mandato. Proibições de
  dívida, repressão ou gasto são bloqueadas pelo motor; então o fallback escolhe a melhor opção legal
  e o relatório explica risco, custo e escolha. Ver [09-governador-e-mandato.md](09-governador-e-mandato.md).
- **Bots diplomáticos:** propõem ajuda, reparação ou sanção citando ids do Ledger e fatos do evento.
  Não alteram confiança, aceitação ou dívida: motor calcula essas consequências; ver
  [07-diplomacia.md](07-diplomacia.md).

### Jogo infinito: escala, colapso e renascimento

- Em eras iniciais, a curva prioriza sinais e crises locais; em eras maduras, compromissos entre
  civilizações e manutenção de redes. Valor inicial, sujeito a balanceamento: custo máximo permanece
  4 em toda era; cresce complexidade da cadeia, não números brutos de dano.
- Crescimento habilita mais rotas, fronteiras e manutenção, portanto mais tipos de vulnerabilidade.
  Reservas, eficiência e proteção têm teto administrativo; excedentes viram manutenção, legado ou se
  dissipam ao final da era, conforme [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md). Não
  existe imunidade ou recurso acumulado sem limite.
- Colapso resulta de pressão persistente e decisões acumuladas, nunca de template isolado. Durante a
  transição, Entropia troca catástrofe por sucessão: reconstruir cidade, negociar fronteira ou guardar
  instituição. Isso preserva agência quando a escala diminui.
- No renascimento, ruínas, Crônica, relações e poucos legados persistem; estoque, proteção e cadeias
  pendentes não se multiplicam. Valor inicial, sujeito a balanceamento: no máximo 2 legados ativos;
  excedente converte em memória narrativa. Novo orçamento começa por leitura, sem punição em cascata
  nem crescimento grátis.

## Perguntas abertas

- Mecânica de "interferir na Entropia": quais ações existem, como o custo catastrófico é calculado
  (sorteio com seed, escala com a ambição do pedido) e o que um sucesso raro concede, sem quebrar as
  regras de justiça?
- Catálogo inicial de fenômenos mágicos: quantos, quais gatilhos, como uma nação adere, proíbe ou
  regulamenta (com 05 e 06).
