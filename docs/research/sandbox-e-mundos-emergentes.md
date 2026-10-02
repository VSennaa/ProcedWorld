# Estudo: jogos sandbox e mundos emergentes

> Pesquisa para design e engenharia do ProcedWorld.  
> Escopo: história derivada de estado, direção de eventos, leitura de causa, colapso
> e continuidade. As conclusões de design abaixo são propostas; não mudam decisões de GDD/SDD.

## Lente de comparação

Um sandbox durável não precisa de uma condição de vitória para manter direção.
Ele precisa preservar consequências, apresentar necessidades novas e permitir que o jogador
atribua cada consequência a fatos anteriores.
O padrão recorrente nos casos estudados é: simulação ou conteúdo coerente gera fatos;
uma camada de apresentação os transforma em história legível.
Isso é compatível com o princípio do ProcedWorld de que a Crônica é derivada do estado,
e não sua fonte de verdade (`docs/gdd/10-ciclo-infinito-e-eras.md`; `docs/sdd/09-persistencia.md`).

## Dwarf Fortress

### O que é

*Dwarf Fortress* gera um mundo antes do jogo e registra história de civilizações, sítios,
populações e acontecimentos; a história continua, ainda que mais lentamente, durante o jogo [F1].
O gerador combina geografia, ecologia e uma simulação histórica, em vez de preparar só um cenário [F1].

### Como funciona o mecanismo relevante

A geração histórica acompanha agentes e entidades ao longo de anos simulados [F1].
O relato do criador descreve o resultado como um jogo estratégico sem jogador, com muitos agentes,
do qual a história é o registro [F1].
Assim, uma fortaleza chega a um mundo que já possui lugares, povos e passados causalmente ligados.
A continuidade após a geração impede que o mundo seja apenas uma introdução estática [F1].

### O que aproveitar no ProcedWorld

- Tratar a Crônica como projeção consultável de eventos canônicos, não como prosa inventada.
- Manter entidades históricas estáveis: cidade, dinastia, ruína, tratado, tecnologia e personagem.
- Fazer geografia, recursos e política deixarem vestígios que sobrevivem a uma era.
- Oferecer uma visão resumida da cadeia: fato, participantes, lugar, turno e consequência.

### O que evitar

- Não tentar simular milhares de indivíduos se isso não altera decisões do jogador.
- Não esconder a causalidade atrás de uma enciclopédia extensa que o celular não comporta.
- Não confundir volume de eventos históricos com relevância dramática.

### Pilar/SDD afetado

`docs/gdd/02-mapa-e-tiles.md`, `docs/gdd/10-ciclo-infinito-e-eras.md`,
`docs/sdd/01-nucleo-simulacao.md`, `docs/sdd/09-persistencia.md` e
`docs/sdd/16-visibilidade.md`.

## Caves of Qud

### O que é

*Caves of Qud* mescla locais autorais com regiões, facções, sítios e história procedural [F2].
O sistema concentra histórias em cinco governantes antigos, os sultões, e as torna descobrível
por textos, artefatos e lugares do mundo [F3].

### Como funciona o mecanismo relevante

As biografias míticas dos sultões são geradas dentro de uma estrutura limitada [F3].
O projeto trata relatos históricos como perspectivas limitadas e potencialmente divergentes,
veiculadas por artefatos e textos diegéticos [F3].
O conteúdo histórico conecta nomes, objetos, facções e sítios em vez de existir como texto solto [F2].
A combinação de âncoras autorais com variação procedural reduz a sensação de aleatoriedade total [F2].

### O que aproveitar no ProcedWorld

- Reservar poucas figuras, locais ou instituições memoráveis por era, com IDs persistentes.
- Derivar artefatos, ruínas e nomes de fatos do log, para a Crônica apontar para estado verificável.
- Permitir perspectivas: uma Crônica pública pode registrar alegação; o ledger preserva o fato.
- Usar templates narrativos que exigem campos de grounding antes de serem exibidos.

### O que evitar

- Não gerar lore que não toca mapa, diplomacia, tecnologia ou decisão futura.
- Não apresentar boato, opinião diplomática e fato mecânico como se fossem equivalentes.
- Não deixar o LLM criar genealogias ou causalidades mecânicas fora do catálogo.

### Pilar/SDD afetado

`docs/gdd/05-tecnologia.md`, `docs/gdd/07-diplomacia.md`,
`docs/gdd/10-ciclo-infinito-e-eras.md`, `docs/sdd/08-memoria-e-contexto.md` e
`docs/sdd/18-dsl-catalogos.md`.

## RimWorld

### O que é

*RimWorld* usa storytellers que disparam eventos para moldar o ritmo da colônia [F4].
Os três presets são Cassandra Classic, Phoebe Chillax e Randy Random [F4].

### Como funciona o mecanismo relevante

Cassandra alterna escalada de desafio e pausa; Phoebe aumenta o intervalo entre crises [F4].
Randy privilegia aleatoriedade e pode criar sequências severas ou injustas [F4].
O storyteller considera, entre outros fatores, riqueza, população, perdas recentes e tempo desde
evento grande [F4].
O efeito prático é uma curva de tensão que converte o estado atual em ocasiões de decisão.

### O que aproveitar no ProcedWorld

- Manter o orçamento de tensão e janelas de leitura, crise e respiro já propostos para a Entropia.
- Selecionar apenas templates cuja causa, alvo e resposta útil estejam presentes no estado.
- Medir exposição repetida, recuperação e tempo desde a última crise por civilização.
- Usar personalidades como pesos entre eventos válidos, nunca como regra que ignora justiça.

### O que evitar

- Não usar aleatoriedade como substituta para distribuição de justiça entre civilizações.
- Não escalar catástrofe diretamente por “poder” sem mostrar a vulnerabilidade que a habilitou.
- Não copiar o caráter imprevisível de Randy para eventos que possam causar colapso inevitável.

### Pilar/SDD afetado

`docs/gdd/08-entropia-e-eventos.md`, `docs/gdd/04-cidades-e-populacao.md`,
`docs/sdd/06-entropia.md`, `docs/sdd/01-nucleo-simulacao.md` e
`docs/sdd/14-testes.md`.

## WorldBox

### O que é

*WorldBox* se apresenta como sandbox de deus em PC, mobile e outras plataformas [F5].
Sua descrição oficial enfatiza observar civilizações progredirem e interagirem, ou intervir
com poderes destrutivos [F5].

### Como funciona o mecanismo relevante

A página oficial descreve a observação de crescimento, reinos, colonização, guerra, queda de
impérios e rebelião urbana [F6].
O jogador alterna entre semear condições e assistir os resultados sistêmicos [F5].
Detalhes de algoritmo, regras de sucessão e critérios de justiça não foram verificados em fonte
técnica primária; não devem orientar uma especificação do motor.

### O que aproveitar no ProcedWorld

- Exibir um modo “observar o mundo” com resumos curtos de mudanças entre turnos.
- Fazer a intervenção rara na Entropia ser uma escolha explícita e cara, não um botão milagroso.
- Priorizar visualização de fronteira, cidade, crise e consequência sobre textos longos.

### O que evitar

- Não deslocar a agência estratégica para poderes externos que desfazem qualquer decisão.
- Não adotar eventos apenas espetaculares se não deixam estado, custo e contrajogo legíveis.
- Não inferir regras internas de WorldBox além do que foi verificado acima.

### Pilar/SDD afetado

`docs/gdd/08-entropia-e-eventos.md`, `docs/gdd/11-experiencia-mobile.md`,
`docs/sdd/06-entropia.md` e `docs/sdd/12-cliente.md`.

## Songs of Syx

### O que é

*Songs of Syx* é um simulador de cidade-estado fantástico que combina construção e estratégia [F7].
O site afirma simular indivíduos e delegar tarefas mundanas à IA enquanto o jogador decide em
escala mais ampla [F7].

### Como funciona o mecanismo relevante

A população inclui classes como nobres, cidadãos, crianças e escravizados; cidadãos também têm
subclasses funcionais [F8].
A interface de população expõe idade, mudanças de população, imigração e satisfação [F9].
Essa leitura em camadas liga agregado estratégico a causas humanas compreensíveis.
As especificidades de suas regras econômicas e de colapso além disso: não verificado.

### O que aproveitar no ProcedWorld

- Mostrar no cartão de cidade os fatores que compõem estabilidade, privação e crescimento.
- Separar decisão de alto nível do trabalho automático do Governador, limitado pelo Mandato.
- Produzir explicações agregadas que preservem as entidades causais, sem simular UI por indivíduo.

### O que evitar

- Não usar população como número opaco que cresce até o desempenho cair.
- Não delegar sem relatório: o jogador deve ver o que o Governador fez e por qual restrição.
- Não copiar escala de simulação sem medir orçamento de turno e memória.

### Pilar/SDD afetado

`docs/gdd/04-cidades-e-populacao.md`, `docs/gdd/09-governador-e-mandato.md`,
`docs/sdd/05-governador.md`, `docs/sdd/15-regras-de-dominio.md` e
`docs/sdd/12-cliente.md`.

## Crusader Kings III

### O que é

*Crusader Kings III* é uma estratégia centrada em personagens e dinastias medievais [F10].
Ao morrer um personagem, o jogador segue seu herdeiro dinástico, preservando continuidade entre
gerações [F10].

### Como funciona o mecanismo relevante

Escolhas contrárias a traços aumentam estresse, vinculando ação a personalidade [F11].
Uma atualização introduziu memórias visíveis no visualizador do personagem [F12].
O sistema torna relações especiais e experiências passadas inspecionáveis, em vez de só aplicar
modificadores invisíveis [F12].
Sucessão muda o agente jogado, mas não apaga a posição histórica da dinastia [F10].

### O que aproveitar no ProcedWorld

- Modelar legado como continuidade parcial: instituição, prática ou memória sobrevive ao colapso.
- Explicar ação diplomática por entradas nomeadas no Ledger de Relações, não por um score só.
- Exibir “por que isto mudou” em cada relação, cidade e crise, com links à Crônica.
- Tratar perda como mudança de perspectiva e objetivo, não como tela final obrigatória.

### O que evitar

- Não criar atributos psicológicos profundos antes de haver contrajogo e explicação adequados.
- Não transformar relações em uma lista longa de modificadores sem ordem, duração ou evidência.
- Não prometer personagens individuais simulados na primeira fase sem orçamento técnico.

### Pilar/SDD afetado

`docs/gdd/06-sociedade-e-governo.md`, `docs/gdd/07-diplomacia.md`,
`docs/gdd/10-ciclo-infinito-e-eras.md`, `docs/sdd/07-diplomacia.md` e
`docs/sdd/08-memoria-e-contexto.md`.

## Ultima Ratio Regum

### O que é

*Ultima Ratio Regum* é um roguelike voltado à geração procedural de cultura [F13].
O jogador investiga uma conspiração por pistas obtidas em culturas, textos, locais e histórias
reais do mundo gerado [F14].

### Como funciona o mecanismo relevante

O projeto descreve cadeias de significado: geradores posteriores leem a saída de geradores
anteriores [F14].
História, religião, sociedade e linguagem alimentam pistas e mistérios, em vez de serem
camadas independentes [F14].
Essa dependência transforma conhecer o mundo em instrumento de decisão.
Detalhes atuais da implementação interna: não verificados.

### O que aproveitar no ProcedWorld

- Fazer cada gerador consumir referências canônicas anteriores: bioma → recurso → cidade → crença.
- Exigir que evento e texto indiquem quais fatos do estado os sustentam.
- Projetar a Crônica para responder perguntas do jogador, não somente registrar acontecimentos.
- Criar descobertas emergentes como consequência de necessidade local e não de uma rolagem isolada.

### O que evitar

- Não permitir que conteúdo gerado use nomes ou símbolos sem origem no estado.
- Não construir uma camada cultural que não afete economia, diplomacia, tecnologia ou mapa.
- Não tornar interpretação textual o único caminho para entender regra mecânica.

### Pilar/SDD afetado

`docs/gdd/02-mapa-e-tiles.md`, `docs/gdd/05-tecnologia.md`,
`docs/gdd/06-sociedade-e-governo.md`, `docs/sdd/08-memoria-e-contexto.md` e
`docs/sdd/18-dsl-catalogos.md`.

## Kenshi

### O que é

*Kenshi* é um sandbox de sobrevivência, esquadrão, construção e comércio em mundo reativo [F15].
Não depende de enredo linear: o jogador é um agente entre facções e perigos do mundo [F16].

### Como funciona o mecanismo relevante

Relações de facção possuem limiares explícitos para hostilidade e aliança [F15].
Segundo a documentação comunitária, cidades podem trocar de controle, ser destruídas ou
reconstruídas por ação do jogador ou NPC, e líderes ou facções podem cair [F15].
O mundo não precisa proteger o jogador como protagonista para produzir narrativas de sobrevivência.
Detalhes dos gatilhos internos de mudança de mundo: não verificados.

### O que aproveitar no ProcedWorld

- Preservar mundo ativo para bots e jogador ausente por meio do Governador e turnos simultâneos.
- Permitir que cidades e tratados mudem de estado sem encerrar a campanha.
- Expor limiares diplomáticos, obrigações e fatos que explicam amizade, tensão e guerra.
- Converter derrota local em problema de reconstrução, sucessão ou migração com escolhas.

### O que evitar

- Não usar opacidade total como sinônimo de mundo indiferente.
- Não deixar a queda de uma cidade apagar seus vestígios, dívidas, tecnologias e relações.
- Não dar a NPCs mudança política sem comando, regra e registro reproduzível.

### Pilar/SDD afetado

`docs/gdd/01-loop-e-turnos.md`, `docs/gdd/07-diplomacia.md`,
`docs/gdd/10-ciclo-infinito-e-eras.md`, `docs/sdd/03-turnos-e-sessao.md` e
`docs/sdd/07-diplomacia.md`.

## Recomendações para o ProcedWorld

1. Definir um `ChronicleEvent` canônico e derivado do log: IDs de participantes, lugar, turno,
   template/revisão, causas citadas, efeito aplicado e texto de apresentação separado.
2. Acrescentar ao contrato da Entropia um teste de explicabilidade: todo template escolhido deve
   retornar causa observável, resposta útil e proteção/cooldown aplicável antes de virar comando.
3. Implementar uma tela curta “desde seu último turno” que agrupe fatos por cidade, civilização e
   crise; cada item deve abrir a cadeia causal, apropriada a `docs/gdd/11-experiencia-mobile.md`.
4. Fixar poucos legados após colapso — instituição, prática e registro histórico — e criar
   redescoberta baseada em ruína, território ou sobrevivente, conforme `gdd/05` e `gdd/10`.
5. Usar personalidade da Entropia somente para ponderar templates já elegíveis e justos;
   registrar no evento a ponderação aplicada para auditoria e replay.
6. Tornar o Ledger de Relações uma linha do tempo de promessas, custos, violações e reparações,
   com decaimento explícito; o score agregado nunca é a única explicação disponível.
7. Para cada descoberta emergente, armazenar a necessidade local e o template tecnológico que a
   originou; narrativa livre só nomeia e descreve o candidato validado.
8. Tratar agregados de população como explicação em camadas: valor total → grupos relevantes →
   fatores de mudança; não simular indivíduos sem uma decisão jogável que dependa deles.
9. Criar fixtures de “histórias longas” que provem replay, rastreabilidade e continuidade após
   queda, migração, paz e reconstrução, além do hash de estado.

## Fontes

- **[F1] Dwarf Fortress Wiki — World generation.** História acompanha civilizações, sítios,
  populações e eventos; inclui o relato do criador sobre a simulação histórica.  
  https://new.dwarffortresswiki.org/index.php/World_generation
- **[F2] Freehold Games — roadmap de Caves of Qud.** Escopo de locais, cultos e sítios históricos.
  https://cavesofqud.com/roadmap/
- **[F3] Grinblat, Jason — *Subverting Historical Cause & Effect*.** Paper do desenvolvedor sobre
  biografias dos sultões e relatos perspectivados.  
  https://www.freeholdgames.com/papers/Generation_of_Mythic_Biographies_in_CavesofQud.pdf
- **[F4] RimWorld Wiki — AI Storytellers.** Presets, curva de Cassandra/Phoebe/Randy e fatores de
  eventos.  
  https://rimworldwiki.com/wiki/AI_Storytellers
- **[F5] WorldBox — site oficial.** Proposta de sandbox de deus e observação/intervenção.
  https://www.superworldbox.com/
- **[F6] Steam — página de WorldBox em PT-BR.** Crescimento de civilizações, reinos, rebeliões e
  queda de impérios na descrição do produto.  
  https://store.steampowered.com/app/1206560/WorldBox__God_Simulator/?l=brazilian
- **[F7] Songs of Syx — site oficial.** Escopo do simulador e delegação de tarefas à IA.
  https://www.songsofsyx.com/
- **[F8] Songs of Syx Wiki — Subjects.** Classes de população e atividades.
  https://songsofsyx.com/wiki/index.php/Subjects
- **[F9] Songs of Syx Wiki — Fulfillment.** Informações expostas na interface de população.
  https://www.songsofsyx.com/wiki/index.php/Fulfillment
- **[F10] White Rose Research — *Historical Emulation: How Strategy Games Teach History*.**
  Personagem, dinastia e sucessão em CK3.  
  https://eprints.whiterose.ac.uk/id/eprint/237296/1/Historical%20Emulation_%20How%20Strategy%20Games%20Teach%20History.pdf
- **[F11] Crusader Kings III — referência de mecânicas.** Relação entre traços e estresse.
  https://en.wikipedia.org/wiki/Crusader_Kings_III
- **[F12] Steam Community — diário de desenvolvimento de CK3 1.7.** Memórias e visualizador.
  https://store.steampowered.com/news/posts/?appids=1158310&enddate=1662033633&feed=steam_community_announcements
- **[F13] Johnson, Mark R. — *Procedural Generation of Linguistics*.** Contexto e foco cultural de
  *Ultima Ratio Regum*.  
  https://www.pcgworkshop.com/archive/johnson2016procedural.pdf
- **[F14] Game Developer — entrevista sobre *Ultima Ratio Regum*.** Cadeias de significado e pistas.
  https://www.gamedeveloper.com/design/the-10-year-journey-of-ultima-ratio-regum-the-culture-generating-roguelike
- **[F15] Kenshi.zone — Game Mechanics.** Relações de facção e alterações do mundo.
  https://kenshi.zone/en/mechanics
- **[F16] Lo-Fi Games/Steam — descrição de *Kenshi*.** Sandbox aberto, esquadrão e mundo sem linearidade.
  https://store.steampowered.com/app/233860/Kenshi/
