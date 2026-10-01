# 04 — Cidades e população

## Decidido
- (nada ainda)

## Proposta

### Papel da cidade no turno

- A cidade converte território e pessoas em comida, materiais, serviços e capacidade de agir.
  O dilema recorrente é usar o próximo trabalhador e investimento para crescer, proteger a
  população atual ou sustentar a expansão. Uma cidade maior não é automaticamente melhor.
- Cada cidade guarda população agregada, moradia, reserva local de comida, fila de construção,
  tiles controlados, postos, grupos de interesse, estabilidade e histórico de protesto.
  Um ponto de população é uma unidade de trabalho e consumo; grupos repartem esse total.
- Fundação exige tile válido, custo de assentamento e ligação possível com a civilização,
  conforme [02-mapa-e-tiles.md](02-mapa-e-tiles.md) e [03-economia.md](03-economia.md).
  Fundar longe pode assegurar recurso raro, mas aumenta custo administrativo e fragilidade.
- Ordem proposta da resolução: validar comandos; atribuir trabalho; produzir; consumir e pagar
  manutenção; avançar obras; calcular crescimento e migração; atualizar grupos, estabilidade
  e coesão; avaliar protestos e transições. A Entropia entra no ponto previsto da resolução,
  sem reordenar comandos aceitos (ver [01-loop-e-turnos.md](01-loop-e-turnos.md)).

### Trabalho dos tiles e construções

- Cada ponto de população ocupa no máximo um posto: um tile trabalhado ou um posto de edifício.
  Só uma cidade trabalha cada tile no turno. Quando duas disputam um tile controlado, o dono
  escolhe a cidade; sem escolha, o motor mantém a atribuição anterior ou usa o menor id.
- Raio de trabalho: até dois hexágonos do centro, condicionado por controle e ligação local.
  É um **valor inicial, sujeito a balanceamento**. Terreno, recurso, clima e melhorias definem
  postos e rendimentos inteiros (ver [02-mapa-e-tiles.md](02-mapa-e-tiles.md)). Especializar
  aumenta rendimento, mas concentra risco se um evento atingir poucos tiles críticos.
- O motor aloca automaticamente os postos mínimos para cobrir o consumo de comida previsto,
  depois manutenção essencial, então a prioridade da cidade. Empates seguem id estável.
  O jogador escolhe uma política: **abastecer**, **construir** ou **diversificar**; pode fixar
  até dois postos críticos. Fixar a mina conclui uma obra antes, mas pode expor a colheita.
  Esse limite de postos fixos é um **valor inicial, sujeito a balanceamento**.
- Uma fila ativa por cidade contém edifício, melhoria ou unidade. Cada projeto tem custo,
  requisito e manutenção em catálogo versionado. Materiais reservados são consumidos com o
  progresso; cancelar devolve apenas o não consumido. Postos de construção avançam a fila.
  O jogador prioriza moradia, reserva, defesa ou produção sabendo qual obra será adiada.
- Edifícios abrem postos, moradia ou proteção, sempre com manutenção. Proposta de limite:
  um edifício por função básica e cidade; melhorar substitui o nível anterior. É um
  **valor inicial, sujeito a balanceamento**. Tecnologia abre funções novas, sem bônus
  percentuais que se empilham sem teto (ver [05-tecnologia.md](05-tecnologia.md)).

### Abastecimento, crescimento e migração

- Consumo base: uma comida por ponto de população a cada turno. A reserva local cobre faltas
  antes de gerar privação; importação depende de rota e compromisso econômico (ver
  [03-economia.md](03-economia.md)). Estoque local comporta três turnos de consumo;
  excedente segue comércio ou perda do catálogo econômico. Ambos são **valores iniciais,
  sujeitos a balanceamento**. Estocar protege de seca, mas ocupa produção e manutenção.
- Crescimento requer comida disponível, moradia livre e estabilidade de pelo menos 50.
  Cada turno com reserva final de um turno de consumo dá um ponto de progresso; ao chegar
  a `max(2, população atual)`, progresso vira +1 população e zera. São **valores iniciais,
  sujeitos a balanceamento**. O custo crescente desacelera crescimento absoluto.
  O jogador decide construir moradia, ampliar comida ou manter estabilidade; nenhum deles
  isoladamente gera população.
- Falta de comida gera `D_c = min(20, 5 × unidades sem comida)`. Uma falta interrompe o
  progresso; duas faltas consecutivas retiram um ponto de população por turno até a oferta
  voltar. São **valores iniciais, sujeitos a balanceamento**. Perder pessoas reduz trabalho
  e demanda, permitindo recuperação. A privação urbana compõe `D` da pressão de crise
  definida em [00-visao.md](00-visao.md): `D = limitar(0, 20, teto(Σ(pop_c × D_c)
  / Σpop_c))`. Cidade vazia pesa zero. Assim não surge outra fórmula de crise.
- Migração interna transfere pessoas, nunca as multiplica. Se uma cidade tem privação ou
  estabilidade abaixo de 40 e outra tem vaga, comida e ligação, há opção de transferir
  até um ponto de população por turno. O grupo acompanha o migrante. O jogador escolhe
  acolher, restringir ou investir na origem; acolher preserva trabalho, mas consome a
  margem da receptora. Limiar e fluxo são **valores iniciais, sujeitos a balanceamento**.
  Uma política permanente de governo ou Mandato também pode autorizar a transferência.

### Grupos de interesse e estabilidade

- Três grupos por função: **cultivadores** pedem abastecimento e acesso à terra;
  **ofícios** pedem trabalho, obras e segurança; **mercadores** pedem rotas e previsibilidade.
  População sem posto conserva o último grupo; novos habitantes entram no grupo do primeiro
  posto. Essa abstração evita simular indivíduos, mas mantém interesses em conflito.
  Leis e crenças mudam demandas pelos dados de [06-sociedade-e-governo.md](06-sociedade-e-governo.md).
- Satisfação de cada grupo vai de 0 a 100. Base inicial 50; abastecimento completo dá +3
  por turno, privação −5, trabalho no setor favorecido +2, ausência de posto −3. Governo e
  eventos aplicam modificadores limitados; variação total por turno fica entre −10 e +10.
  São **valores iniciais, sujeitos a balanceamento**. Mudança gradual dá tempo de resposta.
- Quando um grupo cai abaixo de 40 ou pede recurso disponível, surge uma pauta. O jogador
  pode **atender** com custo imediato, **negociar** uma promessa com prazo, ou **recusar**
  para preservar recursos. Promessa cumprida melhora satisfação; descumprida deixa agravante
  e fato na Crônica. Só a pauta mais urgente aparece por cidade e turno; outras entram
  no resumo. O limiar é um **valor inicial, sujeito a balanceamento**.
- Estabilidade urbana: `S = limitar(0, 100, média ponderada da satisfação dos grupos
  + serviços − privação − pressão administrativa − dano de conflito)`. Peso é tamanho do
  grupo. Serviços dão até +10; penalidades têm teto por fonte em catálogo. Números são
  **valores iniciais, sujeitos a balanceamento**. O jogador mantém serviços, reduz
  tributos ou troca postos, com custos diferentes; estabilidade não é bônus permanente.
- `S` é local; coesão de [00-visao.md](00-visao.md) pertence à civilização. Proposta:
  a média urbana ponderada por população altera coesão em −2 abaixo de 40, +1 acima de 70,
  ou 0 entre os limiares, por turno. Promessas quebradas e secessão têm causas próprias.
  O componente `G` da pressão de crise usa a fração de pessoas em grupos com satisfação
  abaixo de 40: `G = teto(20 × pessoas desses grupos / população total)`, limitado
  a 0–20; `D` usa a falta de comida. População total zero produz `G = 0`. Tudo é
  **valor inicial, sujeito a balanceamento**. A estabilidade não é somada outra vez a `P`.

### Protesto, revolta e secessão

- Estabilidade abaixo de 40 por dois turnos inicia **protesto**. Abaixo de 25 por mais
  dois turnos, com pauta recusada ou privação persistente, habilita **revolta**.
  Acima de 50 por dois turnos reduz o estágio em um. Limiares e prazos são **valores
  iniciais, sujeitos a balanceamento**; avisos mostram prazo e causa antes da escalada.
- Protesto suspende um posto do grupo mais insatisfeito. Respostas: concessão com custo,
  mediação com prazo ou repressão com perda de coesão e risco de escalada. Revolta suspende
  mais postos e pode contestar tiles. O jogador decide qual compromisso sacrificar;
  pagar uma vez não apaga toda a cadeia de causas.
- Secessão exige em conjunto: revolta ativa por dois turnos, estabilidade abaixo de 20,
  comunidade territorial viável e pelo menos uma destas condições: ligação administrativa
  rompida ou carga acima da capacidade em 10 pontos. Esses são **valores iniciais,
  sujeitos a balanceamento**. O motor
  oferece ultimato de autonomia, reconciliação custosa ou confronto e resolve controle
  de tiles por regra. Não separa uma cidade por texto livre ou por um único sorteio.
- A comunidade sucessora herda pessoas, tiles, obrigações e fatos pertinentes do Ledger;
  a antiga conserva sua Crônica. A separação cria reivindicações diplomáticas verificáveis
  (ver [07-diplomacia.md](07-diplomacia.md) e
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- A Entropia pode propor greve, epidemia, disputa por terra ou levante somente quando um
  template validado encontra vulnerabilidade mensurável. Template fixa alcance, duração,
  custo e teto de alteração de satisfação. Ele pressiona uma escolha, sem substituir o
  contador de revolta ou a elegibilidade de secessão (ver
  [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).

### Expansão e ciclo infinito

- A expansão cria carga administrativa compartilhada:
  `carga = 3 × cidades + teto(soma das distâncias de ligação / 3)
  + max(0, cidades − 3)²`. Distâncias contam hexágonos pela ligação controlada;
  cidade desconectada usa distância territorial mínima mais 4. Capacidade proposta:
  `12 + bônus institucional`, com bônus limitado a 18. Tudo é **valor inicial, sujeito
  a balanceamento**. Excesso `max(0, carga − capacidade)` reduz estabilidade em até 20
  pontos, repartido por distância e população conforme regra do motor.
- Não há limite rígido de cidades. Além da capacidade, novas cidades podem trazer
  defesa ou recurso, mas agravam manutenção, descontentamento e secessão. O jogador
  pode melhorar ligação, investir em instituições, conceder autonomia ou aceitar
  alcance menor. Autonomia reduz carga e também tributos e controle direto, conforme
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md).
- Uma era abre opções de trabalho e administração; não multiplica automaticamente
  rendimento, população ou capacidade. Moradia, postos, estoque e manutenção limitada
  contêm expansão. Cidade madura troca produção máxima por reservas ou diversidade
  para sobreviver ao próximo choque (ver
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- No colapso, cidades não somem em bloco: preservam população sobrevivente, obras
  existentes, grupos e ligações locais. Podem integrar sucessora, ficar autônomas ou
  perder infraestrutura conforme causas registradas. No renascimento, o jogador escolhe
  comunidade sucessora e prioridade de reconstrução; reabre ligações e negocia antigas
  reivindicações. Ruínas e memória dão opções, não bônus cumulativos por ciclo.

### Governador, bots e celular

- O Governador propõe política de trabalho, obra, pauta, migração e autonomia dentro
  do Mandato. O resumo mostra causa, custo e prazo: “faltam duas comidas; colher adia
  a muralha um turno”. O motor bloqueia linha vermelha e teto de gasto. Sem IA, T0
  prioriza evitar privação, depois revolta, depois o Mandato (ver
  [09-governador-e-mandato.md](09-governador-e-mandato.md)).
- Bots diplomáticos usam fome, migração, secessão e entradas do Ledger para propor
  ajuda, comércio ou reconhecimento. Promessa de grãos só melhora satisfação após
  entrega pelas regras. O motor calcula aceitação, custos e mudança de relação;
  o bot propõe intenção justificada (ver [07-diplomacia.md](07-diplomacia.md)).
- A tela da cidade mostra comida em turnos, obra e prazo, estabilidade e grupo em risco.
  Um toque abre a causa; até três respostas comparáveis aparecem na pauta; mais um toque
  confirma. Política em lote atende cidades semelhantes com exceções visíveis.
  Alertas interrompem só por privação, revolta elegível ou decisão com prazo; o resto
  vai ao resumo (ver [11-experiencia-mobile.md](11-experiencia-mobile.md)).

### Fronteira entre regra e IA

- **Determinístico:** fundação, trabalho, produção, consumo, obra, crescimento,
  migração, satisfação, estabilidade, coesão, carga, escalada, secessão e aceitação
  diplomática. Motor usa inteiros ou ponto fixo, ordem estável e seed versionada;
  comandos e parâmetros aceitos entram no log com hash de estado por turno.
- **Proposta da IA:** intenções tipadas para cidade ou evento e justificativas ancoradas
  em ids de cidade, tiles, grupos, Mandato, Crônica e Ledger. O motor valida requisitos,
  limites e catálogos versionados antes de converter intenção em comando. Saída inválida
  ou ausente usa T0 determinístico. IA não concede população, satisfação ou território
  por texto livre, conforme ADR-0006.

## Perguntas abertas
- Nível de detalhe da população (número agregado, grupos, ou "pops" estilo Victoria)?
  - **Recomendação:** população agregada repartida em três grupos por função. Gera
    disputas políticas verificáveis sem exigir microgestão; testar os três grupos
    antes de adicionar identidades.
- Limite de cidades e como a expansão é contida a longo prazo.
  - **Recomendação:** sem limite rígido; carga crescente, capacidade limitada e
    autonomia como saída. Permite império arriscado e fragmentação jogável; validar
    em simulações longas para evitar expansão dominante ou colapso inevitável.
- Migração interna deve exigir ordem individual ou seguir política permanente?
  - **Recomendação:** permitir política permanente e pedir confirmação quando a
    mudança alterar o grupo dominante ou deixar a origem sem posto essencial.
    Reduz toques e preserva controle nas consequências políticas relevantes.
