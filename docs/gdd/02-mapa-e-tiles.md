# 02 — Mapa e tiles

## Decidido
- Grid hexagonal, coordenadas axiais/cúbicas (ADR-0004).
- Mapa gerado proceduralmente a partir de uma seed.

## Proposta

> Tudo abaixo é proposta. Números são **valor inicial, sujeito a balanceamento**. Orientação e borda
> aparecem em "Perguntas abertas"; as regras abaixo funcionam com qualquer uma das opções.

### 1. Princípio: o mapa é um sistema vivo, não um pano de fundo
O mapa tem duas partes com ritmos diferentes, e essa separação sustenta o jogo infinito:
- **Substrato (lento)**: elevação, relevo, rios principais, depósitos minerais. Muda só por eventos raros.
- **Superfície (viva)**: temperatura, umidade, bioma efetivo, fertilidade, florestas, melhorias, dono.
  Muda por clima, eventos e ação humana.

Toda a geração e toda a mudança são **determinísticas**: o mapa é função de `(seed_do_mundo, era, comandos)`,
com PRNG versionado e matemática inteira (ADR-0006). A IA nunca desenha nem altera tiles (ver seção 9).

### 2. Camadas do tile
Cada tile guarda números inteiros pequenos, para caber em snapshots leves e hashear rápido.

| Camada | Tipo | Faixa inicial | Muda por |
|---|---|---|---|
| Elevação | int | -2 (mar profundo) a 8 (pico) | Geração; eventos tectônicos raros |
| Temperatura | int | 0 a 100 | Latitude + altitude + clima da era |
| Umidade | int | 0 a 100 | Distância de água, vento, clima da era |
| Bioma efetivo | id de catálogo | derivado | Recalculado de elevação/temperatura/umidade |
| Fertilidade | int | 0 a 100 | Uso agrícola, desmatamento, eventos |
| Recurso | id + estoque | 0 ou 1 por tile | Descoberta, esgotamento, renovação |
| Feature | flags | rio, floresta, pântano, duna... | Geração; clima; ação humana |
| Melhoria | id de catálogo | 0 ou 1 | Comandos do jogador/Governador |
| Dono | id de civilização | 0 ou 1 | Fronteira da cidade, conquista, colapso |
| Visibilidade | por civilização | desconhecido / lembrado / visível | Exploração (seção 7) |

O bioma **não é sorteado**: é lido de uma tabela de catálogo `(temperatura, umidade, elevação) → bioma`.
Biomas iniciais: oceano, costa, planície, floresta, selva, savana, deserto, estepe, tundra, pântano, montanha,
geleira. Cada bioma dá rendimento-base de comida/produção/riqueza (valores em 03-economia.md) e um custo
de movimento.

### 3. Geração do mundo (determinística)
Pipeline em etapas, cada uma com sub-seed derivada da seed do mundo (`hash(seed, etapa)`):
1. **Placas e continentes**: ruído de baixa frequência define massas de terra; alvo de 30% a 40% de terra
   (valor inicial).
2. **Relevo**: ruído de alta frequência mais cadeias de montanha nas bordas de placa.
3. **Clima-base**: temperatura por latitude e altitude; umidade por proximidade de água e sombra de montanha.
4. **Rios**: nascem em tiles de umidade alta e elevação alta, escorrem pelo vizinho mais baixo
   (desempate por ordem canônica de índice, nunca por hash map) até o mar ou um lago. Rios correm
   **nas arestas** entre hexágonos, não sobre eles: dão bônus de comida aos dois tiles, custo extra
   para atravessar e tornam-se corredores de comércio.
5. **Biomas e recursos**: aplicação da tabela e posicionamento de recursos (seção 5).
6. **Pontos de partida**: escolhidos pelo motor com pontuação de justiça (seção 6).
7. **Validação**: se o mapa falhar em conectividade ou equilíbrio mínimo, o motor incrementa uma sub-seed
   de tentativa e regenera (limite de tentativas; a seed final e a tentativa ficam no log).

O que o jogador decide ao criar o mundo: tamanho, abundância de terra e "personalidade climática"
(estável, sazonal, instável). Decisão interessante porque define o tipo de jogo (expansão vs. adaptação)
com um toque, sem expor parâmetros de ruído.

### 4. Clima e variação por era
O clima do mundo é um pequeno vetor global por era: `deslocamento_de_temperatura` e `deslocamento_de_umidade`
(valor inicial: cada um entre -15 e +15), mais um fator regional de monção/seca por faixa de latitude.

- **Regra do motor**: ao virar a era (ver 10-ciclo-infinito-e-eras.md), o vetor é atualizado dentro de um
  passo máximo (valor inicial: ±5 por era, cumulativo com teto ±15). Os tiles têm o bioma efetivo
  recalculado de forma gradual: no máximo 5% dos tiles mudam de bioma por turno (valor inicial), começando
  pelos de fronteira climática. Isso dá tempo de reagir.
- **Previsibilidade**: o vetor da próxima era é um fato do estado; o jogador e o Governador veem uma
  **tendência** ("esfriando", "secando") com alguns turnos de antecedência. O jogador decide se migra,
  irriga, estoca comida ou ignora. Sem aviso, clima seria só azar; com aviso, é planejamento.
- **Eventos climáticos** (seca, cheia, glaciação, onda de calor) são templates da Entropia
  (ver 08-entropia-e-eventos.md). Eles não editam tiles: aplicam um **modificador temporário** de
  temperatura/umidade em uma região, e o motor recalcula bioma e fertilidade pelas regras acima.
  Exemplo de template: `seca_regional`, raio 3 a 6 hexes, umidade -20 a -35, duração 8 a 20 turnos
  (valores iniciais), com cadeia possível para fome e migração.
- **Mudança permanente** (um deserto que avança, rio que seca) só ocorre quando o modificador persiste
  por tempo suficiente ou na virada de era; nunca por um único evento.

### 5. Recursos
Três classes, para que cada uma crie uma decisão diferente:
- **Renováveis** (gado, peixe, madeira, cultivos): rendimento cai se a pressão de uso passar do limite do
  tile e se recupera sozinho. Decisão: explorar ao máximo agora ou manter sustentável.
- **Finitos** (minérios, pedra, carvão): cada depósito tem estoque (valor inicial: 100 a 400 "cargas");
  a melhoria extrai por turno e o tile esgota. Decisão: onde investir sabendo que o depósito acaba.
- **Estratégicos/de luxo raros**: poucos por mapa (valor inicial: 1 a 2 por 40 tiles de terra), distribuídos
  de modo que nenhuma civilização tenha todos. Motivam comércio e disputa (ver 03-economia.md e
  07-diplomacia.md).

Descoberta: depósitos finitos nascem **ocultos** e são revelados por tecnologia, exploração ou evento
(ver 05-tecnologia.md). Assim o mapa continua entregando novidade a cada era sem criar tiles novos.
**Renovação para o longo prazo**: ao virar a era, uma pequena fração dos tiles de substrato sorteados pela
seed (valor inicial: 1% a 2%) ganha depósito novo oculto, e depósitos esgotados em terra abandonada
voltam lentamente a ser "regiões prospectáveis". O mapa nunca fica seco por completo.

### 6. Tamanho do mapa, bordas e pontos de partida
- **Tamanho por civilizações** (valor inicial): `tiles_de_terra ≈ civs × 70`, e `tiles_totais ≈ tiles_de_terra / 0,35`.
  Exemplos: 4 civs ≈ 800 tiles totais (~30×27); 8 civs ≈ 1.600 (~40×40); 16 civs ≈ 3.200 (~57×57).
  Piso de 600 tiles totais (mundo muito pequeno ainda tem espaço para 3 cidades por civilização).
- **Mundo com folga**: reservar 25% da terra como "terra de ninguém" no início, para expansão e para
  receber novas civilizações (renascimento, novos jogadores; ver 10-ciclo-infinito-e-eras.md).
- **Partida justa**: cada ponto de partida é pontuado pelo motor (comida, produção, água doce, recurso
  estratégico num raio de 3); diferença máxima entre civilizações de 15% (valor inicial). Distância
  mínima entre capitais: 8 hexes (valor inicial).
- **Mapa fixo por mundo, regiões fluidas**: o tamanho não muda depois de criado. O que cresce é o uso, não o
  espaço (seção 8).
- **Custo no celular**: o servidor envia só os tiles conhecidos pela civilização, em pedaços por região (chunks
  de 8×8 hexes) e em diferenças por turno; o cliente nunca precisa do mapa inteiro.

### 7. Névoa de guerra e exploração
Três estados por civilização e tile: **desconhecido**, **lembrado** (visto antes; mostra a última informação,
marcada como antiga) e **visível**.
- **Visão**: unidades e cidades têm raio de visão (valor inicial: 2 hexes; +1 em terreno alto, -1 em
  floresta/selva). A visão é calculada pelo motor sem linha de visão complexa: só custo de terreno
  e elevação, o que é barato e determinístico.
- **Exploração como investimento**: a primeira descoberta de um tile da terra rende pouco, mas
  **maravilhas naturais, rios longos e depósitos revelados** dão conhecimento/cultura uma vez
  (valores em 03-economia.md). Decisão do jogador: mandar o explorador agora ou guardar a unidade.
- **Ordem de exploração em um toque**: "Explorar automaticamente" cria uma intenção persistente que o motor
  executa por regras (vai ao tile desconhecido mais próximo, evita perigo conhecido). Sem LLM.
- **Governador**: ele só enxerga o que a civilização enxerga (mesma informação do jogador). Não há
  trapaça por informação perfeita. O Mandato (ver 09-governador-e-mandato.md) tem uma prioridade
  "explorar" e uma proibição possível ("não entrar em território de X sem tratado"). O Governador decide
  entre explorar e economizar unidades dentro desses limites.
- **Diplomacia**: o primeiro contato (estado "contato" em 07-diplomacia.md) acontece quando duas
  civilizações têm tiles visíveis um do outro. Trocar mapas é uma moeda diplomática: compartilhar
  tiles lembrados vira entrada no Ledger de Relações ("ajuda") e é validado pelo motor. Bots só podem
  citar fatos geográficos que sua civilização conhece, o que impede que a IA invente fronteiras (grounding).
- **Mundo antigo**: tiles "lembrados" ficam obsoletos e o cliente os marca assim; ruínas e cidades
  perdidas mudam o que o jogador encontra ao revisitar.

### 8. Como o mapa sustenta décadas de jogo infinito
Riscos de um mapa fixo em jogo infinito: lotar, esgotar e ficar estático. Respostas:
1. **Custo crescente de ocupar terra**: manutenção das melhorias aumenta com a quantidade de tiles
   possuídos acima de um limiar (valor inicial: +2% por tile acima de 25 por cidade); ver 03-economia.md
   e 04-cidades-e-populacao.md. A expansão tem retorno decrescente, sem teto artificial.
2. **Degradação do solo**: fertilidade cai com uso intensivo contínuo (valor inicial: -1 a cada 10 turnos
   em tile no máximo de uso) e se recupera em pousio. O jogador alterna entre produzir e conservar;
   o Governador segue a prioridade do Mandato.
3. **Era como virada de mapa**: a cada era o clima desloca biomas, abre fronteiras produtivas (tundra que
   degela) e fecha outras (planície que seca). O melhor lugar para morar muda. Isso reavalia a disputa
   territorial sem criar tiles novos.
4. **Colapso libera território**: quando uma civilização colapsa, seus tiles viram ruínas e **região
   abandonada**: o dono é removido, melhorias degradam em prazo fixo (valor inicial: 50 turnos para
   virar ruína) e a floresta/vegetação avança. Abre espaço e um motivo de exploração (ruínas com
   bônus de Legado, ver 10-ciclo-infinito-e-eras.md).
5. **Renascimento**: uma civilização renascida escolhe ponto de partida entre as regiões abandonadas
   mais justas, não numa cópia do início. O mapa é palimpsesto: camadas de eras anteriores permanecem
   como ruínas, nomes e Crônica.
6. **Limite de dados do mapa**: tiles guardam inteiros fixos; registros históricos por tile são
   compactados por região na virada de era (resumo na Crônica, sem lista infinita de eventos).

### 9. ADR-0006: o que é determinístico e o que a IA propõe
| Determinístico (motor/catálogo) | A IA propõe (intenção tipada, validada) |
|---|---|
| Geração, rios, biomas, recursos, pontos de partida | Nomes de regiões, rios e marcos; descrições na Crônica |
| Atualização do vetor climático por era e recálculo de biomas | Escolher qual template climático a Entropia dispara e onde (dentro do orçamento de tensão) |
| Visão, névoa e custos de movimento | Governador: para onde explorar, onde fundar, que região proteger |
| Esgotamento, renovação, degradação do solo | Bots: pedir troca de mapas, recusar acesso a região, alegar disputa de fronteira |
| Aceitar ou recusar qualquer comando | Narrativa de eventos com base em fatos do estado |

Toda intenção cita ids de tiles/regiões/eventos existentes. Se não validar, cai no T0 (heurística):
por exemplo, explorar o tile desconhecido mais próximo.

### 10. Interação com outros pilares (resumo)
- **08-entropia-e-eventos.md**: clima, desastres e depósitos ocultos são os ganchos; a Entropia lê o mapa
  (fertilidade, vizinhança de fronteira) para justificar o evento, e a regra de justiça impede
  catástrofes em série sobre a mesma região.
- **07-diplomacia.md**: fronteiras, rios compartilhados, recursos disputados e troca de mapas alimentam o
  Ledger; um rio que seca a montante pode ser incidente rastreável.
- **03-economia.md** e **04-cidades-e-populacao.md**: rendimentos por tile, manutenção por território,
  migração e secessão em regiões que deixam de render.
- **11-experiencia-mobile.md**: camadas visuais (fertilidade, clima, donos) alternadas por um toque; toque
  longo no tile mostra a explicação ("por que rende pouco?").

## Perguntas abertas
- Orientação (pointy-top ou flat-top) e borda (wrap horizontal, cilindro, ilha fechada).
  Recomendação: flat-top ou pointy-top é questão de UI; sugiro **pointy-top** (linhas horizontais
  de hexágonos combinam com rolagem vertical e com polegar em retrato) e **cilindro** (wrap
  horizontal, polos fechados). O cilindro elimina bordas laterais, mantém clima por latitude coerente e
  evita o "canto seguro". Mundo pequeno com ilha fechada é opção por configuração do mundo.
- Tamanho do mapa por número de civilizações.
  Recomendação: usar a fórmula da seção 6 (~70 tiles de terra por civilização, piso de 600 tiles) para
  o MVP, medir tempo de turno e tamanho de payload no celular e só então ajustar. Começar menor é
  mais fácil de ampliar do que reduzir.
- Névoa de guerra e exploração: como funcionam para o Governador?
  Recomendação: o Governador usa exatamente a visibilidade da civilização (sem informação extra), com
  prioridade "explorar" no Mandato. Isso mantém o jogo justo, a explicação do relatório honesta e
  reduz contexto de IA (só tiles conhecidos entram no prompt).
- Novo: o clima deve ser **escolhido pelo jogador na criação do mundo** (estável/sazonal/instável),
  ou é sempre decidido pela personalidade da Entropia (ver 08-entropia-e-eventos.md)?
  Recomendação: ligar os dois. A personalidade da Entropia define o padrão e o jogador pode
  sobrescrever na criação, porque clima instável muda muito o estilo de jogo e deve ser consentido.
- Novo: o mapa pode ter **mudanças permanentes de relevo** (vulcão cria montanha, mar sobe)?
  Recomendação: sim, mas raras (no máximo 1 por era no mundo, via template aprovado), pois dão
  memória e drama ao mapa sem arriscar invalidar cidades de forma injusta.