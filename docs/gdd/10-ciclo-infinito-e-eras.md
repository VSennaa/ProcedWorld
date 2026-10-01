# 10 — Ciclo infinito e eras

## Decidido
- Não há condição de vitória obrigatória.

## Proposta

### Função do ciclo

- Uma era é um arco de transformação da civilização, não uma contagem de anos nem uma corrida de
  pesquisa. Ela começa quando a sociedade assume uma direção, passa por expansão e tensão, e fecha
  quando essa direção deixa um legado, é reformada ou falha. O jogador sempre pode continuar: sua
  pergunta é o que preservar e a que custo, não se venceu a partida.
- O ciclo proposto é **ascensão → auge → crise → colapso ou renovação → renascimento**. Nem toda era
  precisa terminar em colapso; a crise torna visíveis os custos acumulados e pode ser resolvida por
  reforma, concessão diplomática ou redução deliberada de escala.
- A escala é lida pela **coesão social** e pela **pressão de crise** de `00-visao.md`, nunca por uma
  pontuação final. Isso faz uma civilização rica, mas incapaz de alimentar ou integrar suas cidades,
  parecer poderosa e vulnerável ao mesmo tempo.

### Marcos e fases da era

- No início, o motor abre uma era quando a civilização atinge dois marcos dentre: fundar ou integrar
  uma cidade, estabelecer uma rota ou tratado, concluir uma descoberta, consolidar uma instituição,
  ou sobreviver a uma crise relevante. A lista é um catálogo de marcos versionado; não depende de
  texto da IA. Antes de abrir outra era, o motor exige ao menos quatro turnos desde a anterior.
  Esses valores são iniciais, sujeitos a balanceamento.
- A **ascensão** ocupa os primeiros marcos: novas cidades, instituições ou acordos aumentam opções,
  mas também alcance administrativo, manutenção e obrigações. O jogador escolhe um **compromisso de
  era** entre até três opções geradas a partir do estado, como integrar fronteiras, reservar excedente
  ou aprofundar um tratado. Cada opção nomeia um ganho e uma renúncia comparáveis em uma tela.
- O **auge** começa após um terceiro marco da mesma direção. A civilização recebe a melhor versão
  temporária de sua especialização, mas passa a pagar seu custo: por exemplo, uma rede comercial
  aumenta reservas enquanto eleva dependência de rotas e credores. O jogador decide se converte o
  excedente em reserva, legado ou expansão; a decisão é interessante porque protege riscos distintos
  e não há conversão que maximize os três.
- A **crise** inicia quando a pressão de crise é 70 ou mais por dois turnos, conforme a regra proposta
  em `00-visao.md`, ou quando um template elegível determina uma crise equivalente. Ela apresenta
  até três respostas: absorver o custo com reservas, mudar a política e desagradar um grupo, ou
  ceder território/compromisso. O motor só oferece respostas que sejam executáveis no estado atual.
- A era fecha quando a crise foi resolvida, quando ocorre colapso, ou após doze turnos sem marco novo.
  Quatro, doze e o requisito de marcos são valores iniciais, sujeitos a balanceamento. A salvaguarda
  evita que uma civilização segura fique presa numa era só por evitar mudanças, sem transformar a era
  em relógio fixo.

### Direção sem vitória

- Ao abrir a era, o motor seleciona três **objetivos emergentes** de templates elegíveis, cada um
  ligado a fatos do estado: sanar a privação de uma cidade, estabilizar um grupo, assegurar uma rota,
  preservar uma fronteira ou reparar uma relação. Eles não dão pontos de vitória nem bloqueiam outros
  planos; são lentes para tornar uma necessidade concreta e comparável em sessão curta.
- O jogador fixa no máximo um objetivo como prioridade e pode ignorá-lo. Concluí-lo concede um marco
  e habilita uma escolha de consolidação; abandoná-lo registra uma consequência normal, como dívida,
  tensão de grupo ou perda de confiança, apenas se a regra que o originou assim definir. Logo, uma
  promessa diplomática não vira missão artificial: seu peso continua vindo do Ledger de Relações.
- Exemplo: diante de seca e uma aliança frágil, a prioridade pode ser “garantir alimento por quatro
  turnos”. Importar resolve a privação e cria dívida; racionar preserva autonomia e reduz coesão;
  deslocar população protege o núcleo e abandona a fronteira. O objetivo torna a escolha legível, mas
  os custos vêm de economia, população e diplomacia, não de um bônus narrativo.

### Escala, manutenção e freios de crescimento

- Cada cidade além da primeira, rota além da capacidade administrativa e compromisso diplomático
  ativo adiciona **carga de complexidade**. Valor inicial, sujeito a balanceamento:
  `complexidade = cidades - 1 + rotas_excedentes + tratados_com_obrigação`.
  A cada 3 pontos, a manutenção total aumenta 10% e a exposição administrativa aumenta 1, até o teto
  de 50%. O motor calcula tudo em inteiros ou ponto fixo; valores negativos nunca reduzem a carga.
- A capacidade administrativa inicial é `1 + instituições consolidadas`, com teto de 6; cada
  instituição só aumenta o teto se ocupar uma vaga de legado. Os números são valores iniciais,
  sujeitos a balanceamento. Assim, conhecimento e governo liberam organização, mas não crescimento
  exponencial: ampliar o mapa sem consolidar aumenta a pressão de crise em vez de só rendimentos.
- Recursos acumulados acima de uma reserva máxima de seis turnos de manutenção sofrem conversão no
  fechamento da era: o jogador escolhe investir em reparo, legado ou ajuda contratual a outro povo.
  O excedente não vira multiplicador permanente. Se não escolher, a regra T0 o reserva até o teto e
  converte o restante em manutenção atrasada. Seis turnos é valor inicial, sujeito a balanceamento.
- Tecnologias, edifícios e efeitos de eras posteriores devem abrir alternativas ou substituir custos,
  não elevar indefinidamente produção, combate ou coesão. Esse pilar exige que progressão e perdas
  tecnológicas adotem tetos e substituições no catálogo (ver `05-tecnologia.md` e `03-economia.md`).

### Crise, colapso e renascimento

- **Renovação** é a saída preferível: se a crise cai abaixo de 40 antes de três turnos de crise, a
  civilização fecha a era sem fragmentar. Ela escolhe uma consolidação, mas perde a direção anterior;
  isso impede empilhar bônus de foco. Os limiares e três turnos são valores iniciais, sujeitos a
  balanceamento.
- **Colapso** só ocorre se a coesão for 0 por dois turnos, ou se uma crise grave validada deixar o
  governo incapaz de cumprir funções por três turnos. Não há eliminação por sorteio, nem por um único
  evento da Entropia. A cadeia causal — privação, tensão, guerra, resposta recusada — aparece no
  resumo e na Crônica (ver `04-cidades-e-populacao.md`, `06-sociedade-e-governo.md` e
  `08-entropia-e-eventos.md`).
- Na transição, o motor cria comunidades sucessoras usando controle territorial, distância das cidades,
  população e regras de sucessão do catálogo. O jogador escolhe uma delas para continuar; as demais
  viram civilizações de bot com seus territórios, relações e reivindicações registrados. O dilema é
  escolher o núcleo coeso, a cidade mais rica ou a fronteira que guarda um legado.
- A sucessora conserva 40–60% da população e capacidade administrativa, uma fração das reservas até
  dois turnos de manutenção e no máximo dois legados ativos. Faixas são valores iniciais, sujeitos a
  balanceamento. O restante se torna ruína, patrimônio disputado, conhecimento degradado ou memória
  histórica no mundo. A redução devolve escala administrável e evita que o colapso seja uma rota de
  expansão grátis.
- O **renascimento** abre uma era de reconstrução com coesão inicial 45–60 e pressão de crise limitada
  a 60 no primeiro turno, salvo guerra em curso. São valores iniciais, sujeitos a balanceamento. Não é
  imunidade: concede tempo para decidir entre reconquistar, negociar reconhecimento ou fortalecer o
  núcleo. Reivindicações e dívidas herdadas mantêm os vizinhos envolvidos.

### Legado e Crônica

- Um legado é uma marca persistente e escassa, não uma coleção de bônus. Ao fechar uma era, o jogador
  pode consolidar no máximo uma escolha entre: **instituição** (eleva ou altera capacidade
  administrativa), **patrimônio** (monumento, ruína ou obra que fixa um efeito local) ou **memória
  social** (traço cultural que altera uma escolha de crise). Máximo de dois legados ativos por
  civilização; números iniciais, sujeitos a balanceamento.
- Cada legado possui condição, efeito limitado, origem e dono atual. Se a cidade for perdida, um
  patrimônio pode ser capturado; se a sucessora não escolher uma instituição, ela fica como ruína;
  memória social só sobrevive se a comunidade sucessora contiver população suficiente da anterior.
  O jogador escolhe antes da crise qual parte da história vale sustentar, em vez de receber herança
  automática.
- No fechamento, o motor consolida fatos do período em uma entrada canônica da **Crônica**: marcos,
  causas de crises, comandos relevantes, mudanças de território, legados e fatos diplomáticos. O
  texto narrativo pode resumir esses fatos, mas a fonte é o estado e o log de comandos. Entradas de
  detalhe passam de memória episódica a uma síntese de era; a Crônica de eras antigas mantém apenas
  legados, relações ainda ativas e precedentes de conflito.
- Para limitar contexto e evitar história infinita ilegível, cada civilização mantém as duas últimas
  eras em detalhe e uma ficha consolidada das anteriores; o mundo mantém uma linha do tempo de
  legados, colapsos e tratados ainda relevantes. Dois períodos detalhados é valor inicial, sujeito a
  balanceamento. A compactação segue as salvaguardas de saturação de `CLAUDE.md`; apagar contexto de
  IA nunca apaga fatos do motor.

### Entropia, Governador e diplomacia

- A curva de tensão da Entropia varia por fase: baixa na ascensão para deixar escolhas criarem
  vulnerabilidades, moderada no auge para testar dependências e alta na crise para oferecer pressão
  elegível, não punição arbitrária. Seu orçamento de tensão não acumula entre eras: saldo não gasto
  expira no fechamento. Faixas de orçamento pertencem a templates de dados e são valores iniciais,
  sujeitos a balanceamento (ver `08-entropia-e-eventos.md`).
- A Entropia pode propor uma intenção de evento com template, alvo, parâmetros permitidos e fatos que
  demonstram elegibilidade; por exemplo, seca onde existe exposição ambiental ou incidente onde há
  dívida diplomática. O motor determina elegibilidade, sorteia com seed, debita o orçamento e aplica
  somente os efeitos do template validado. A IA pode escrever o relato, jamais acrescentar dano,
  recursos ou uma transição de era.
- O Governador recebe a fase, a pressão, os objetivos fixados, legados ameaçados e limites do Mandato.
  Quando delegado, propõe uma intenção dentre respostas já válidas: preservar reserva, aceitar ajuda,
  reformar ou reduzir fronteira. Linhas vermelhas e limites de gasto/guerra do Mandato bloqueiam a
  proposta; sem jogador ou IA, T0 escolhe a opção de menor pressão compatível com as prioridades
  declaradas (ver `09-governador-e-mandato.md`).
- Bots diplomáticos usam a fase de era para explicar urgência, não para ignorar relações. Podem propor
  trégua, ajuda, proteção de patrimônio ou reconhecimento de sucessora, citando entradas do Ledger e
  fatos da Crônica. O motor calcula aceitação, obrigações, decaimento e mudança de estado; uma proposta
  rejeitada não altera números só porque sua narrativa foi persuasiva (ver `07-diplomacia.md`).

### Determinismo e experiência móvel

- São determinísticos: contagem de marcos, abertura e fechamento de eras, carga de complexidade,
  elegibilidade de objetivos e eventos, limiares de crise/colapso, sucessão, herança, consolidação
  factual da Crônica e efeitos de tratados. Todos são comandos ou resultados de regras com seed
  explícita, registrados para replay conforme ADR-0006.
- São propostas de IA: qual objetivo elegível destacar, qual legado recomendar, qual resposta válida
  o Governador tenta, qual evento-template a Entropia tenta usar e como bots redigem uma negociação.
  Cada intenção deve referenciar ids de cidades, legados, entradas do Ledger ou fatos da Crônica. O
  validador descarta intenção inválida, fora do Mandato ou sem grounding, e aplica fallback T0.
- A tela de início de era mostra somente: “o que mudou”, “o risco que cresce” e até três compromissos.
  A tela de crise apresenta causa, consequência de não agir e até três respostas. Fechar uma era é
  uma escolha de legado em até dois toques, com detalhes expansíveis. Isso mantém sessões de 2–8
  minutos sem esconder o encadeamento causal (ver `11-experiencia-mobile.md` e `01-loop-e-turnos.md`).
- Notificações são reservadas para início de crise, risco de colapso, prazo de tratado ou escolha de
  sucessora. O relatório comum registra ascensão, auge e consolidação sem exigir retorno imediato.
  Quem se ausenta continua protegido pelo Mandato, mas vê claramente quais decisões T0 tomou e por quê.

### Mundos persistentes e entrada tardia

- Em um mundo realmente infinito, a idade não deve equivaler a vantagem irreversível. Capacidade,
  reservas, legados e contexto têm tetos; perdas, manutenção e diplomacia preservam espaço para novas
  sociedades. Ruínas e tratados tornam o passado útil, mas não concedem ao fundador uma produção sem
  contrapartida.
- Um novo jogador entra como uma comunidade inicial em região com capacidade ecológica e distância
  mínima de capitais, escolhida pelo motor entre opções comparáveis. Recebe uma era de assentamento,
  dois turnos de proteção contra anexação direta e um pacto de não agressão com vizinhos imediatos;
  não recebe recursos de era avançada. Regras e durações são valores iniciais, sujeitos a balanceamento.
- A entrada oferece duas posturas: fundar sem vínculos, com mais autonomia e menos infraestrutura, ou
  aceitar patrocínio de uma civilização existente, com acesso a rota/legado local e obrigação no
  Ledger. A decisão cria diplomacia desde o primeiro toque e permite que mundos velhos sejam cenário,
  não sala fechada.

## Perguntas abertas

- O mundo tem fim (mundos "temporada") ou é realmente eterno?
  - **Recomendação:** iniciar o MVP com mundos persistentes e eternos, pois o ciclo de legado,
    colapso e renascimento precisa provar seu valor antes de uma reinicialização periódica. Avaliar
    temporadas depois como modo separado, com regra explícita para arquivo de Crônica e novos mundos;
    elas atendem recomeços competitivos, mas não devem invalidar a promessa principal.
- Como entram novos jogadores num mundo antigo e avançado?
  - **Recomendação:** usar assentamento protegido por curto período, duas posturas de entrada e
    progressão limitada pelos mesmos tetos de manutenção e legado descritos acima. Isso permite
    participação imediata sem presenteá-lo com tecnologia avançada nem tornar a diplomacia opcional;
    validar os valores de proteção em simulação e testes de entrada tardia.
