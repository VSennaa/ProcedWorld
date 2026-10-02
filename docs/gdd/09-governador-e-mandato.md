# 09 — Governador e Mandato

## Decidido
- O Governador é um bot do jogo, roda no servidor, e o jogador o configura pelo Mandato (ADR-0002).
- O motor bloqueia ações que violem o Mandato.
- **Ausência delega tudo, exceto o irreversível** (ADR-0008): com o jogador ausente, o Governador joga
  todas as áreas dentro do Mandato, mesmo as não delegadas; ações irreversíveis (iniciar guerra,
  romper tratado, ceder cidade, aceitar vassalagem, escolher sucessor) ficam suspensas até o jogador
  voltar. A civilização nunca para. Decidido em 2026-10-01.
- **Três presets visíveis** (Equilibrado, Recuperar, Crescer com cautela), mostrando linhas vermelhas
  e escopos antes do primeiro turno. Decidido em 2026-10-01.
- **O Governador sugere mudanças no Mandato só como recomendação** explicada, citando o fato que a
  motivou; apenas o jogador confirma. Decidido em 2026-10-01.

## Proposta

### Papel no loop

- O Mandato é a constituição operacional de uma civilização: define o que o Governador pode fazer
  entre as intervenções do jogador, não um plano que garante um resultado. Ele transforma uma
  intenção em uma escolha limitada por valores, recursos e compromissos já existentes.
- O Governador atua nas áreas delegadas e, quando o jogador está ausente, em todas as áreas exceto
  ações irreversíveis (ADR-0008). Ele não substitui uma decisão pendente que o Mandato marcou como “exigir confirmação”.
  Assim, voltar ao jogo significa retomar escolhas de rumo, não corrigir microgestão acumulada.
- A decisão recorrente do jogador é qual autonomia conceder agora: delegar crescimento libera uma
  sessão curta, mas pode consumir a reserva que seria útil para uma crise; reter diplomacia preserva
  um compromisso pessoal, mas exige estar presente quando uma proposta chegar.

### Cartão de Mandato

- A tela principal do Mandato é um cartão de uma página com cinco campos: **rumo**, **linhas
  vermelhas**, **reservas**, **postura externa** e **avisos**. Cada campo abre detalhes opcionais;
  nenhum ajuste rotineiro deve exigir editar texto ou visitar cada cidade.
- O cartão mostra uma previsão legível, não uma promessa: “delegado: cidades e pesquisa; não pode
  declarar guerra; reserva protegida: 30; avisar antes de tratado”. Uma alteração vale a partir do
  próximo turno aceito, para que comandos do turno corrente permaneçam auditáveis.
- O jogador pode salvar até três Mandatos nomeados e alternar entre eles em dois toques, por exemplo
  “Recuperar”, “Expandir com cautela” e “Preservar paz”. Os três espaços são um **valor inicial,
  sujeito a balanceamento**; o objetivo é trocar de orientação, não manter uma biblioteca complexa.

### Rumo e prioridades

- O rumo contém quatro prioridades: **segurança**, **sustento**, **desenvolvimento** e **relações**.
  O jogador distribui 100 pontos entre elas em incrementos de 10, ou escolhe um preset. Prioridade
  zero não proíbe a área; apenas impede que ela consuma recursos discricionários sem responder a uma
  obrigação ou crise já validada.
- Para cada ação candidata permitida, o motor calcula uma utilidade inteira:
  `U = 3·Bseg + 3·Bsus + 2·Bdes + 2·Brel − 2·Cop − X`, em que `Bseg`, `Bsus`, `Bdes` e `Brel` são o
  benefício normalizado de segurança, sustento, desenvolvimento e relações, ponderados pelos pontos
  do rumo; `Cop` é custo de oportunidade e `X` é risco exposto pela ação (nomes distintos de coesão
  `C`, privação `D` e estabilidade `S` de outros pilares). Cada termo fica entre 0 e 100 e vem de regras e
  catálogos; pesos e faixa são **valores iniciais, sujeitos a balanceamento**.
- O Governador escolhe a maior `U` apenas entre ações válidas e com benefício acima de 10; em empate,
  usa uma ordem estável de identificadores. A escolha é interessante porque o jogador define qual
  perda aceita: segurança alta pode preservar tropas e fronteiras ao preço de atrasar recuperação ou
  inovação; relações altas pode sustentar um aliado, consumindo recursos em casa.
- A fórmula não decide produção, pesquisa ou guerra sozinha. Os respectivos pilares oferecem ações
  candidatas com benefícios, custos e pré-requisitos; o Mandato apenas ordena o que já é possível
  (ver [03-economia.md](03-economia.md), [05-tecnologia.md](05-tecnologia.md) e
  [07-diplomacia.md](07-diplomacia.md)).

### Linhas vermelhas e reservas

- Linhas vermelhas são proibições absolutas, começando por: não iniciar guerra, não romper tratado,
  não ceder cidade, não gastar reservas em diplomacia e não deslocar população de uma cidade. Elas
  são escolhas por alternância, não instruções em linguagem natural. Novas linhas só existem quando
  um template de ação tiver uma categoria validada correspondente.
- Reservas definem tetos por turno: gasto de tesouro, consumo de estoque estratégico e perda aceita
  de unidades. Cada teto pode ser “zero”, “baixo”, “médio” ou “alto”, mapeado para 0%, 10%, 25% e
  40% do recurso disponível no começo do turno — **valores iniciais, sujeitos a balanceamento**.
  Reservas já prometidas por tratado ou necessárias para sobrevivência não entram na parcela livre.
- Uma guarda de sobrevivência é aplicada antes do teto: o motor não converte recurso se isso deixar
  uma cidade abaixo do abastecimento mínimo ou uma defesa abaixo do mínimo legal do seu estado. Isso
  impede que um Mandato agressivo burle regras de população, estabilidade ou tratados para otimizar
  um número (ver [04-cidades-e-populacao.md](04-cidades-e-populacao.md) e
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md)).
- O jogador escolhe reservas porque elas tornam o custo de delegar visível. Proteger 25% do tesouro
  reduz respostas imediatas à escassez, mas preserva margem para o jogador negociar ou reagir na
  próxima sessão.

### Postura diplomática

- A postura externa é uma preferência entre **conciliadora**, **recíproca** e **dissuasória**.
  Ela muda o conjunto de intenções que o Governador pode apresentar, nunca a máquina de estados, a
  aceitação nem os efeitos de uma relação.
- A postura conciliadora pode propor ajuda e tratados permitidos; a recíproca só propõe concessão
  quando há benefício equivalente ou dívida reconhecida; a dissuasória prioriza reforço, aviso e
  propostas de garantia. Declaração de guerra continua bloqueada se for linha vermelha ou estiver
  fora do escopo delegado.
- Bots usam o mesmo modelo de postura e os mesmos limites de tratado. Sua IA pode propor o texto e
  a intenção diplomática com fatos do Ledger; o motor calcula aceitação, custos, transição de estado
  e entradas no Ledger. Isso preserva a coesão da diplomacia e torna uma recusa explicável (ver
  [07-diplomacia.md](07-diplomacia.md)).
- A escolha interessa porque um vizinho ameaçador pode exigir dissuasão agora, enquanto uma sociedade
  com privação pode precisar de conciliação para importar alimento. Nenhuma postura garante paz ou
  prosperidade: a outra civilização, o Ledger e os limites materiais continuam importando.

### Escopo de delegação e confirmação

- O jogador liga ou desliga cinco escopos: cidades e economia, exploração e defesa, tecnologia,
  diplomacia e resposta a crises. Todos começam desligados no primeiro Mandato proposto; o preset
  escolhido pode ligá-los de forma explícita. Cinco escopos são um **valor inicial, sujeito a
  balanceamento** e agrupam decisões sem exigir menus por subsistema.
- Com o jogador presente, uma ação só é candidata se pertencer a um escopo ligado; ausente, vale a
  regra de ausência do "Decidido". Ações irreversíveis ou que mudam
  soberania — iniciar guerra, romper tratado, ceder cidade, aceitar vassalagem e escolher sucessor
  após colapso — exigem confirmação do jogador por padrão, mesmo com o escopo ligado. Essa lista é
  uma **proposta**, sujeita à definição dos estados diplomáticos e do ciclo (ver
  [07-diplomacia.md](07-diplomacia.md) e [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Para cada escopo, há apenas dois modos: “agir dentro dos limites” ou “propor e aguardar”. O segundo
  mantém a intenção no painel até o jogador responder; se o turno avançar com ele ausente (ADR-0008),
  o fallback é não agir, exceto por
  defesa automática que uma regra de sobrevivência já permita. O jogador decide entre continuidade
  e controle sem precisar configurar exceções em cascata.

### Crise, Entropia e jogo infinito

- A Entropia não lê nem altera o Mandato como se fosse uma ordem. Ela escolhe templates de evento
  elegíveis pelo estado e pelo orçamento de tensão; o Mandato define apenas quais respostas do
  Governador são permitidas depois que o evento gera uma pressão real (ver
  [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).
- Se pressão de crise atingir 40 ou mais, o Governador pode realocar apenas uma ação discricionária
  por turno para mitigação; em 70 ou mais por dois turnos, pode usar a reserva do nível selecionado
  se “resposta a crises” estiver delegada. Os limiares, a ação por turno e a duração são **valores
  iniciais, sujeitos a balanceamento** e devem usar a pressão e a coesão de
  [00-visao.md](00-visao.md), não uma segunda escala de crise.
- O Mandato nunca autoriza sacrificar coesão para esconder um problema: toda resposta que aumenta
  privação, tensão de grupos ou exposição precisa mostrar esse risco no relatório. O jogador pode
  escolher preservar uma fronteira ou uma comunidade sabendo qual componente da pressão piora.
- Em eras posteriores, o Mandato não ganha mais pontos de prioridade, mais categorias ou tetos
  multiplicativos. Novas tecnologias liberam ações candidatas, enquanto alcance territorial,
  instituições e tratados aumentam manutenção e obrigações. O mesmo cartão continua legível quando
  a civilização é grande (ver [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Em colapso, todos os escopos passam para “propor e aguardar”, exceto sobrevivência básica validada
  pelo motor. Após o jogador escolher uma comunidade sucessora, ela herda as linhas vermelhas e a
  postura, mas zera prioridades para o preset “Recuperar”; reservas são recalculadas sobre a nova
  escala. Herança, preset e recálculo são **propostas, sujeitos a balanceamento**. Isso cria um
  renascimento legível sem carregar poder ou autonomia ilimitados de eras anteriores.

### Presets e relatório

- Três presets reduzem a fricção inicial: **Equilibrado** (25 pontos em cada prioridade),
  **Recuperar** (40 sustento, 30 segurança, 20 relações, 10 desenvolvimento) e **Crescer com
  cautela** (35 desenvolvimento, 30 sustento, 20 segurança, 15 relações). Os valores são **iniciais,
  sujeitos a balanceamento**; cada preset expõe antes de confirmar seus escopos, reservas e linhas
  vermelhas.
- O relatório do Governador abre com no máximo três cartões: “fiz”, “não fiz” e “precisa de você”.
  Cada cartão contém ação ou intenção, custo, efeito previsto ou resolvido, regra do Mandato aplicada
  e dois fatos do estado que a justificam. O jogador pode abrir o detalhe para ver comando, template
  e referência do Ledger ou da Crônica.
- Exemplo: “Importei alimento: sustento 40 e crise delegada; estoque cairia abaixo do mínimo em dois
  turnos. Custo: 10 de tesouro; consequência: dívida comercial com Aram.” O relatório ensina a cadeia
  causal em uma leitura curta e permite ajustar o Mandato em vez de desconfiar de uma caixa-preta.
- Avisos são gatilhos determinísticos: intenção que exige confirmação, guerra declarada contra a
  civilização, pressão de crise em 40, coesão abaixo de 30, reserva prestes a ser usada e proposta
  diplomática que expira no próximo turno. Valores 40 e 30 são **iniciais, sujeitos a balanceamento**.
  Mudanças rotineiras entram apenas no relatório para não transformar o celular em um alarme contínuo.

### Aplicação pelo motor e limites da IA

- **Determinístico:** carregar a versão do Mandato no início do turno; filtrar escopo, linhas
  vermelhas, confirmação, pré-requisitos e reservas; calcular `U`; ordenar empates; converter a ação
  válida em comando; aplicar efeitos de templates; registrar rejeições, comandos, hash e relatório.
  O fallback T0 usa a mesma lista filtrada e a mesma ordenação.
- **Proposto pela IA:** o Governador pode escolher uma intenção tipada dentre ações candidatas e
  fornecer justificativas ligadas a IDs de estoque, cidade, evento ou Ledger. Entropia pode escolher
  e parametrizar um template elegível; bots podem propor ofertas e textos diplomáticos. Nenhuma dessas
  saídas define números, cria efeitos, aceita tratado ou altera o Mandato diretamente.
- A validação ocorre antes de virar comando. Intenção inválida, sem fatos de grounding, fora do
  Mandato, expirada, timeout ou indisponibilidade de provedor é descartada; o motor usa T0 ou não age
  conforme o escopo. O comando aceito e a versão do Mandato entram no event log, permitindo replay
  sem nova chamada de IA, conforme ADR-0006.
- O Governador pode sugerir uma alteração como uma intenção separada — por exemplo, “aumentar reserva
  para médio” — mas ela gera somente cartão de recomendação. Apenas o jogador confirma e o motor
  grava a mudança para um turno futuro.

## Perguntas abertas

- Valores exatos dos presets e dos limiares de aviso (balanceamento no harness).
