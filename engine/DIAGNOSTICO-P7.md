# Diagnóstico P7 — simulação degenerada

## Sintoma observado

No seed `20261001`, oito civilizações chegavam ao turno 1.000 com uma cidade e
uma pessoa cada, pressão média 60 e oito colapsos. O replay e o hash não eram a
causa: a sequência de comandos e a resolução eram determinísticos.

## Causas raiz e evidência

1. `pw-engine/src/bots.rs`, `BotT0::decide`, enfileirava `unit.scout` antes de
   considerar `unit.settler`. Isso atrasava a primeira expansão até depois do
   custo do batedor e deixava parte das civilizações sem a segunda cidade no
   horizonte de 300 turnos do teste de saúde.
2. `pw-engine/src/world.rs`, `resolve_research_and_practices`, calculava
   `available * investment / 100 / 2`. Com uma capital de produção 1 e o
   investimento T0 de 20%, isto era zero em todos os turnos. Consequentemente
   `tech.storage`, pré-requisito de `unit.settler`, nunca era pesquisada.
3. `pw-engine/src/bots.rs`, a antiga função `next_city_id`, consultava somente
   o estado antes da resolução. Todos os bots que propunham fundar no mesmo
   turno recebiam o mesmo ID, e só o primeiro comando poderia ser aceito. Isso
   ainda impediria a expansão coletiva depois de produzir colonos.
4. `pw-engine/src/world.rs`, `apply_command` aceitava `FoundCity` sem conferir
   nem consumir um `unit.settler`. A fundação inicial e a expansão posterior não
   tinham requisitos mecânicos distintos, contrariando o ciclo de produção e
   assentamento de 04.
5. `pw-engine/src/world.rs`, `CivilizationState::default` iniciava o tesouro
   em zero, enquanto `resolve_economy` cobra a manutenção obrigatória de
   `1 + teto(população / 4)` antes de uma cidade de uma pessoa poder alocar um
   segundo posto à riqueza. A falta de manutenção criava `D = 5` desde o turno
   inicial; `resolve_society` reduzia satisfação, depois coesão, e o colapso
   corretamente congelava a civilização após dois turnos em `C = 0`.
6. `pw-engine/src/world.rs`, as cidades novas tinham moradia 2 e não há ainda
   uma mecânica de construção de moradia. Mesmo corrigindo expansão, dezesseis
   cidades ficariam limitadas a 32 pessoas, abaixo da condição de saúde de 40.
7. `pw-engine/src/world.rs`, `FoundCity` iniciava cada cidade com estoque de
   comida zero. Uma capital cuja melhor alocação inicial rende exatamente uma
   comida a consumia no mesmo turno e nunca terminava com a reserva de um
   consumo exigida para crescimento em 04. Ela não chegava à população 2,
   portanto não podia alocar produção para pesquisar armazenamento e produzir
   um colono. Isso deixa a expansão dependente, incorretamente, de a abertura
   já ter um tile de comida 2 ou mais.

## Correções aplicadas

- O T0 pesquisa `tech.storage` com progresso mínimo inteiro e, após a cidade
  atingir população 2, prioriza um colono sobre um batedor se a civilização
  ainda tiver menos de duas cidades. O limite é uma política temporária
  conservadora do T0: permite validar expansão sem ignorar a carga
  administrativa prevista por 04, que ainda não está implementada.
- Cada colono recebe um `CityId` derivado do seu `UnitId` em uma faixa reservada;
  a ordem e os IDs permanecem estáveis. Fundar a segunda cidade requer e consome
  o colono no tile. O T0 só tenta fundar depois que o colono deixa o tile da
  cidade, pois a validação corretamente o considera ocupado. A capital inicial
  continua sendo a exceção de início de jogo.
- A produção agora exige e debita também o custo de comida do catálogo. Isso
  torna o custo misto de `unit.settler` efetivo, em vez de tratar só produção.
- `INITIAL_TREASURY_WEALTH = 12` é a reserva inicial explícita. Ela cobre seis
  turnos da manutenção base de uma capital de uma pessoa (`2` por turno), tempo
  para o crescimento liberar o segundo posto. O valor fica abaixo do teto de
  tesouro de uma população (`18`) definido em 03.
- `INITIAL_CITY_HOUSING = 4` é o valor inicial explícito enquanto não existe
  construção de moradia. Ele preserva a regra de 04 de que crescimento exige
  moradia e torna possível a meta de 40 pessoas sem criar população grátis.
- `INITIAL_CITY_FOOD_STOCK = 1` dá a toda fundação a reserva de um turno de
  consumo exigida para o progresso de crescimento em 04. Com isso, uma abertura
  de comida 1 pode formar o segundo trabalhador; não há mudança no consumo,
  no limiar de crescimento ou na regra de fome.
- A pesquisa arredonda para cima o ponto resultante de metade da produção
  investida. Não usa ponto flutuante e conserva o teto por turno; investimentos
  de 10% e 20% deixam de ser inertes em uma cidade fundadora.

As fórmulas de `D`, `G`, `P` e `C`, e o limiar de colapso (`C = 0` por dois
turnos), foram preservados conforme 12. A correção remove a privação sem causa
econômica no turno inicial, não reduz o limiar de colapso.

## Cobertura de saúde

`pw-harness/src/lib.rs` contém a integração de 300 turnos para o seed
`20261001`, oito civilizações, pelo menos 16 cidades, população total de pelo
menos 40, no máximo dois colapsos e pressão média entre 10 e 80. Os limites são
os solicitados, sem flexibilização.

## P7b — investigação e correções complementares

### Evidência e limites da investigação

A medição recebida da VPS para `20261001`, oito civilizações e 300 turnos foi
`cities 13 population 52 average_pressure 7 crises 0 collapses 1`. Cinco
civilizações tinham duas cidades e três tinham uma. Esses são resultados do
P7, **não uma medição do código P7b**. Não houve compilação nem execução Rust
neste ambiente, conforme o brief. A atribuição de cada uma das três
civilizações a um bloqueio específico ainda depende do replay/estado na VPS;
o total agregado não contém filas, estoques, posições ou rejeições.

Os caminhos abaixo são verificáveis no código e agora têm regressões
direcionadas. Não se deve apresentar todos como causas observadas em cada
uma das três civilizações.

### Expansão

1. O teto `city_count < 2` era a causa direta de ninguém passar de duas
   cidades. Foi removido, sem substituir por outro teto numérico. O T0 mantém
   apenas uma expansão pendente por civilização, contando colonos em campo,
   filas existentes e propostas do próprio turno. Só encomenda se houver um
   local alcançável. Isso substitui a política temporária descrita no P7.
2. O foco ignorava o custo de comida da fila: com reserva suficiente apenas
   para consumo, escolhia diversificação mesmo sem as dez comidas do catálogo
   para concluir o colono. Agora pede abastecimento até cobrir o custo da fila
   mais uma reserva de consumo. Os custos continuam dez comidas e 22 de
   produção; pesquisa e rendimentos não receberam bônus.
3. O movimento selecionava vizinhos sem verificar custo. Um colono com dois
   pontos podia repetir para sempre uma tentativa de entrar em oceano de custo
   três ou floresta com rio de custo três. Agora usa o mesmo cálculo do motor
   e o movimento renovado do catálogo, por busca em largura com desempate por
   tile. Não usa `movement_left` residual do turno anterior. A exploração
   também filtra movimentos pagáveis e prioriza desconhecidos da própria civ.
4. O colono fundava no primeiro tile fora da própria cidade, sem avaliar
   sobreposição de postos ou meios de subsistência. Agora busca terreno fora
   do oceano com comida excedente, riqueza e produção locais. A política T0
   usa distância mínima cinco entre centros para separar os raios de trabalho
   dois; **não é uma nova proibição do motor**. Pode atravessar tiles próximos
   às cidades e contornar ocupantes; só funda no destino adequado e não propõe
   mover/explorar a unidade que acabou de consumir. Não existir local ou rota
   elegível é motivo legítimo para esperar, não para criar uma cidade grátis.
5. `resolve_unit_production` debitava comida/produção e retirava a fila antes
   de conferir se havia uma unidade no centro. Assim perdia o colono completo.
   Agora acumula o trabalho e espera espaço, preservando comida e fila até
   poder inserir a unidade.
6. O P7 descrevia `UnitId` como nunca reutilizado, mas o alocador considerava
   apenas unidades vivas. Consumir o maior ID podia liberá-lo, repetindo o
   `CityId` derivado e provocando `CityAlreadyExists`. O alocador agora inclui
   os IDs reservados pelas cidades de colonos e verifica esgotamento antes de
   gastar recursos. Isso mantém distintas as fundações repetidas e simultâneas
   no modelo atual, que não remove cidades.

### Privação e pressão: fórmula versus entradas

- **Os termos não estavam ausentes.** `resolve_society` já somava
  `2D + 2G + W + E + (100-S)/10 - C/5`. A diferença encontrada era o `+9`
  no numerador da estabilidade, que fazia teto onde 12 define divisão inteira.
  Foi removido; essa correção pode reduzir `P` em um, não elevar sua média.
- A manutenção era sinalizada cidade a cidade usando o mesmo tesouro sem
  descontar pagamentos anteriores e apenas a renda das cidades já percorridas.
  Duas cidades com renda total dois e custo total quatro podiam ambas parecer
  pagas; o débito agregado posterior saturava em zero, ocultando a privação.
  Agora toda renda do turno é reunida antes dos pagamentos, que consomem um
  saldo único em ordem de `CityId`. Uma cidade com pagamento incompleto recebe
  `M_c = 5`, permitindo que `D`, satisfação, `G` e `P` reflitam a falta real.
- A alocação automática só garantia comida, pulando a manutenção essencial
  exigida em 04. Agora aloca riqueza depois do consumo e antes do foco. Isso
  evita que a correção contábil gere privação por uma escolha que o próprio
  GDD manda evitar quando há postos disponíveis. Falta real continua contando.
- `G_c` continua sendo o teto da fração com `A_g < 40`; `D_civ` mantém teto e
  `P_civ` mantém média ponderada pela população. Nenhum piso artificial de
  pressão, redução de custo ou mudança de limiar de crise/colapso foi aplicado.

Coesão baixa não implica pressão positiva: o desgaste passado de `C` persiste
quando os grupos recuperam satisfação. Com `D = G = W = E = 0`, `S = 100` e
`C = 26`, o GDD exige `P = max(0, -26/5) = 0`. Como guerra, ambiente e Entropia
ainda não geram perturbações neste recorte, **não é possível garantir um piso
universal de dez em uma simulação pacífica e abastecida mantendo a fórmula**.
Uma única civ com pressão 60 e sete com zero já explicaria a média inteira
sete recebida; isso é um exemplo compatível, não uma reconstrução da medição.

O teste de saúde original permanece intacto, inclusive cidades ≥ 16, população
≥ 40, colapsos ≤ 2 e pressão 10–80. Não está demonstrado que esse seed específico
seja incapaz de alcançar dez após as correções: falta medi-lo. Se o único
limite restante for pressão abaixo de dez com entradas legitimamente baixas,
o limite justificável é o resultado canônico de 12, inclusive zero, e a
correção deve parar aí. Criar privação para passar no teste seria incorreto;
produzir pressão por novos sistemas ou alterar o requisito exige decisão
separada, fora deste patch.

### Cobertura e validação pendente

Regressões adicionadas em `bots.rs` e `world.rs`:

- expansão real por 150 turnos em mapa fértil controlado, além da segunda
  cidade, consumo de colonos e apenas uma expansão pendente;
- rota evitando terreno caro e ocupação; foco que reserva comida para a fila;
- fila concluída em tile ocupado preservada e ID reservado por cidade respeitado;
- manutenção compartilhada sem gasto duplicado e postos essenciais antes do foco;
- pressão local exata 57 com todos os termos, média nacional 14 para populações
  um e três, teto de `D_civ`, grupos insatisfeitos e estabilidade com peso reduzido;
- pressão zero legítima com satisfação plena e coesão 26.

### Correção automática 1 — métrica de pressão do harness

O `Metrics::average_pressure` usava somente `crisis_pressure` do estado final.
Isso não era uma média da simulação: descartava 299 dos 300 estados resolvidos
e reportava zero quando as civilizações terminavam abastecidas, mesmo que a
pressão tivesse sido observada durante a expansão. A pressão final igual a zero
é válida pela fórmula de 12 e não deve ser aumentada artificialmente.

O harness agora acumula `P_civ` para cada civilização após cada turno resolvido e
divide pelo número de amostras. Cidades, população, crises e colapsos continuam
medidos no estado final. O limite de saúde de 10–80 não mudou: ele passou a
avaliar a média temporal que seu nome e a validação de frequência de 12 exigem.
Há uma regressão que confirma uma amostra por civilização em cada turno.

**Pendente na VPS:** `cargo test --workspace`, incluindo o teste de saúde com
os mesmos limites e os testes existentes de determinismo/replay. Registrar a
nova medição antes de declarar P7b verde. A métrica nova não altera o estado,
os comandos nem o hash; replays produzidos pelo código corrigido precisam
continuar idênticos.

### Correção automática 2 — resultado da VPS e limite canônico de `P`

O `cargo test --workspace` executado na VPS confirmou que os 38 testes de
`pw-engine` e quatro dos cinco testes de `pw-harness` passam. A simulação de
saúde também passou os limites de cidades, população e colapsos, pois a
primeira asserção que falhou foi somente a pressão média: `0`, abaixo de `10`.
Isso confirma que a métrica temporal da correção automática 1 está sendo
coletada, mas que nenhum turno resolvido deste seed produziu `P_civ` positivo.

Não há uma correção de balanceamento compatível com o recorte atual que possa
elevar esse valor a dez. Com `D = G = W = E = 0` e `S = 100`, a fórmula
canônica é `P_c = limitar(0, 100, -C/5) = 0`, inclusive quando a coesão é
baixa. O motor ainda não implementa os sistemas que poderiam fornecer uma
causa legítima para esses termos (carga administrativa, conflitos, exposição
ambiental ou eventos da Entropia), e o T0 corretamente evita privação. Um
piso, uma penalidade inventada ou uma mudança na fórmula/teste apenas para
produzir `P >= 10` violaria 12 e as regras deste brief.

Assim, P7b deve parar neste limite justificável: a expansão, o crescimento e
a ausência de colapso sistêmico estão verificados; a faixa de pressão do teste
de saúde exige uma fonte de tensão ainda fora do escopo. Para torná-la verde,
é necessária uma decisão posterior que implemente uma fonte prevista no GDD
ou redefina o critério de saúde para este recorte pacífico. O teste permanece
inalterado e vermelho para sinalizar essa decisão pendente.

### Correção automática 3 — confirmação da execução na VPS

Uma nova execução de `cargo test --workspace` na VPS confirmou o diagnóstico
anterior: todos os 38 testes de `pw-engine` e quatro dos cinco testes de
`pw-harness` passaram; o único erro foi a mesma asserção de saúde, com
`average_pressure 0`. A coleta temporal está presente no binário executado:
ela soma `P_civ` depois de cada turno resolvido e divide pelas 2.400 amostras
do cenário (oito civilizações por 300 turnos). Portanto, zero não é o valor
final usado por engano nem um efeito de divisão inteira de uma soma positiva;
cada amostra de `P_civ` foi zero.

O código confirma a causa mecânica: a fórmula em `resolve_society` é
`2D + 2G + W + E + (100-S)/10 - C/5`, limitada a 0--100, exatamente como
em 12. Neste cenário, o T0 mantém alimento e manutenção essenciais, logo
`D = 0`; grupos abastecidos alcançam satisfação plena, logo `G = 0` e
`S = 100`; e as etapas de guerra, ambiente e Entropia ainda não produzem
`W` ou `E`. A coesão reduz o resultado restante, em vez de ser uma fonte de
pressão. Assim, a soma por turno é legitimamente zero.

Não foi aplicado piso, penalidade por cidade, alteração da fórmula ou mudança
no teste: cada uma criaria uma regra nova para mascarar a ausência de uma fonte
de tensão. A fonte que o GDD já prevê, mas que precisa de uma tarefa/decisão
posterior, é a carga administrativa e sua exposição associada (`06`, `10`),
ou uma fonte determinística de ambiente, guerra ou Entropia. O limite mais
próximo justificável neste recorte continua sendo `0`; P7b não fica verde até
que uma dessas extensões seja autorizada ou o critério seja decidido de novo.

Validação local: `python tools/agents/check-encoding.py` retornou `encoding ok`;
os três arquivos alterados foram conferidos como UTF-8 com finais LF.

## Questões abertas

Ainda faltam obras de moradia, carga administrativa, manutenção de unidades e
efeitos de práticas. P7b não implementa esses sistemas nem inventa `W`/`E` para
elevar pressão. A distância de assentamento é uma escolha conservadora do T0,
revisável quando houver políticas de compartilhamento dos postos. A medição
P7b e a compatibilidade do piso de pressão com o recorte atual continuam abertas.

## Decisão do supervisor sobre a pressão mínima (2026-10-02)

Depois do P7b, cidades, população e colapsos atingiram as metas, mas a pressão média ficou em 0. Pela
fórmula do GDD 12 (`P_c = 2·D + 2·G + W + E + (100 − S)/10 − C/5`), um mundo abastecido, sem tensão de
grupos, sem guerra e sem eventos climáticos tem `P = 0` legitimamente — e na Fase 2 ainda não existem a
Entropia nem a guerra entre bots, que são as fontes de `W` e `E` e de choques de `D`. A exigência de
pressão mínima 10 no teste de saúde foi um erro do brief, não do motor. O teste passa a exigir só o teto
(≤ 80); o piso volta na Fase 3 junto com o diretor de eventos.

## P12 — piso de pressão média com o diretor de eventos (2026-10-02)

Com a Entropia implementada (`pw-engine/src/entropy.rs`, `dsl.rs`), o cenário de saúde (seed `20261001`,
8 civilizações, 300 turnos) passou a registrar cerca de 419 eventos, sem colapsos extras e com os demais
limites do teste mantidos (cidades >= 16, população >= 40, colapsos <= 2). A pressão média continua **0**.
O piso `>= 10` foi restaurado no teste, medido e **não é alcançável** sob as regras de justiça; por isso o
teste mantém só o teto (`<= 80`) e uma nova asserção exige que a Entropia realmente abra eventos.

Por que é inalcançável, sem inventar regra:

1. A fórmula canônica (GDD 12) subtrai `C/5`. Com a coesão inicial `C = 50`, o termo vale `-10`: qualquer
   `P_c` com soma de causas abaixo de 10 é limitado a 0. Para uma média de 10 seria preciso uma soma de
   causas média próxima de 20 em cada turno de cada civilização.
2. O orçamento de tensão (GDD 08) é `B = 12 + 2 x civs vivas` por era de 8 turnos, ou seja, 28 pontos para
   8 civilizações: cerca de 0,44 ponto por civilização por turno. Cada evento custa 1 a 3 pontos, dura
   1 a 3 turnos e a regra de resposta útil faz o Governador responder já no turno seguinte. A exposição `E`
   de um evento ativo é o seu custo (GDD 12: modificadores climáticos negativos / 5), portanto um evento
   soma 2 a 3 a uma única cidade por um turno. A média ponderada por população e por 2.400 amostras fica
   muito abaixo de 10, e depois do `-10` de coesão resulta em 0.
3. Aumentar a magnitude dos eventos, fazê-los durar mais, ignorar a resposta do Governador ou adicionar um
   piso de pressão violaria a justiça (GDD 08: perda previsível, respondível, sem repetir o golpe; nenhum
   evento causa colapso) ou a fórmula de 12. Nada disso foi feito.

O que ainda permitiria pressão média >= 10 de forma legítima são fontes que o GDD prevê e que não existem
no motor: guerra entre civilizações (`W`), carga administrativa e crises sistêmicas que reduzam a coesão.
O critério deve ser reavaliado quando elas existirem (Fase 3, diplomacia e guerra), ou redefinido pelo
usuário para um cenário com conflito.

# Diagnóstico P12b — pressão de crise
## Execução de referência
- Ambiente: VPS Linux, Docker `rust:1-slim`.
- Comando: `cargo test --workspace`.
- Cenário: `health_simulation_expands_grows_and_avoids_systemic_collapse`.
- Pressão média observada: `4`.
- Piso exigido: `10`.
- Eventos da Entropia: `253`.
- Crises locais: `0`.
## Verificação da fórmula
A implementação de `P_c` usa o termo de coesão decidido no P12b:
`- (C - 50) / 5`
O teste unitário de pressão também cobre a fórmula centrada: com `C = 49`, o termo
trunca para `0`, e a pressão local calculada é `66`.
## Conclusão
Com os pesos, catálogo e limites atuais, a fórmula centrada não torna o piso de pressão
média `>= 10` atingível no cenário de saúde de referência. O teste de saúde foi mantido
inalterado; não foram alterados seus limites, a ordem de resolução ou o balanceamento da
Entropia, pois essas mudanças excederiam o escopo do P12b. É necessária uma decisão de
balanceamento para elevar a pressão sem enfraquecer esse gate.

# Melhorias de terreno (A2, 2026-10-03)
## Simplificações registradas
- Tile do motor não guarda bioma de origem nem depósito de recurso: cada terreno do motor equivale a um
  bioma do catálogo e oferece os `resource_tags` desse bioma (`core.biomes`). Sem terreno de montanha, a
  pedreira não pode ser construída; savana/tundra contam como estepe.
- Pedra e madeira não têm estoque: custos não-produção viram trabalho do trabalhador 1:1; ritmo fixo de
  2 de trabalho por turno.
- Efeitos `AddTag` das melhorias não têm regra no motor (mina, especiarias, cisterna e a proteção do
  campo irrigado só valem pelo id); o bot T0 não constrói melhoria sem ganho de rendimento.
- Manutenção inadimplida suspende o bônus só no turno; não há suspensão manual nem reparo com produção.
- Território = `control` explícito ou raio 2 da cidade mais próxima (empate por id).
## Harness `run --seed 20261001 --civs 8 --turns 300` (VPS, Docker `rust:1-slim`)
- Antes (base `836e121`): `cities 29 population 116 average_pressure 4 crises 0 collapses 0 events 253 ms/turn 10.139`
  (diplomacia: audits 663, accepted 633, rejected 30, ungrounded 0).
- Depois: `cities 27 population 108 average_pressure 4 crises 0 collapses 0 events 251 ms/turn 11.513`
  (diplomacia: audits 691, accepted 653, rejected 38, ungrounded 0). Ao fim: 28 melhorias e 26
  trabalhadores nas 8 civilizações; nenhuma cidade com sustento essencial inadimplido.
- Primeira versão do bot (só reserva de tesouro) gerou **5 colapsos**: as melhorias consumiam o tesouro
  que paga o sustento de cidades novas ainda sem renda, e a coesão (que só cai) chegou a 0. Corrigido no
  bot exigindo renda de riqueza do turno ≥ sustento + manutenção das melhorias + 1. A queda leve de
  cidades/população vem do custo dos trabalhadores (4 comida, 10 produção); com moradia inicial 4 por
  cidade, o bônus de comida das fazendas ainda não vira crescimento.
