# 11 — Experiência mobile

## Decidido
- Cliente para celular; o servidor faz todo o processamento (ADR-0001).
- BYOK: o jogador informa a API key e o resto é automático.

## Proposta

### Princípio da sessão: uma escolha que muda a sociedade

- A unidade de experiência não é administrar cada hexágono, mas entender uma mudança e assumir um
  compromisso. Em uma sessão curta, o jogador deve conseguir responder: “o que mudou, por que mudou
  e o que aceito sacrificar para lidar com isso?”.
- A tela inicial de cada retorno mostra primeiro uma **pauta de turno** com, no máximo, uma decisão
  crítica e duas decisões importantes. O restante é relatado como consequência observável, não como
  uma lista de alertas que exige confirmação.
- Uma decisão crítica é exibida quando altera diretamente a coesão, a pressão de crise, um tratado,
  a continuidade territorial ou uma transição de era. Este critério conecta a interface à promessa
  de preservar uma sociedade sob pressão, em vez de premiar presença constante (ver
  [00-visao.md](00-visao.md)).
- Cada cartão de decisão mostra causa, efeito imediato, risco futuro e prazo. Por exemplo: durante
  uma seca, importar alimento, racionar ou retirar uma colônia de fronteira. O jogador escolhe qual
  capacidade preservar; não procura a opção universalmente “correta”.
- Valor inicial, sujeito a balanceamento: uma sessão normal contém 1–3 escolhas relevantes, dura
  2–10 minutos e permite concluir o turno em até 8 toques a partir da pauta. Abrir detalhes nunca é
  obrigatório para enviar uma ordem válida.

### Estrutura de uma sessão

1. **Retorno e relatório.** Ao abrir o jogo, o relatório compara o estado atual com o último turno
   visto: até três mudanças, cada uma com um fato causal rastreável e um link para o local, cidade,
   tratado ou evento correspondente. Exemplo: “a colheita caiu porque dois hexágonos irrigados foram
   afetados pela seca; a reserva cobre mais um turno”.
2. **Escolha de prioridade.** O cartão crítico apresenta até três respostas legais e mutuamente
   exclusivas, já resumidas em ganho, custo e consequência provável. Um toque escolhe; um segundo
   toque confirma. A interface nunca apresenta uma resposta que o motor rejeitará por recursos,
   alcance ou Mandato.
3. **Inspeção opcional.** Tocar uma causa abre o mapa no hexágono ou a folha da entidade; tocar uma
   consequência abre a explicação da regra. Assim, o jogador curioso investiga a cadeia causal sem
   tornar a leitura superficial obrigatória para todos.
4. **Plano ou delegação.** Se não houver crise, o jogador escolhe uma intenção de foco para o turno,
   confirma o plano do Governador ou ajusta uma linha do Mandato. O botão “pronto” mostra quantas
   ordens serão enviadas e quais áreas continuarão delegadas.
5. **Resolução e fecho.** Depois da resposta do servidor, uma animação curta destaca apenas o que
   mudou no mapa. O relatório final registra “ordem → regra aplicada → consequência”; a Crônica pode
   transformar o mesmo fato em narrativa, sem substituir a explicação mecânica.

- O jogador decide entre agir diretamente, aceitar a sugestão do Governador ou mudar o Mandato que
  guiará as ausências. A escolha é interessante porque delegar reduz atrito agora, mas uma prioridade
  mal configurada pode preservar uma área às custas de outra em uma crise futura (ver
  [09-governador-e-mandato.md](09-governador-e-mandato.md)).
- Turnos simultâneos não exigem permanecer conectado: não há relógio (ADR-0008); a pauta mostra o que
  vence em quantos turnos e permite marcar “pronto”. Se o jogador sair, o Governador joga por ele nos
  turnos seguintes dentro do Mandato; a ordem de resolução
  permanece responsabilidade do servidor (ver [01-loop-e-turnos.md](01-loop-e-turnos.md)).

### Telas e navegação

| Tela | Pergunta que responde | Ação principal | Informação progressiva |
|---|---|---|---|
| Pauta | O que merece minha atenção agora? | Escolher, delegar ou marcar pronto. | Causa, prazo e comparação entre respostas. |
| Mapa | Onde isso acontece e o que está conectado? | Selecionar hexágono, cidade, unidade ou rota. | Camadas de recursos, controle, risco e alcance. |
| Sociedade | O que mantém a civilização coesa? | Escolher uma resposta de crise ou foco de era. | Grupos, abastecimento e trajetória da pressão. |
| Relações | Em quem posso confiar e o que devo? | Aceitar, recusar ou contrapor um tratado. | Entradas do Ledger e previsão de aceitação. |
| Mandato | O que pode acontecer sem mim? | Fixar prioridade, limite e linha vermelha. | Simulação textual de exemplos de delegação. |
| Crônica e custo | O que aconteceu e quanto a IA consumiu? | Ler, revisar limites ou trocar de modo de IA. | Eventos, intenções aceitas e detalhamento de consumo. |

- A barra inferior oferece Pauta, Mapa, Sociedade, Relações e Crônica. O Mandato abre pela Pauta e
  pelo perfil do Governador, para não disputar espaço com a decisão imediata.
- O jogo retorna sempre à Pauta, nunca ao último painel profundo. Isso reduz a carga de retomada após
  uma sessão interrompida, enquanto links no relatório preservam o caminho para investigação.
- O mapa é a única tela que permite emitir uma ordem espacial; as demais preparam escolhas e
  explicam consequências. Isso evita que o jogador precise abrir cinco painéis para cumprir uma
  decisão que já entendeu.

### Mapa hexagonal para toque

- Um toque seleciona um hexágono e mostra um cartão inferior com terreno, produção, controle,
  ameaça e ações legais. Um segundo toque na ação destaca em azul os destinos ou alvos válidos; tocar
  o destino confirma a ordem em uma prévia, e “enviar” só ocorre ao marcar pronto.
- Arrastar move a câmera; pinçar amplia; toque duplo centraliza e alterna entre o zoom regional e o
  zoom local. Pressionar e segurar abre a explicação de um ícone, sem depender de hover.
- No zoom regional, cidades, crises e propostas diplomáticas aparecem como marcadores agrupados; no
  zoom local, os hexágonos recebem bordas de seleção largas. Valor inicial, sujeito a balanceamento:
  área mínima de toque de 9 mm e ampliação automática de alvos adjacentes quando o zoom não os
  separa visualmente.
- O jogador decide se responde ao sintoma na cidade afetada ou à causa territorial, como proteger um
  corredor produtivo em vez de reforçar uma cidade isolada. A leitura de alcance e exposição torna
  expansão, defesa e abastecimento escolhas conectadas ao mapa (ver [02-mapa-e-tiles.md](02-mapa-e-tiles.md)
  e [03-economia.md](03-economia.md)).
- Uma camada por vez evita poluição visual: produção, abastecimento, controle, pressão e diplomacia.
  A camada selecionada mantém uma legenda curta e usa padrão/ícone além de cor; ela não esconde a
  seleção atual nem os prazos críticos.

### Relatório, pressão e acontecimentos

- O relatório de turno é um objeto de jogo derivado do log, não uma narração livre. Para cada mudança
  importante ele contém: fato observado, regra ou template que a produziu, entidades envolvidas e a
  próxima decisão disponível. “Por que?” abre essa cadeia em vez de um texto persuasivo sem prova.
- A pauta usa três estados de prioridade: **acompanhar**, **decidir neste turno** e **crise iminente**.
  Valor inicial, sujeito a balanceamento: apenas o último estado pode interromper o fluxo de abertura
  com um cartão em tela cheia; os outros ficam ordenados na lista.
- A pressão de crise é apresentada como tendência — subindo, estável ou caindo — e por seus fatores,
  não como uma barra que convida a otimização cega. O jogador vê privação, tensão social, ameaça de
  guerra, exposição ambiental e coesão quando esses valores forem definidos pelas regras do mundo.
  Uma queda de coesão deve mostrar as contribuições relevantes e possíveis respostas, pois a pressão
  só é interessante quando é legível e negociável (ver [00-visao.md](00-visao.md)).
- A Entropia pode fazer um acontecimento aparecer na pauta apenas se um template elegível tiver sido
  validado. A interface distingue “sinal” — vulnerabilidade observada — de “evento resolvido” — efeito
  já aplicado — para que o jogador tenha chance de responder sem acreditar que narrativa é causa
  mecânica (ver [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).
- O jogador escolhe quando investigar e qual perda absorver; não escolhe se o relatório esconderá
  uma ameaça real. Em particular, uma revolta ou incidente diplomático deve citar os fatos que a
  tornaram elegível, preservando a coesão social como algo construído por compromissos (ver
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md)).

### Diplomacia e o Governador na interface

- Propostas diplomáticas entram na Pauta com uma frase curta, o prazo e o compromisso pedido. O
  cartão oferece aceitar, recusar ou abrir contraproposta; esta última exibe somente cláusulas que o
  catálogo permite e que a civilização pode oferecer.
- Antes de confirmar, a tela Relações mostra até três fatos do Ledger que justificam a oferta — por
  exemplo, ajuda recebida, promessa quebrada e ameaça comum — e um intervalo qualitativo de
  aceitação. O jogador decide se compra segurança imediata, autonomia ou reputação futura; não joga
  um minijogo de adivinhação contra um texto de IA (ver [07-diplomacia.md](07-diplomacia.md)).
- Quando o Governador propõe uma ação para o jogador ausente, a Pauta a rotula como “delegada” e
  exibe a prioridade do Mandato que a justificou. O jogador pode confirmar uma vez, substituir a
  ordem neste turno ou alterar o Mandato para os próximos. Linhas vermelhas e tetos permanecem
  visíveis junto à ação, para tornar a delegação revogável e auditável.
- Bots usam a mesma apresentação causal nas ofertas recebidas. O texto pode ter voz própria, mas os
  fatos do Ledger e as cláusulas são a fonte de explicação; uma justificativa ausente é tratada como
  falha de proposta, não como mistério dramático.

### Onboarding BYOK e custo de IA

- O onboarding começa explicando, em linguagem simples, que o jogo funciona sem IA paga usando
  fallback determinístico, mas que uma chave pode melhorar narrativa, planejamento e negociação. A
  chave não é condição para criar a primeira civilização.
- Fluxo em quatro passos: **(1)** escolher “sem chave por enquanto” ou informar chave; **(2)** testar
  a credencial por uma chamada mínima e informar somente sucesso, falha ou indisponibilidade, sem
  exibir ou registrar o segredo; **(3)** definir um teto diário e confirmar o que acontece ao atingi-lo;
  **(4)** criar civilização, escolher o primeiro Mandato e abrir a Pauta.
- Valor inicial, sujeito a balanceamento: o teto sugerido começa em zero até o jogador escolher um
  valor; avisos aparecem em 50%, 80% e 100% do teto. Em 100%, chamadas pagas param para aquela
  civilização até a próxima janela diária ou alteração explícita do teto; T0 continua resolvendo o
  jogo. A janela, moeda exibida e conversão do provedor precisam ser definidas tecnicamente antes de
  lançamento.
- O indicador de custo é persistente, discreto e tocável: “hoje: gasto / teto”. A tela de custo separa
  consumo por Governador, diplomacia, Entropia e narrativa, além de mostrar chamadas evitadas por
  cache ou fallback. O jogador decide quais capacidades delegar com base em custo e utilidade, não em
  surpresa após muitas sessões.
- Alterar chave, teto ou autorização de categorias requer confirmação explícita e produz um registro
  de preferência, nunca a chave em si. A experiência deve explicar que credenciais são processadas
  pelo servidor e que a simulação não depende da disponibilidade do provedor (ADR-0001 e
  [09-governador-e-mandato.md](09-governador-e-mandato.md)).

### Determinismo, IA e falhas previsíveis

- **Determinístico, no motor:** elegibilidade de ações e eventos; custo de recursos; efeitos de
  tratados; cálculo de pressão, coesão e colapso; validação do Mandato; transição de turno e criação
  do relatório a partir do log. O cliente só exibe opções que chegaram validadas e envia escolhas como
  comandos; o estado mostrado após a resolução vem do servidor.
- **Proposto pela IA:** o Governador pode propor uma intenção dentro do Mandato; bots podem propor
  termos diplomáticos e a Entropia pode escolher e parametrizar um template elegível. Cada proposta
  precisa referenciar entidades e fatos do estado. A IA também pode redigir a versão narrativa da
  Crônica, identificada como narrativa.
- **Validado por regras e dados:** schemas, catálogos de tratados e templates de eventos limitam os
  parâmetros e os efeitos. O motor aceita ou rejeita a intenção e grava o comando aceito. Uma IA não
  cria um efeito pelo texto, não escolhe uma cláusula fora do catálogo e não muda a interface para
  esconder uma rejeição (ADR-0006).
- Se uma chamada exceder o teto, expirar ou retornar inválida, a Pauta mostra “resolvido por regras
  locais” quando isso for material para a decisão. O fallback T0 propõe uma ação legal e auditável;
  ele não bloqueia o turno nem pede que o jogador reenvie a mesma escolha.

### Eras, colapso e retorno em um mundo infinito

- A interface de era não transforma progresso em uma pilha de números. Ao alcançar um marco, a
  Pauta oferece uma escolha de consolidação entre um pequeno conjunto de legados elegíveis — por
  exemplo, instituição, rede ou conhecimento — e mostra o custo de manutenção ou renúncia envolvido.
  Valor inicial, sujeito a balanceamento: 2–3 opções por marco e um legado ativo por categoria.
- Painéis de longo prazo usam tendências e marcos, não totais sem limite. Recursos excedentes devem
  aparecer como reserva com teto, manutenção futura ou oportunidade de conversão, nunca como um
  multiplicador permanente invisível. Assim, o jogador decide entre ampliar alcance, fortalecer a
  base ou guardar capacidade para a próxima crise (ver [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Se a civilização colapsar, a tela de sucessão primeiro explica a cadeia de pressão e as decisões
  anteriores; depois permite escolher uma comunidade sucessora elegível. Ruínas, memórias e relações
  herdadas aparecem como legado limitado, enquanto território e capacidade administrativa são
  reavaliados pelas regras. O retorno é uma nova sessão de fundação com história, não uma tela de
  derrota nem uma reinicialização com bônus acumulativos.
- A Entropia pode tornar vulnerabilidades antigas relevantes em novas eras, mas a UI agrupa eventos
  repetidos e mantém uma linha do tempo compactada. Crescer o mundo aumenta variedade de dilemas e
  não o número de notificações, telas ou ordens obrigatórias.

### Notificações, acessibilidade e métricas

- Notificações só convocam o jogador para escolha crítica que vence em poucos turnos, proposta diplomática
  que expira, gatilho de Mandato que pede intervenção ou crise iminente. Produção rotineira, relatos
  narrativos e ações delegadas bem-sucedidas ficam no relatório ao abrir o jogo.
- Cada notificação começa pela consequência (“fronteira pode se separar em um turno”), seguida da
  causa curta, e abre diretamente o cartão de decisão. Valor inicial, sujeito a balanceamento: no
  máximo uma notificação acionável por civilização a cada 6 horas, exceto quando uma decisão anterior
  do jogador tiver prazo menor explicitamente exibido.
- Todas as escolhas críticas são utilizáveis com leitor de tela, foco sequencial e rótulos que incluem
  efeito e custo. Cores de facção, risco e pressão têm equivalentes em ícone, textura e texto; o
  tamanho do texto, contraste e redução de movimento são configuráveis. Animações de resolução podem
  ser reduzidas sem ocultar informação.
- Métricas de protótipo propostas: mediana de sessão de 2–10 minutos; 80% das escolhas críticas
  concluídas em até 8 toques; 80% dos participantes identificam a causa principal de uma crise após
  ler o relatório; e nenhum teto de custo é ultrapassado sem confirmação registrada. São metas de
  validação, não decisões de produto aprovadas.

## Perguntas abertas
- Android primeiro? Cliente web/desktop para testes?
  - **Recomendação:** priorizar Android para validar a interação por toque e manter um cliente web ou
    desktop interno para teste e depuração do servidor. Isso concentra o primeiro produto no público
    alvo sem transformar ferramentas de desenvolvimento em promessa de plataforma pública.
- Orientação de tela (retrato ou paisagem)?
  - **Recomendação:** adotar retrato como padrão da pauta, relatórios e decisões; permitir paisagem
    somente no mapa se o cliente suportar a transição sem esconder ações críticas. Retrato favorece
    sessões de poucos minutos e alcance com uma mão, enquanto paisagem pode melhorar a leitura de
    mapas hexagonais densos.
