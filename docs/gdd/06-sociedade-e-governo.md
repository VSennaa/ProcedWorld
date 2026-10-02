# 06 — Sociedade e governo

## Decidido

- Revoltas são parte dos eventos possíveis.
- **Grupos híbridos** (ver 04-cidades-e-populacao.md): os três grupos por função de cada cidade são
  os mesmos grupos políticos deste pilar; demandas (autonomia, participação, tradição, reforma etc.)
  são atributos deles. As regras de grupos abaixo devem ser lidas sobre esse modelo único e
  ajustadas na revisão deste pilar. Decidido em 2026-10-01.
- **Magia como fenômeno + reação política**: a magia surge por template da Entropia (local, grupo ou
  traço de uma sociedade); cada nação reage por leis e políticas — aderir, proibir, regulamentar —
  com consequências mecânicas. Decidido em 2026-10-01.
- **Pressão de crise em duas escalas**: a fórmula de 00-visao.md é aplicada **por cidade** (crises
  locais, revoltas) e **por civilização** (média ponderada por população; crises nacionais e
  colapso). Decidido em 2026-10-01.
- **Governo por 3 eixos discretos** (centralização, participação, obrigação econômica), 3 posições
  cada; reformas mudam um eixo por vez. Decidido em 2026-10-01.
- **Revolução e Mandato, conforme a intensidade**: revolução moderada põe em vigor um Mandato proposto
  pelo novo regime, que o jogador pode reverter; revolução severa impõe o Mandato do novo regime
  (mudar depois exige reforma). Os níveis são definidos por regra do motor. Decidido em 2026-10-01.

## Proposta

### Papel no loop

- Sociedade não é um placar de felicidade. Ela responde à pergunta recorrente da civilização:
  **quais compromissos ainda são aceitos quando recursos, segurança ou autonomia ficam escassos?**
  Governo transforma essa resposta em políticas; cultura e crenças definem quais custos cada grupo
  aceita; crise testa se esse pacto ainda sustenta as cidades.
- O jogador não administra cidadãos individualmente. Em cada sessão, lê uma mudança e escolhe entre
  preservar capacidade estatal, atender um grupo ou fazer uma concessão duradoura. A decisão é
  interessante porque resolve uma pressão agora, mas altera quem terá confiança para cooperar depois.
- O estado social deve ser comum à civilização e ter expressão por cidade. Isso deixa a interface
  legível e impede que dezenas de barras locais escondam a causa de uma revolta.

### Pacto social: legitimidade, coesão e grupos

- Cada civilização tem **Legitimidade** `L` e **Coesão** `C`, ambas inteiras de 0 a 100.
  Legitimidade é a aceitação do direito do governo de decidir; coesão é a disposição dos grupos de
  cumprir compromissos entre si. Não são recursos para maximizar: valores altos reduzem atrito, mas
  reformas e expansão podem consumir ambos por bons motivos.
- Cada cidade tem uma **estabilidade local** `S` de 0 a 100. Ela parte do pacto nacional, mas responde
  a abastecimento, segurança, distância administrativa e presença de grupos locais. Assim, uma capital
  estável não mascara uma fronteira abandonada (ver [04-cidades-e-populacao.md](04-cidades-e-populacao.md)).
- Os grupos sociais são os **três grupos por função** de 04-cidades-e-populacao.md (cultivadores,
  ofícios, mercadores), presentes em cada cidade e somados na civilização. O peso `w` de um grupo é
  sua fração da população. Cada grupo carrega até duas **demandas** ativas: `segurança`,
  `abastecimento`, `autonomia`, `participação`, `tradição` ou `reforma`, que mudam com cultura,
  crenças, magia e crises. Os grupos são abstrações de interesses organizados, não povos reais.
- A cada resolução, o motor calcula a satisfação de demanda de cada grupo `A_g` entre 0 e 100 a partir
  de fatos do estado. Exemplo: abastecimento vem da cobertura de necessidades da cidade; autonomia vem
  de política vigente, distância e controle local. A tensão do grupo é `T_g = 100 - A_g`.
- **Valores iniciais, sujeitos a balanceamento:**
  `C' = limitar(0,100,C + 4*compromissos_cumpridos - 5*compromissos_quebrados - média_ponderada(T_g)/10 - 2*conflitos_ativos)`.
  `L' = limitar(0,100,L + 3*serviços_cumpridos + 2*vitórias_defensivas - 4*políticas_violadas - média_ponderada(T_g)/12)`.
  Cada termo é limitado por turno para que uma única ação não apague uma civilização.
- O jogador escolhe quais demandas entram no pacto pela política e pela resposta a crises. Favorecer
  uma fronteira pode baixar a tensão dela, mas, se consumir reservas destinadas ao centro, abre uma
  disputa de legitimidade. A leitura mostra os três grupos que mais alteraram `C`, nunca uma soma opaca.

### Governo e políticas: poucas escolhas, compromissos duráveis

- Um governo é composto por três **eixos de instituição**: centralização, participação e obrigação
  econômica. Cada eixo tem três posições discretas. A combinação é uma identidade legível, sem exigir
  que o jogador decore uma lista enorme de regimes: por exemplo, centralizado, consultivo e tributário.
- Cada posição libera um conjunto pequeno de políticas de dados. A civilização mantém uma política
  ativa por eixo e pode iniciar apenas uma mudança institucional por vez. Isso impede acumular todos os
  bônus e faz do tempo de transição um risco calculável.
- Uma política declara em seu template: demandas favorecidas e prejudicadas, custo de manutenção,
  modificador de estabilidade local e pré-requisitos tecnológicos. Efeitos iniciais de política ficam
  entre `−10` e `+10` pontos de satisfação de demanda e entre `−5` e `+5` de estabilidade por cidade,
  sujeitos a balanceamento; não há multiplicadores permanentes de produção.
- **Reforma** é trocar uma posição de eixo ou política. Ela dura 2 turnos como valor inicial, sujeito
  a balanceamento. No primeiro turno custa `−8 L` e `−4 C`; ao concluir, aplica somente os efeitos do
  template novo. Grupos favorecidos reduzem a tensão; grupos prejudicados podem organizar oposição.
- O jogador decide se reforma cedo, pagando instabilidade enquanto ainda há reserva, ou sustenta uma
  instituição conhecida até ter capacidade de compensar seus perdedores. A segunda opção é segura no
  curto prazo, mas pode tornar um cisma inevitável.
- Uma cidade tem `S < 40` quando seu modificador local passa a reduzir rendimentos não essenciais em
  até 20%; em `S < 20`, ela pode entrar em agitação. São valores iniciais, sujeitos a balanceamento.
  A regra conecta administração a produção sem transformar descontentamento em mera punição econômica.

### Cultura, crenças e cismas

- **Cultura** é um conjunto de até três traços ativos da civilização: por exemplo, hospitalidade,
  disciplina cívica ou autonomia comunal. Um traço descreve uma preferência social e uma regra pequena,
  como tolerar melhor racionamento ou valorizar participação; ele não concede poder militar bruto.
- **Crença** é uma instituição cultural organizada em torno de uma prioridade social. Ela pode cruzar
  cidades e fronteiras, oferecendo apoio em crise em troca de influência sobre uma demanda. Cultura e
  crenças mudam por adoção, reforma ou cisma; não são selecionadas livremente como bônus de abertura.
- A cada marco de era, o motor oferece no máximo duas **evoluções culturais** elegíveis, derivadas de
  ações repetidas e eventos registrados na Crônica. Adotar uma troca um traço: concede `+8` de
  satisfação para uma demanda e `−4` para uma demanda conflitante, valores iniciais sujeitos a
  balanceamento. O jogador escolhe consolidar uma história que já viveu, em vez de caçar o melhor bônus.
- Um **cisma** fica elegível quando um grupo com crença ou traço incompatível mantém `T_g >= 65` por
  3 turnos e possui peso `w >= 25`. Esses limiares são valores iniciais, sujeitos a balanceamento.
  O motor oferece a escolha de conciliação, tolerância local ou repressão legal, conforme templates
  disponíveis; não sorteia uma troca de regime sem aviso.
- Conciliação reduz tensão e pode enfraquecer centralização; tolerância preserva o pacto nacional, mas
  cria uma diferença local persistente; repressão melhora controle imediato e reduz `L` e `C`. A escolha
  faz uma crença importar no mapa, na política e na história, sem exigir uma árvore teológica extensa.

### Pressão, revolta e revolução

- A **pressão de crise** usa a fórmula única de [00-visao.md](00-visao.md) em duas escalas: `P_cidade`
  com os termos locais e `P_civ` como média ponderada por população. Termos: `D` privação (0–20), `G`
  tensão dos grupos (0–20), `W` ameaça de guerra (0–20), `E` exposição ambiental (0–20) e coesão.
  O rascunho deste pilar propunha somar `(100−S)/2` (estabilidade local) em `P_cidade`; incluir ou
  não esse termo fica para a consolidação, para não contar a estabilidade duas vezes.
  São escalas iniciais, sujeitas a balanceamento. Privação e exposição vêm de regras de economia, mapa
  e cidade, não de narrativa (ver [02-mapa-e-tiles.md](02-mapa-e-tiles.md),
  [03-economia.md](03-economia.md) e [04-cidades-e-populacao.md](04-cidades-e-populacao.md)).
- `P` de 40–69 gera um aviso e uma decisão de mitigação; `P >= 70` por 2 turnos torna elegível uma
  crise local; `P >= 85` com `L < 30` torna elegível uma revolta. Valores iniciais, sujeitos a
  balanceamento. O resumo sempre aponta os dois maiores termos da fórmula e a cidade afetada.
- Revolta é um template de evento que cria disputa de controle, bloqueia uma política ou exige uma
  concessão; seu efeito exato é validado pelo motor. **Revolução** é a escalada social: exige pelo
  menos uma revolta ativa, `C < 20` por 2 turnos e um grupo elegível a liderar. Ela pode forçar
  reforma, secessão ou substituição institucional; nunca é um resultado narrativo sem cadeia causal.
- O jogador decide entre atender a causa, redistribuir capacidade de cidades estáveis ou sustentar a
  ordem atual com custo político. A melhor escolha depende do Mandato, das reservas e de quais cidades
  ainda podem suportar a perda, por isso não existe resposta universal.
- A Entropia pode propor um template de revolta, cisma ou reforma somente quando as condições de
  elegibilidade já forem verdadeiras e houver orçamento de tensão. Ela escolhe a apresentação e
  parâmetros dentro das faixas do template; o motor calcula `P`, valida o alvo e aplica o efeito (ver
  [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).

### Cidades, diplomacia e Mandato

- Cidades estáveis contribuem serviços e reservas para `L`; cidades em agitação consomem capacidade
  administrativa e podem interromper rotas. Expandir para um hexágono distante aumenta opções de
  recursos, mas também a demanda por integração e defesa: crescimento territorial é uma decisão social,
  não só econômica (ver [02-mapa-e-tiles.md](02-mapa-e-tiles.md)).
- Ajuda externa, comércio e tratados podem satisfazer necessidades ou proteger minorias, elevando a
  estabilidade. Em contrapartida, dívida, promessa quebrada ou apoio a um cisma alteram a confiança no
  Ledger de Relações. Bots devem citar esses fatos ao propor ajuda, sanção ou apoio diplomático; o
  motor decide aceitação e efeitos (ver [07-diplomacia.md](07-diplomacia.md)).
- O **Mandato** inclui postura de ordem social: prioridade entre estabilidade, reforma e autonomia;
  limite de legitimidade que o Governador pode arriscar; ações proibidas, como reprimir protestos; e
  gatilhos de aviso, como `P >= 70` ou revolução elegível. É uma preferência e uma fronteira, não uma
  garantia de evitar toda crise (ver [09-governador-e-mandato.md](09-governador-e-mandato.md)).
- O Governador pode propor uma intenção tipada de política, reforma, resposta a cisma ou negociação.
  O motor rejeita a intenção que viole Mandato, pré-requisito, limite de gasto ou estado da cidade. Se
  a delegação estiver ativa, o jogador escolhe antecipadamente quanta legitimidade o Governador pode
  gastar antes de notificar; em celular, essa decisão evita alertas para ajustes rotineiros.

### Determinismo e papel da IA

- **Determinístico:** grupos presentes, pesos, satisfação, `L`, `C`, `S`, `P`, elegibilidade de cisma,
  reforma, revolta e revolução; custos, efeitos e duração de políticas; aceitação diplomática; ordem de
  resolução e transições de controle. Tudo deriva de estado, comandos, seed e templates versionados.
- **Proposto pela IA:** Governador, Entropia e bots escolhem uma intenção permitida e fornecem uma
  justificativa ancorada em ids e fatos — por exemplo, cidade com `P=74`, compromisso quebrado ou
  entrada do Ledger. O LLM pode redigir a Crônica e a mensagem diplomática, mas não escolhe valores
  mecânicos fora do schema nem altera estado diretamente.
- Intenções inválidas, sem grounding, fora do Mandato ou indisponíveis por timeout são descartadas.
  O fallback T0 escolhe deterministamente uma resposta válida segundo prioridades do Mandato; cada
  comando aceito entra no log. Replays reproduzem efeitos sem chamar IA, conforme ADR-0006.

### Eras, colapso e renascimento

- Instituições acumulam **complexidade administrativa**: cada cidade além da capacidade efetiva, rota
  longa e política excepcional soma pressão de manutenção. A capacidade cresce por marcos e instituições,
  mas é limitada: exceder o limite aumenta `P` e reduz serviços, em vez de aumentar rendimentos sem fim.
  Os coeficientes ficam em dados e são valores iniciais, sujeitos a balanceamento (ver
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- No marco de era, o jogador escolhe uma consolidação: ampliar capacidade administrativa, preservar um
  traço cultural ou quitar obrigações sociais. Escolher uma deixa as outras mais vulneráveis; excedentes
  viram reserva limitada, manutenção ou legado, nunca multiplicadores acumulativos.
- Se `C = 0` por 2 turnos ou uma revolução resolve a perda de controle, ocorre colapso parcial. O
  motor reorganiza território por controle local, distância e regras de sucessão; o jogador escolhe
  uma comunidade sucessora. Não há eliminação por um único evento da Entropia.
- O renascimento conserva Crônica, ruínas, até um traço cultural e legados explicitamente definidos,
  mas começa com apenas 40–60% da capacidade administrativa anterior, valor inicial sujeito a
  balanceamento. A escala menor recupera legibilidade; reivindicações, cismas e dívidas persistentes
  tornam a nova era diferente da anterior, sem acumular expansão grátis.

### Sessão curta e leitura causal

- A tela social mostra um cartão único: `o que piorou`, `por quê` (termos da fórmula e fatos) e até
  três respostas válidas. Tocar numa resposta revela quem ganha, quem perde e sua duração; confirmar
  exige no máximo dois toques depois de abrir o cartão.
- Crises abaixo do limiar ficam no relatório do Governador. Notificações exigem atenção apenas para
  reforma concluindo, `P >= 70`, revolta elegível, ou intenção bloqueada pelo Mandato. Isso protege
  sessões de 2–8 minutos sem tirar do jogador decisões que definem a crônica.

## Perguntas abertas

- Leis podem restringir a migração automática (ver 04-cidades-e-populacao.md)?
- Quais limiares separam revolução moderada de severa?
- Incluir a estabilidade local `S` em `P_cidade` ou mantê-la só como efeito sobre rendimentos?
