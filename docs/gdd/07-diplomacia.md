# 07 — Diplomacia

## Decidido
- Ações diplomáticas de bots precisam ser justificáveis pelo estado e pelo histórico.
- Máquina de estados explícita mais Ledger de Relações; o motor calcula a aceitação e o LLM gera
  a proposta e o texto (CLAUDE.md §3.2).

## Proposta

### Papel no ciclo

Diplomacia converte proximidade, dependência e memória em compromissos verificáveis. Ela não é uma
barra de amizade: um vizinho pode ser confiável e competir por território, ou hostil e ainda cumprir
um acordo comercial. Cada relação tem direção própria para confiança e obrigações, embora os fatos
comuns (guerra, tratado) sejam visíveis aos dois lados.

O jogador deve poder resolver a decisão principal em poucos toques: revisar uma proposta, ver seus
termos e riscos, então aceitar, rejeitar ou fazer uma contraproposta estruturada. Um relatório curto
explica quais fatos pesaram e quando o compromisso vence.

### Máquina de estados

Cada par de civilizações mantém um estado canônico: `desconhecido`, `contato`, `paz`, `tensão`,
`pacto`, `aliança`, `guerra` ou `trégua`. `Vassalagem` é um vínculo de soberania sobreposto a paz,
pacto ou trégua, não um estado que apaga a relação entre suserano e terceiros. A relação começa em
`desconhecido`; explorar/contatar abre `contato`; uma primeira interação aceita estabelece `paz`.

Transições só ocorrem por comandos válidos ou regras de prazo: quebra de tratado pode elevar paz a
tensão; declaração válida pode iniciar guerra; aceite de termos encerra guerra em trégua, que volta
a paz ao fim do prazo se não houver violação. `Pacto` cobre um compromisso delimitado (por exemplo,
passagem ou comércio); `aliança` adiciona obrigação de defesa. Romper qualquer um encerra o vínculo e
registra a quebra antes de recalcular o estado. Não há transição causada apenas por prosa do LLM.

Toda transição tem pré-condições (estado atual, atores, recursos, prazo e autoridade), comando,
efeito definido e evento de ledger. Proposta de transição inválida é rejeitada sem efeito e recebe
motivo enumerado para interface e auditoria.

### Ledger de Relações e memória

O Ledger é por par e por direção. Uma entrada imutável registra turno, tipo, intensidade, origem,
partes, prazo, estado (ativa, cumprida, expirada ou quebrada), referência ao comando/evento e fatos
que a sustentam. Categorias: promessa, dívida, ofensa, ajuda, comércio, fronteira, incidente e
tratado. Saldos são derivados das entradas; não se editam saldos à mão.

Para cada direção, confiança `C` e ressentimento `R` usam escala inteira 0–100, e dívida líquida
`D` usa faixa limitada de −100 a +100; valores são iniciais, sujeitos a balanceamento. Um fato de
intensidade `w` (1–10, valor inicial, sujeito a balanceamento) aplica deltas catalogados. Exemplo:
ajuda aceita soma até `+2w` em confiança e cria dívida favorável; promessa cumprida soma `+w`,
quebrada subtrai `2w` de confiança e soma `2w` de ressentimento. Guerra e tratado alteram valores
apenas pelas regras do template, sem interpretação narrativa.

Decaimento ocorre no fechamento de cada era, não a cada turno: confiança move 1 ponto em direção
à base neutra 50; ressentimento cai 1 ponto; dívida sem vencimento cai 1 ponto em direção a zero.
São valores iniciais, sujeitos a balanceamento. Ofensa grave e promessa quebrada criam uma marca de
traição com causa, era e severidade; ela não desaparece pelo decaimento comum. Uma reconciliação
formal pode reduzir severidade em passos definidos por dados, após reparação aceita. Assim, o
jogador escolhe entre exigir reparação, perdoar com risco ou congelar contato.

Para manter custo e legibilidade em mundos infinitos, o Ledger preserva todos os fatos no log
reproduzível, mas agrega fatos antigos por era em resumos com referência aos eventos originais.
Somente compromissos ativos e marcas de traição recentes entram no cálculo corrente; resumos antigos
continuam consultáveis na Crônica. Nenhum histórico é apagado nem vira bônus acumulativo ilimitado.

### Propostas e aceitação pelo motor

Uma proposta é um objeto estruturado: tipo, emissor, destinatário, termos, duração, custo, condição
de conclusão e fatos citados. Termos vêm de catálogo validado: comércio com quantidades limitadas,
passagem por rota/território, ajuda, pacto, aliança, reparação, trégua, cessão territorial ou
vassalagem. O motor valida capacidade, fronteiras, Mandato, compatibilidade de estado e ausência de
termos impossíveis antes de calcular aceitação.

Para uma contraparte automática, o motor calcula `S = 40 + 0,30(C−50) − 0,25R + 0,20D + U − K`,
limitado a 0–100; `C`, `R` e `D` são os valores direcionais normalizados para −100..100 onde
necessário, `U` é utilidade concreta dos termos (−20..+20) e `K` é custo/risco (0..30). Faixas e
pesos são valores iniciais, sujeitos a balanceamento. Aceita se `S ≥ 60`, recusa se `S < 40` e,
entre esses limites, oferece contraproposta somente se houver template válido que melhore custo sem
violar restrições. Guerra, trégua e vassalagem podem exigir limiares próprios em dados. O motor
registra componentes e limiar usados; o resultado não depende de texto nem de chamada externa.

Para escolhas de jogador, o motor valida e apresenta custo, efeito e prazo; não simula uma aceitação
que ainda cabe ao jogador. Em negociação entre bots, ambos usam a mesma fórmula. Uma rodada admite
até duas contrapropostas e expira no fim do turno seguinte; valor inicial, sujeito a balanceamento.
Expiração não é ofensa. Repetição do mesmo pedido recusado fica bloqueada por dois turnos, salvo
mudança material dos fatos citados.

### Linguagem natural e papel das IAs

A interface oferece cartões de termos para decisões rápidas. Linguagem natural é uma camada opcional
para pedir “negocie passagem em troca de grãos”: o LLM traduz para uma intenção estruturada que cita
entidades, quantidades e fatos do estado. A validação rejeita campos ausentes, IDs inexistentes,
valores fora dos limites e cláusulas sem template. Texto livre nunca cria obrigação mecânica.

Determinístico: validação, pontuação de aceitação, transições, prazos, custos, deltas do Ledger,
resolução de guerra e fallback. Proposto por IA: intenção de negociar, seleção entre opções
permitidas, redação de termos já estruturados e texto explicativo. O Governador segue o Mandato,
incluindo limites de concessão e guerra; se não houver resposta de IA, T0 escolhe uma ação válida
por prioridades configuradas. Bots usam as mesmas opções e regras, sem acesso privilegiado.

A Entropia pode propor um incidente diplomático somente via template elegível, com partes, gatilho e
severidade limitados (ver 08-entropia-e-eventos.md). O motor confirma o gatilho, registra o incidente
e aplica o delta fixo; a Entropia redige a crônica, mas não escolhe arbitrariamente culpados ou
resultados. Incidentes devem pressionar uma vulnerabilidade existente e permitir resposta jogável.

### Guerra e paz negociada

Uma intenção de guerra precisa citar uma reivindicação ou agressão registrada e um objetivo do
catálogo: proteger rota, conter incursão, recuperar território ou forçar reparação. O motor verifica
estado, autoridade, Mandato e elegibilidade; só então aceita o comando de declaração. Objetivo define
condição de encerramento e faixa de custo, não concede vitória automática. Guerra altera ameaça e
coesão conforme regras de sociedade (ver 06-sociedade-e-governo.md), e disputa rotas/recursos da
Economia (ver 03-economia.md).

Durante guerra, cada lado pode oferecer trégua estruturada. Termos especificam cessar ataques,
retirada, reparação limitada, troca/retorno de território e duração; o motor calcula aceitação com
risco de cada cláusula. O jogador decide se aceita a paz mesmo quando o custo parece alto: prolongar
a guerra pressiona recursos e coesão, enquanto concessões podem gerar dívida e ressentimento. Violação
posterior reabre tensão ou guerra conforme a cláusula e cria marca de traição. Rendição total não é
a única saída e não apaga a identidade do derrotado.

### Ciclo infinito, colapso e renascimento

Com novas eras, relações acompanham a sociedade sucessora por continuidade institucional, não por
multiplicador de poder. Em colapso, tratados de defesa e obrigações militares expiram; dívidas e
reivindicações passam apenas se um template de sucessão identificar herdeiro e território. O novo
regime pode ratificar, renegociar ou repudiar vínculos, com consequências registradas. Renascimento
permite reabrir contato e contestar fronteiras, mas não limpa marcas de traição nem concede confiança
instantânea.

A escala fica estável por limites nos saldos, decaimento por era, limites de proposta e agregação
histórica. Novos templates podem alterar termos, não remover tetos sem revisão de dados. Relações
entre muitos impérios não aumentam custo por chamada de IA: o motor prioriza relações com fronteira,
tratado ativo, guerra ou evento recente; outras recebem atualização determinística em lote. Crises
ambientais ou sociais podem tornar ajuda diplomática decisiva (ver 08-entropia-e-eventos.md e
06-sociedade-e-governo.md), mas a pressão e seus efeitos continuam calculados pelas regras desses
pilares.

### Auditoria e experiência móvel

Cada intenção diplomática de bot produz um registro de auditoria: ator, intenção/schema, decisão final,
transição, fatos de estado citados, IDs das entradas do Ledger, componentes de pontuação, versão de
regras/templates e motivo de fallback ou rejeição. O relatório do harness lista esses dados por ação;
intenções sem referência válida são descartadas. O texto do modelo é armazenado como entrada externa
para replay, mas não é reexecutado nem tratado como prova mecânica (ADR-0006).

Na tela, mostrar resumo de uma frase, até três fatos justificadores, termos em cartões e botões aceitar,
rejeitar ou ajustar. Um toque abre histórico e impacto futuro; notificações são reservadas a guerra,
quebra de tratado e prazo de paz. Isso preserva sessões curtas sem esconder compromissos permanentes.

## Perguntas abertas
- Jogadores negociam com bots em linguagem natural (via LLM), por propostas estruturadas, ou ambos?
  - **Recomendação:** ambos: cartões estruturados como caminho principal e linguagem natural opcional
    convertida em intenção tipada. Assim a negociação é rápida no celular e flexível, sem deixar o
    texto determinar regras ou resultados.
- Guerra: objetivos de guerra explícitos e paz negociada?
  - **Recomendação:** sim, com objetivos de catálogo e trégua negociada como descrito acima. Isso
    torna custos e saídas legíveis, conecta guerra a recursos e coesão e evita que vitória militar
    vire eliminação automática num mundo persistente.
- Relações sobrevivem a colapso e renascimento? Sob quais condições?
  - **Recomendação:** preservar apenas dívidas, reivindicações e traições vinculadas a sucessor
    identificável; encerrar obrigações de defesa e permitir ratificação. A continuidade parcial dá
    peso à história sem prender um novo regime a tratados impossíveis.
