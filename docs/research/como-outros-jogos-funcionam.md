# Estudo — Como outros jogos de estratégia funcionam

> Data da pesquisa: 2026-10-01. Escopo: lições de design e engenharia para o ProcedWorld.
> Este é um estudo comparativo, não uma especificação nova. “Aproveitar” e “evitar” são
> recomendações derivadas das fontes e devem passar pela revisão normal de GDD/SDD.

## Critérios de leitura

O ProcedWorld já decidiu servidor autoritativo, motor determinístico, event sourcing e turnos
simultâneos sem relógio (ADR-0001, 0003, 0006 e 0008). Portanto, a comparação procura padrões
que preservem essas decisões, e não substitutos para elas.

Os jogos comerciais mudam com patches e expansões. Uma afirmação abaixo descreve somente o que a
fonte vinculada declara; quando ela não permite concluir o detalhe técnico, o texto marca
**não verificado**. As relações com arquivos internos são de impacto, não uma autorização para
alterá-los.

## Civilization VI e Civilization VII

### O que é

Civilization VI é um 4X histórico por turnos; seu manual oficial diferencia o jogo solo por
turnos do multiplayer em turnos simultâneos. [F1]

Civilization VII organiza uma partida em três Ages jogadas conjuntamente por todos e separa a
escolha persistente de líder da civilização escolhida para cada Age. [F2]

### Como funciona o mecanismo relevante

- Em Civ VI, o modo simultâneo permite que os participantes façam seu turno no mesmo intervalo;
  a fonte consultada não descreve publicamente a regra de arbitragem de ações conflitantes.
  **Não verificado** se há ordem estável equivalente à ordem de aceitação do ProcedWorld. [F1]
- Civ VI usa líderes com agendas históricas na diplomacia, e uma entrevista de desenvolvimento
  descreve a IA como baseada em pesos (“flavors”) capazes de encadear objetivos. [F3]
- Em Civ VII, cada Age é um capítulo; no início de uma nova Age o jogador escolhe uma civilização
  exclusiva dela, enquanto o líder permanece. Elementos anteriores podem carregar conforme
  civilização e sucessos. [F2]
- A documentação de suporte também registra mudança de manutenção de relacionamento: relações
  passam a retornar gradualmente a zero, em vez de saltar na transição de Age. [F4]
- A fonte oficial consultada não documenta o algoritmo de IA nem o modelo de persistência/replay.
  **Não verificado.**

### Aproveitar no ProcedWorld

- Manter a resolução explícita do ADR-0003, em vez de copiar uma simultaneidade cuja arbitragem
  não está documentada: cada comando precisa de sequência e causa de aceitação/rejeição.
- Usar eras como ponto de reavaliação de metas e de contexto, mas preservar legado identificável;
  a transição por capítulos de Civ VII é uma boa referência de ritmo, não de reset obrigatório.
- Aplicar decaimento gradual e auditável a relações, como inspiração para o Ledger, sem apagar
  dívidas, promessas e ofensas que ainda tenham efeito mecânico.
- Representar preferências de bot por vetores/utility e fatos de estado, não por texto livre;
  agendas podem virar pesos declarados e justificáveis.

### Evitar no ProcedWorld

- Não deixar disputa simultânea depender de velocidade de rede, ordem incidental ou interface;
  isso contrariaria o motor reprodutível e tornaria o resultado difícil de explicar.
- Não tratar toda virada de era como apagamento de relações e instituições; o ciclo infinito pede
  continuidade, colapso parcial e legado, não uma campanha segmentada.
- Não expor “personalidade” diplomática como caixa-preta: o Ledger deve mostrar o fato que alterou
  a avaliação, mesmo que o texto narrativo tenha voz própria.

### Pilar/SDD afetado

`docs/gdd/01-loop-e-turnos.md`, `07-diplomacia.md`, `10-ciclo-infinito-e-eras.md` e
`09-governador-e-mandato.md`; `docs/sdd/03-turnos-e-sessao.md`, `04-camadas-de-ia.md`,
`07-diplomacia.md`, `08-memoria-e-contexto.md` e `09-persistencia.md`.

## Old World

### O que é

Old World é um 4X histórico da Mohawk Games em que cada turno representa um ano e os governantes
são mortais, segundo a página do estúdio. [F5]

O jogo combina personagens, famílias, ambições, legitimidade, ordens e eventos dinâmicos para
produzir uma história procedimental da nação. [F5]

### Como funciona o mecanismo relevante

- A página oficial diz que completar ambições, maravilhas e feitos aumenta Legitimidade; ela
  concede mais Orders por ano e melhora a posição perante a população. [F5]
- A wiki oficial detalha que Legitimidade pode vir de ambições, cognomes e eventos; cada ponto
  concede opinião de família e fração de Order por turno. [F6]
- A fonte oficial descreve eventos dinâmicos derivados de decisões, conquistas e personagens;
  ambições são geradas a partir de eventos e desejos dos personagens. [F5]
- O manual oficial lista “Simultaneous Turns” como uma opção de multiplayer. A regra de conflito,
  o servidor e a persistência assíncrona não são detalhados na evidência consultada.
  **Não verificado.** [F7]

### Aproveitar no ProcedWorld

- Converter história social em capacidade operacional limitada: coesão, legitimidade ou confiança
  podem afetar prioridades/ações disponíveis, desde que o efeito fique explícito no motor.
- Encadear evento → escolha → consequência → memória/Crônica, usando IDs de causa e parâmetros
  de template. O evento deve consultar estado, nunca inventar efeito por narrativa.
- Fazer ambições de personagens, cidades ou facções criarem objetivos situacionais para bots e
  Governador; a intenção deve citar a ambição e o estado que a tornou relevante.

### Evitar no ProcedWorld

- Não acoplar uma métrica social a bônus opacos ou a ganhos cumulativos sem contrapeso; isso
  comprometeria a economia de longo prazo e a explicabilidade.
- Não usar eventos encadeados que só parecem causais: cada elo precisa de pré-condição, escolha,
  prazo em turnos e consequência registrada.
- Não copiar a mortalidade dinástica como obrigação de tema; o valor transferível é a ponte entre
  indivíduo, instituição e sistema, não a ambientação antiga.

### Pilar/SDD afetado

`docs/gdd/04-cidades-e-populacao.md`, `06-sociedade-e-governo.md`,
`08-entropia-e-eventos.md` e `10-ciclo-infinito-e-eras.md`; `docs/sdd/05-governador.md`,
`06-entropia.md`, `08-memoria-e-contexto.md` e `18-dsl-catalogos.md`.

## Stellaris

### O que é

Stellaris é estratégia espacial da Paradox; suas expansões oficiais descrevem diplomacia por
federações, comunidade galáctica, espionagem e crises de meio/fim de jogo. [F8] [F9]

A atualização 4.0 também menciona revisão de sistemas de população e comércio para desempenho
em galáxias grandes e tardias. [F10]

### Como funciona o mecanismo relevante

- Federations apresenta coesão interna de federações, recompensas aos membros e uma comunidade
  galáctica que vota resoluções, aplica sanções e define foco comum. [F8]
- Nemesis acrescenta espionagem, operações de infiltração e a alternativa de um império tornar-se
  uma crise, enquanto outros podem assumir papel de Custodian. [F9]
- O material oficial de Apocalypse descreve saqueadores que podem se unificar e disparar uma crise
  de meio de jogo. [F11]
- O anúncio de 4.0 atribui melhorias de desempenho a mudanças em população e comércio; ele não
  publica estruturas de dados, algoritmo de agregação ou custo por tick. **Não verificado.** [F10]
- Um patch posterior afirma que reduziu economias e frotas “supercharged”, sinalizando que
  crescimento excessivo de poder/economia exigiu rebalanceamento. [F12]

### Aproveitar no ProcedWorld

- Escalar crise a partir de estado mensurável e de orçamento, como a Entropia já propõe, em vez de
  sortear calamidade desconectada de pressões, coesão e histórico.
- Criar mecanismos diplomáticos multilaterais somente quando houver relações e obrigações
  rastreáveis; sanção, favor ou auxílio devem gerar entradas no Ledger e efeitos em dados.
- Agregar população e economia no nível necessário à decisão, medindo o custo por turno desde o
  harness; a referência reforça que simulações tardias são risco de desempenho.
- Tratar expansão de capacidade como fonte de manutenção, atrito e crises, não só de bônus.

### Evitar no ProcedWorld

- Não transformar a Entropia em “crise que encerra o jogo”: o princípio do projeto é ciclo
  infinito, portanto crise deve abrir escolhas de adaptação, colapso parcial ou renascimento.
- Não simular cada detalhe individual sem evidência de que muda decisão; população microscópica
  pode inviabilizar turno mobile e simulação longa.
- Não permitir poder diplomático extraordinário sem prazo, custo e revogação explícitos.

### Pilar/SDD afetado

`docs/gdd/03-economia.md`, `04-cidades-e-populacao.md`, `07-diplomacia.md`,
`08-entropia-e-eventos.md` e `10-ciclo-infinito-e-eras.md`; `docs/sdd/06-entropia.md`,
`07-diplomacia.md`, `15-regras-de-dominio.md` e `14-testes.md`.

## Victoria 3

### O que é

Victoria 3 é uma estratégia da Paradox centrada em economia, política e sociedade; seus diários
de desenvolvimento tratam Pops, profissões, grupos de interesse, bens e preços. [F13] [F14]

Na documentação consultada, Pops trabalham em prédios, têm profissão e podem afetar salário,
força política e afiliação a grupos de interesse. [F13]

### Como funciona o mecanismo relevante

- O diário de emprego afirma que prédios requerem Pops de profissões específicas para produzir
  seus efeitos, e que profissão influencia classe, salários, força política e grupos apoiados. [F13]
- O diário de padrão de vida liga o custo de atender necessidades ao preço local de bens e sugere
  diversificar a economia para aliviar demanda por insumos cruciais. [F14]
- A documentação de bens afirma que preço varia por oferta e demanda. [F15]
- O diário define Interest Group como conjunto de Pops com visões políticas que querem mudar o
  país; lobbies são coleções desses grupos voltadas a uma agenda externa específica. [F16] [F17]
- Fórmulas atuais de mercado, resolução de preço, IA e desempenho interno não são estabelecidas
  aqui além das descrições dos diários. **Não verificado.**

### Aproveitar no ProcedWorld

- Modelar população por grupos agregados com necessidades, trabalho, coesão e inclinações; não é
  necessário copiar Pops individuais para obter consequência social de escassez e política.
- Fazer preço/valor de referência responder a produção, consumo e rotas, mas limitar estoque,
  manutenção e capacidade para que riqueza não vire multiplicador de riqueza ilimitado.
- Derivar facções de condições materiais e instituições: uma facção deve explicar sua demanda por
  fatos do estado, e não aparecer como evento arbitrário.
- Exibir causa econômica curta (“falta de X”, “manutenção de Y”, “contrato Z”) antes de pedir uma
  escolha; é uma leitura de UX aplicável ao relatório causal do ProcedWorld.

### Evitar no ProcedWorld

- Não importar uma economia de mercado completa sem orçamento de CPU e UX; escala deve ser
  estabelecida por métricas do harness, não por ambição de fidelidade.
- Não deixar preço variável mascarar inflação sistêmica. Para o ProcedWorld, valores de referência,
  tetos de reserva, desgaste e manutenção precisam manter comparabilidade inter-eras.
- Não criar facções com opinião sem base no estado: elas precisam de composição, gatilho, objetivo,
  prazo e efeito que possam ser auditados.

### Pilar/SDD afetado

`docs/gdd/03-economia.md`, `04-cidades-e-populacao.md`, `06-sociedade-e-governo.md` e
`07-diplomacia.md`; `docs/sdd/07-diplomacia.md`, `15-regras-de-dominio.md`,
`16-visibilidade.md` e `14-testes.md`.

## Humankind

### O que é

Humankind é um 4X histórico em que o jogador combina culturas ao longo de seis eras, segundo a
apresentação oficial do jogo. [F18]

O site oficial diz que cada cultura adiciona uma camada própria de jogo; o guia oficial de mods
acrescenta que a cultura adotada ou transcendida deixa um Legacy Trait. [F18] [F19]

### Como funciona o mecanismo relevante

- A mudança de cultura por era compõe uma civilização a partir de escolhas históricas distintas.
  O material de modding descreve adoção ou transcendência a cada Era. [F19]
- Affinities dão orientação de gameplay e efeitos/bônus especiais, e o Legacy Trait continua até
  o fim da partida, segundo o mesmo guia. [F19]
- O site identifica fama como critério de vitória; essa parte não é transferível diretamente para
  o ProcedWorld, que não tem vitória obrigatória. [F18]
- A fonte não fornece o algoritmo de escolha de IA ou a resolução de transição de era.
  **Não verificado.**

### Aproveitar no ProcedWorld

- Fazer cada Era pedir uma escolha de foco e renúncia, preservando alguns legados limitados;
  isso concretiza progresso sem depender de uma árvore de bônus linear.
- Versionar legados como dados com custo, pré-condição e capacidade, de modo que possam entrar em
  replays e no catálogo imutável da partida.
- Conectar escolha de era a governo, tecnologia, economia e relações, para que seja uma decisão
  sistêmica e não troca cosmética de skin.

### Evitar no ProcedWorld

- Não empilhar legados sem teto ou manutenção: os próprios objetivos de economia longa e ciclo
  infinito exigem evitar escalada automática de bônus.
- Não usar pontuação/fama como substituto da fantasia de sociedade viva.

### Pilar/SDD afetado

`docs/gdd/05-tecnologia.md`, `06-sociedade-e-governo.md` e
`10-ciclo-infinito-e-eras.md`; `docs/sdd/15-regras-de-dominio.md`,
`18-dsl-catalogos.md` e `09-persistencia.md`.

## The Battle of Polytopia

### O que é

The Battle of Polytopia é um jogo de estratégia por turnos, 4X e multiplataforma, cuja página
oficial o apresenta como “fast paced 4X gaming”. [F20]

A listagem oficial do Android destaca interface enxuta, exploração, cidades, tecnologia,
multiplayer e modos retrato e paisagem. [F21]

### Como funciona o mecanismo relevante

- A página oficial resume o loop como escolher tribo, explorar, construir cidades, desenvolver
  tecnologia e guerrear para controlar o mundo. [F20]
- A listagem do aplicativo declara mapas gerados automaticamente, multiplayer, Pass & Play,
  tratados de paz/embaixadas e suporte a orientação retrato e paisagem. [F21]
- A fonte não especifica tempo típico de sessão, modelo de notificações, arbitragem de conflito
  multiplayer ou arquitetura de servidor. **Não verificado.**

### Aproveitar no ProcedWorld

- Priorizar uma primeira tela de turno com decisão, mapa e consequência imediatamente legíveis;
  o estudo usa a apresentação enxuta de Polytopia como referência de foco, não como cópia de UI.
- Permitir que ações repetitivas virem prioridade/Mandato e reservar os poucos toques para trade-off
  real, como crise, tratado, obra ou defesa.
- Projetar telas para retrato e paisagem desde o contrato de cliente, com mapa, resumo causal e
  ações críticas acessíveis sem painel profundo.

### Evitar no ProcedWorld

- Não reduzir a profundidade sistêmica a uma árvore curta só por ser mobile; a compressão deve
  estar na apresentação e delegação, não na remoção de consequências.
- Não inferir, sem documentação, que o modelo de multiplayer de Polytopia resolve os requisitos
  de ausência, Governador e replay determinístico.

### Pilar/SDD afetado

`docs/gdd/01-loop-e-turnos.md`, `09-governador-e-mandato.md` e
`11-experiencia-mobile.md`; `docs/sdd/03-turnos-e-sessao.md`, `10-protocolo.md` e
`12-cliente.md`.

## Unciv

### O que é

Unciv é um projeto open source inspirado em Civilization V; seu repositório documenta regras de
mods em arquivos JSON e multiplayer online com servidor executável separado. [F22] [F23]

A documentação de modding enumera arquivos para crenças, edifícios, nações, políticas,
tecnologias, terrenos, recursos, melhorias e unidades. [F22]

### Como funciona o mecanismo relevante

- Mods usam muitos arquivos JSON; a documentação especifica tipos de atributos e estrutura por
  categoria de regra. [F22]
- O exemplo de dificuldade expõe parâmetros de regra, inclusive modificadores, duração de acordos
  e intervalos de turno, como dados. [F24]
- A documentação de multiplayer instrui iniciar `UncivServer.jar` e configurar um diretório de
  arquivos de multiplayer. [F23]
- O material consultado não prova que o servidor seja autoritativo, nem descreve validação,
  compatibilidade de mods, hash de regras ou determinismo entre máquinas. **Não verificado.**

### Aproveitar no ProcedWorld

- Separar conteúdo de regras por categorias e IDs estáveis: tecnologia, unidade, terreno,
  tratado, governo e evento devem ter schema versionado e validação offline.
- Exigir manifesto de catálogo, hash e versões no save/replay; isto estende a ideia de regra em
  dados para o requisito determinístico já aprovado.
- Oferecer modding futuro por DSL declarativa fechada, não por scripts que executem no `step`.

### Evitar no ProcedWorld

- Não aceitar JSON como sinônimo de conteúdo seguro: tipo, faixa, referências, conflito de efeito
  e limites de avaliação precisam ser validados antes de uma partida.
- Não permitir que mod altere a semântica de replay em andamento sem migração explícita e
  preservação do catálogo anterior.

### Pilar/SDD afetado

`docs/gdd/02-mapa-e-tiles.md`, `05-tecnologia.md`, `07-diplomacia.md` e
`08-entropia-e-eventos.md`; `docs/sdd/09-persistencia.md`, `15-regras-de-dominio.md` e
`18-dsl-catalogos.md`.

## Freeciv

### O que é

Freeciv é um jogo livre e open source de construção de império; o repositório oficial o descreve
como projeto upstream para cliente e servidor independentes. [F25]

Seu FAQ afirma que é um sistema cliente/servidor e que o multiplayer é assíncrono: durante um
turno, movimentos são enviados ao servidor sem aguardar resultado. [F26]

### Como funciona o mecanismo relevante

- O README de documentação indica iniciar o servidor e conectar clientes a ele; o FAQ permite
  alterar configurações pelo servidor remoto. [F26] [F27]
- A descrição de multiplayer assíncrono mostra humanos agindo concorrentemente dentro do turno e
  trocando comandos com o servidor. [F26]
- A fonte consultada não afirma que a implementação tenha hashes por turno, event sourcing,
  replays bit-a-bit ou a política de conflitos que o ProcedWorld exige. **Não verificado.**

### Aproveitar no ProcedWorld

- Manter separação clara entre cliente e autoridade de simulação: cliente envia intenção/comando,
  servidor valida, ordena, resolve e devolve relatório.
- Tratar rede como transporte de comandos, não como fonte de verdade; desconexão não deve impedir
  o passo, pois o Governador/T0 pode produzir a ação faltante.
- Registrar no relatório do turno a sequência, validação e efeito de cada comando para que a
  simultaneidade seja inteligível, não apenas rápida.

### Evitar no ProcedWorld

- Não pressupor que “cliente-servidor” sozinho resolve fraude, divergência e replay. As garantias
  adicionais do ProcedWorld exigem estado canônico, seed/versionamento e hashes.
- Não introduzir relógio de parede como regra de jogo para lidar com jogadores ausentes; ADR-0008
  fixa prazos internos em turnos.

### Pilar/SDD afetado

`docs/gdd/01-loop-e-turnos.md` e `11-experiencia-mobile.md`; `docs/sdd/01-nucleo-simulacao.md`,
`03-turnos-e-sessao.md`, `09-persistencia.md`, `10-protocolo.md` e `14-testes.md`.

## Recomendações para o ProcedWorld

1. Especificar em `03-turnos-e-sessao` uma matriz canônica de conflito por tipo de comando:
   pré-condição, instante de validação, ordem estável, custo em rejeição e explicação ao jogador.
2. Prototipar o Ledger como dados de relação com fato de origem, intensidade, decaimento, prazo e
   efeito; usar esse mesmo registro como evidência obrigatória de intenção diplomática de bot.
3. Construir eventos como grafos declarativos pequenos: gatilho tipado → opções limitadas →
   comando/efeito validado → entrada de Crônica. Medir cadeias, rejeições e repetições no harness.
4. Usar população agregada por cidade/grupo e só aumentar granularidade se uma decisão concreta
   não puder ser explicada; estabelecer orçamento de CPU/memória antes de criar subagentes sociais.
5. Impor freios estruturais ao crescimento: capacidade, manutenção, estoque perecível, escassez,
   atrito de distância e legado limitado. Validar em simulações de milhares de turnos.
6. Na transição de Era, apresentar foco, renúncia e legado preservável; relações e instituições
   devem mudar por regras explícitas, não resetar silenciosamente.
7. Versionar cada catálogo de regras com schema, hash, IDs e revisões; congelar sua versão por
   partida e gravá-la no replay, como detalha o SDD 18.
8. Desenhar o turno mobile como “mudança → causa → prazo em turnos → até três respostas”, com
   Mandato para rotina e notificação apenas para limite, crise ou oferta relevante.
9. Criar fixtures de diplomacia e eventos encadeados que exijam justificativa completa; falhar o
   teste se uma ação de bot não puder apontar entradas de Ledger e fatos de estado.

## Fontes

- [F1 — Manual oficial de Civilization VI (turnos solo e multiplayer)](https://steamcdn-a.akamaihd.net/steam/apps/289070/manuals/CIV_VI_25TH_ONLINE_MANUAL_ENG.pdf)
- [F2 — Civilization Support: mudanças de Civilization VII](https://support.civilization.com/hc/en-us/articles/37373635171347-Civilization-VII-Changes-from-Civilization-VI)
- [F3 — Entrevista sobre IA e agendas de Civilization VI](https://time.com/4324490/civilization-6-interview/)
- [F4 — Notas de patch de Civilization VII, 22 jul. 2025](https://support.civilization.com/hc/en-us/articles/42883941758867-Civilization-VII-Patch-Notes-July-22-2025)
- [F5 — Mohawk Games: gameplay de Old World](https://mohawkgames.com/oldworld/gameplay/)
- [F6 — Wiki oficial de Old World: Legitimacy](https://wiki.hoodedhorse.com/Old_World/Legitimacy)
- [F7 — Manual oficial de Old World](https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/597180/manuals/Old_World-Official_User_Manual.pdf)
- [F8 — Paradox: Stellaris Federations](https://www.paradoxinteractive.com/games/stellaris/add-ons/stellaris-federations)
- [F9 — Paradox: Stellaris Nemesis](https://www.paradoxinteractive.com/games/stellaris/add-ons/stellaris-nemesis)
- [F10 — Paradox: atualização Stellaris 4.0 “Phoenix”](https://www.paradoxinteractive.com/media/press-releases/paradox-interactive/biogenesis-expansion-and-free-4-0-phoenix-update-available-now-for-stellaris)
- [F11 — Paradox: Stellaris Apocalypse](https://www.paradoxinteractive.com/games/stellaris/add-ons/stellaris-apocalypse)
- [F12 — Paradox: notas de Stellaris 4.3 “Cetus”](https://www.paradoxinteractive.com/games/stellaris/news/cetus-update-is-now-available)
- [F13 — Victoria 3 Dev Diary #11: emprego e qualificações](https://admin-forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-11-employment-and-qualifications.1487723/page-14)
- [F14 — Victoria 3 Dev Diary #13: padrão de vida](https://steamcommunity.com/games/529340/announcements/detail/2965045886379003737)
- [F15 — Victoria 3 Dev Diary #4: bens](https://www.reddit.com/r/paradoxplaza/comments/o20oj0)
- [F16 — Victoria 3 Dev Diary #6: grupos de interesse](https://www.reddit.com/r/Games/comments/obw9xe)
- [F17 — Victoria 3 Dev Diary #112: lobbies políticos](https://www.reddit.com/r/victoria3/comments/1c1fsu2)
- [F18 — Site oficial de Humankind](https://humankind.game/)
- [F19 — Guia oficial de modding de Humankind: culturas](https://medias.games2gether.com/universes/humankind/mods/HUMANKIND_Official_Modding_Guide.pdf)
- [F20 — Site oficial de The Battle of Polytopia](https://polytopia.io/)
- [F21 — Listagem oficial de Polytopia na Google Play](https://play.google.com/store/apps/details)
- [F22 — Unciv: estrutura de arquivos de mods](https://github.com/yairm210/Unciv/blob/master/docs/Modders/Mod-file-structure/1-Overview.md)
- [F23 — Unciv: multiplayer](https://github.com/yairm210/Unciv/blob/master/docs/Other/Multiplayer.md)
- [F24 — Unciv: arquivos JSON auxiliares](https://github.com/yairm210/Unciv/blob/master/docs/Modders/Mod-file-structure/5-Miscellaneous-JSON-files.md)
- [F25 — Repositório oficial do Freeciv](https://github.com/freeciv/freeciv)
- [F26 — FAQ oficial do Freeciv](https://github.com/thejhh/freeciv/blob/master/doc/FAQ)
- [F27 — README de documentação do Freeciv](https://github.com/thejhh/freeciv/blob/master/doc/README)
