# 00 — Visão

## Decidido
- 4X por turnos, em tiles hexagonais, no espírito de *Civilization*, com ciclo infinito (sem vitória obrigatória).
- Mundo persistente num servidor autoritativo (ADR-0001); alvo principal é o celular.
- Civilizações de jogadores e bots convivem; Governadores agem pelos jogadores (ADR-0002).
- A Entropia gera eventos procedurais; a diplomacia entre bots precisa ser coerente.
- **Tamanho do mundo**: 4–8 civilizações no MVP; teto de 8 na 1.0. **Uma civilização por jogador** em
  cada mundo (decidido em 2026-10-01).
- **Tom**: história alternativa de baixa fantasia. **Magia pode surgir naturalmente** no mundo, quando
  a Entropia decidir: pode ser oculta ou intrínseca a uma sociedade, e pode ser *hard magic* (regras
  explícitas) ou *soft magic* (misteriosa) no plano narrativo; o sistema pode se basear em algum
  sistema de magia livre. O impacto no mundo vem da **reação das nações** — aderir, proibir,
  regulamentar —, e tudo tem consequência mecânica no jogo (decidido em 2026-10-01).
- **Sistema de magia próprio**, desenhado para o motor, com inspirações livres que não exijam
  atribuição (nenhuma regra, texto ou nome copiado de obra protegida). Decidido em 2026-10-01.

## Proposta

### Promessa e fantasia

- O jogador é fundador, guardião e conselheiro de uma sociedade que pode sobreviver a ele.
  Ele escolhe valores e prioridades; cidades, grupos, vizinhos e ambiente reagem com causas legíveis.
- A fantasia não é controlar cada cidadão: é reconhecer uma crise a tempo, escolher o que preservar
  e ver essa escolha mudar a história da sociedade ao longo de eras.
- A pergunta recorrente é: **o que esta sociedade deve proteger agora, e o que aceita arriscar?**
  Expandir pode trazer recursos e fronteiras frágeis; reformar pode reduzir tensões e dividir grupos;
  ceder numa negociação pode comprar tempo, mas gerar uma dívida política.
- O resultado desejado é uma crônica jogável: impérios podem ruir, mas suas instituições, costumes,
  tratados, ruínas e relatos deixam marcas verificáveis no mundo.

### Público e limites de produto

- Público primário: pessoas que gostam de 4X sistêmico, mas preferem decisões concentradas em
  intervalos curtos a uma sessão contínua de várias horas.
- Público secundário: fãs de histórias emergentes e simulação social de *RimWorld* e *Dwarf Fortress*,
  que gostam de descobrir por que uma sociedade mudou, e não só de maximizar uma pontuação.
- A experiência deve servir tanto a quem entra diariamente para resolver uma crise quanto a quem
  delega parte da administração e retorna para ler o relatório e escolher um novo rumo.
- O jogo não promete ausência de perdas: promete que perdas importantes terão causas no estado,
  escolhas de resposta e consequências persistentes. Não promete que a IA “entende” o mundo; regras
  e histórico devem tornar decisões auditáveis mesmo quando o texto narrativo falhar.

### Referências: aproveitar e evitar

| Referência | Aproveitar | Evitar |
|---|---|---|
| *Civilization* | Leitura clara do mapa, expansão por hexágonos e escolhas com efeitos compostos. | Corrida por vitória como único objetivo ou crescimento que só aumenta rendimentos. |
| *RimWorld* | Eventos que pressionam pontos frágeis e transformam sistemas em histórias pessoais. | Catástrofes arbitrárias, opacas ou encadeadas sem chance de resposta útil. |
| *Dwarf Fortress* | História acumulada, lugares e personagens com consequências que persistem. | Exigir microgerenciamento ou conhecimento enciclopédico para tomar uma decisão razoável. |
| *Old World* | Decisões políticas e relações ancoradas em pessoas, compromissos e memória. | Obrigar uma sucessão de ordens urgentes que pune quem não fica conectado. |
| *Stellaris* | Escala de eras, identidades coletivas e mudanças de rumo em uma galáxia persistente. | Nevoeiro de sistemas e números que crescem sem limites legíveis ou custo de manutenção. |

### Núcleo jogável proposto: pressão, escolha e legado

1. **Ler a situação.** No começo da sessão, um resumo mostra uma mudança importante, sua causa
   rastreável e até três decisões que realmente alteram o rumo. O jogador pode abrir o mapa ou os
   detalhes da crônica, mas não precisa revisar cada cidade para compreender a urgência.
2. **Escolher um compromisso.** Diante de uma pressão, escolher uma resposta de curto prazo e um
   custo explícito: por exemplo, racionar durante uma seca, importar alimento assumindo uma dívida,
   ou deslocar população e abandonar uma fronteira. A escolha deve mudar quais sistemas ficam
   protegidos e quais ficam expostos; não deve haver uma opção dominante em todo contexto.
3. **Definir o rumo da era.** Em um marco de era, selecionar um foco social entre opções oferecidas
   pelo estado — por exemplo, integração, conhecimento ou autonomia local — e aceitar uma renúncia.
   O foco dá um benefício limitado e previsível; a renúncia abre espaço para reforma ou recuperação.
4. **Ver a resposta do mundo.** Na resolução do turno, regras de produção, estabilidade, clima e
   relações aplicam as consequências. A Crônica resume a cadeia entre decisão, resultado e próximo
   risco, para que voltar depois de uma ausência continue compreensível.

- **Coesão social (regra proposta):** escala inteira de 0 a 100, representando capacidade de manter
  compromissos comuns, não felicidade individual nem pontuação de vitória. Ganhos e perdas vêm de
  regras explícitas de estabilidade, abastecimento, reformas, conflitos e ajuda externa.
- **Pressão de crise:** calculada ao fim do turno por cidade (`P_c`) e por civilização (`P_civ`, média
  ponderada por população), a partir de privação `D`, tensão de grupos `G`, ameaça de guerra `W`,
  exposição ambiental `E`, estabilidade local `S` (peso reduzido) e coesão `C`. Fórmula canônica em
  [12-variaveis-e-formulas.md](12-variaveis-e-formulas.md).
- **Limiar de resposta (proposta):** P de 0–39 indica tensão administrável; 40–69 habilita um aviso
  e uma decisão de mitigação; 70–100 habilita crise se persistir por dois turnos. São valores
  iniciais, sujeitos a balanceamento. Um evento pode antecipar perigo, mas não pode ignorar as regras
  de elegibilidade ou eliminar uma civilização sem uma cadeia de causas e resposta.
- **Foco de era (regra proposta):** um foco dura até o próximo marco de era. Concede +10 pontos
  percentuais de eficiência em uma área escolhida e impõe −5 em outra, por exemplo integração
  cultural contra autonomia local. É um modificador pequeno, não um multiplicador acumulativo.
  Faixas e combinações são valores iniciais, sujeitos a balanceamento.
- **Colapso parcial (regra proposta):** se a coesão ficar em 0 por dois turnos, ou se uma crise grave
  validada reduzir o governo a incapaz de cumprir suas funções, ocorre uma transição jogável. O motor
  divide ou reorganiza territórios conforme controle, distância e regras de sucessão; o jogador escolhe
  uma comunidade sucessora para continuar. Não há eliminação por um único sorteio da Entropia.

### Conexões entre pilares

- **Território e clima:** biomas, recursos e exposição dos hexágonos definem privação e risco ambiental;
  a expansão traz capacidade produtiva, mas aumenta a área a defender e administrar (ver
  [02-mapa-e-tiles.md](02-mapa-e-tiles.md)).
- **Economia, cidades e sociedade:** abastecimento e manutenção alimentam privação; satisfação dos
  grupos alimenta tensão e coesão. Uma resposta econômica pode estabilizar cidades e, ao mesmo tempo,
  sacrificar pesquisa ou comércio (ver [03-economia.md](03-economia.md),
  [04-cidades-e-populacao.md](04-cidades-e-populacao.md) e
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md)).
- **Entropia:** escolhe um template elegível que pressione uma vulnerabilidade real — seca onde há
  exposição, revolta onde há tensão, incidente onde existe relação pertinente. Orçamento de tensão,
  limites do template e proteção contra repetição controlam frequência e severidade. A Entropia pode
  propor parâmetros válidos; o motor valida e aplica efeitos determinísticos (ver
  [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).
- **Diplomacia:** comércio, promessas, ofensas e ajuda alteram dívida e confiança no Ledger. Uma
  aliança pode aliviar privação ou conter ameaça, mas cria compromissos que pesam em decisões futuras.
  A intenção de um bot deve apontar fatos do Ledger; a aceitação e os efeitos são calculados pelo motor
  (ver [07-diplomacia.md](07-diplomacia.md)).
- **Tecnologia e eras:** marcos tecnológicos mudam opções disponíveis, não elevam todos os números
  indefinidamente. Após colapso, conhecimentos podem se perder, persistir em instituições ou ser
  redescobertos conforme regras de legado (ver [05-tecnologia.md](05-tecnologia.md) e
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).

### Determinismo, IA e agência

- **Determinístico:** cálculo de pressão, elegibilidade e efeito de eventos; resolução de ordens;
  alteração de coesão, recursos, relações, transições de era e colapso; validação de limites do
  Mandato. Tudo é regra do motor, com seed e comandos registrados conforme ADR-0006.
- **Proposta de IA:** Governador, bots diplomáticos e Entropia podem escolher entre intenções tipadas
  permitidas. Cada intenção cita fatos do estado, como escassez, ameaça, Mandato ou entradas do
  Ledger. Templates e schemas de dados validados definem as opções e faixas mecânicas.
- O Governador propõe ações dentro do Mandato: se o jogador delegou, pode priorizar mitigação ou
  manter o foco de era; se há uma linha vermelha, a intenção incompatível é rejeitada. Sem resposta
  de IA, fallback T0 escolhe uma opção válida de acordo com prioridades fixas e auditáveis (ver
  [09-governador-e-mandato.md](09-governador-e-mandato.md)).
- Bots usam as mesmas regras e opções que o jogador. Em diplomacia, um modelo pode redigir uma
  proposta e justificativa; o motor calcula aceitação, muda o estado e registra fatos no Ledger.
  Narrativa pode variar, mas nunca altera o resultado mecânico (ver [07-diplomacia.md](07-diplomacia.md)).
- Saída inválida, timeout ou modelo indisponível não pausa o turno: o motor descarta a intenção e usa
  fallback determinístico. Respostas aceitas entram no log para que o replay não chame a IA de novo.

### Ciclo infinito sem inflação

- Eras avançam por marcos do mundo e da civilização, com salvaguarda de duração para impedir espera
  indefinida. Duração decidida: por marcos, de 4 a 12 turnos, com média esperada de 6–8 (ver
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Rendimentos e capacidades crescem por novas opções, eficiência e especialização, não por duplicação
  automática a cada era. Manutenção de cidades, rotas e instituições aumenta com alcance e complexidade;
  custos e rendimentos usam faixas/tetos definidos em dados (ver [03-economia.md](03-economia.md)).
- Ao fechar uma era, excedentes acima dos limites convertidos em manutenção, reservas ou legado; não
  são carregados como multiplicadores sem teto. O jogador escolhe o que consolidar: infraestrutura,
  conhecimento institucional ou uma rede diplomática. Essa escolha troca poder imediato por resiliência.
- Colapso reduz escala administrativa e pode fragmentar o território. O sucessor começa com uma
  fração da capacidade anterior — faixa proposta de 40–60%, valor inicial sujeito a balanceamento —
  e conserva apenas legados definidos, como traços culturais, crônica, ruínas e alguns conhecimentos.
  A redução transforma a pressão em novos dilemas sem apagar a história do jogador.
- Renascimento é a reorganização de uma comunidade, não um bônus grátis: oferece nova base de coesão,
  mas obriga a negociar autonomia, território e reivindicações com antigos vizinhos. Regras de
  densidade, manutenção e teto de legados evitam que ciclos sucessivos acumulem expansão gratuita.

### Celular e ritmo de sessão

- Alvo de sessão proposto: 2–8 minutos. A primeira tela resume “o que mudou / por quê / o que posso
  fazer agora”; cada crise oferece até três respostas comparáveis. Ações frequentes devem caber em
  poucos toques, com detalhes progressivos para quem quiser investigar.
- O jogador pode confirmar um plano de vários turnos, delegar áreas pelo Mandato ou adiar uma decisão
  quando as regras permitirem. Notificações devem priorizar guerra, risco de colapso e escolha com
  prazo; mudanças rotineiras ficam no relatório. Isso preserva agência sem cobrar presença constante.
- Métricas de sucesso propostas para protótipo fechado: mediana de sessão entre 2 e 8 minutos; 80% das
  decisões críticas concluídas em até 8 toques; pelo menos 80% dos participantes explicam corretamente
  a causa principal de uma crise após ler o resumo; nota média mínima de 4/5 para coerência percebida
  das decisões diplomáticas, com justificativas ligadas a fatos verificáveis.
- Métrica de retorno proposta: pelo menos 30% dos testadores voltam em sete dias após três sessões
  concluídas, sem notificações obrigatórias. Medir separadamente jogadores ativos e delegadores; uma
  boa taxa de retorno não deve recompensar ansiedade nem sessões longas.
- Esses números são critérios iniciais de experimento, sujeitos a validação com jogadores, e não
  decisões de produto já aprovadas. Instrumentação deve contar sessões, duração, toques em decisões,
  compreensão causal e avaliação de justificativas sem registrar conteúdo pessoal desnecessário.

## Perguntas abertas

- Catálogo inicial de fenômenos mágicos: **poucos (3–5 no MVP), raros e sistêmicos**, decidido em
  2026-10-01; o conteúdo de cada um é desenhado no catálogo (ver 05, 06 e 08).
