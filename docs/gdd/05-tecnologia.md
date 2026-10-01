# 05 — Tecnologia

## Decidido
- A Entropia pode fazer tecnologias "surgirem" como eventos.

## Proposta

### Função no jogo

- Tecnologia oferece respostas novas a problemas do mapa e da sociedade; não é uma pontuação
  de progresso. Conhecer uma técnica não significa conseguir mantê-la durante uma crise.
- O jogador decide o que pesquisar, quanto da produção desviar e quais práticas sustentar.
  Cada escolha troca proteção futura por abastecimento, defesa ou estabilidade imediata
  (ver [03-economia.md](03-economia.md) e [04-cidades-e-populacao.md](04-cidades-e-populacao.md)).

### Árvore base e estados

- Um catálogo versionado define uma árvore base finita. Cada entrada tem `id`, era mínima,
  pré-requisitos, custo de pesquisa, tags de terreno ou recurso, prática associada, custo
  de manutenção e efeitos mecânicos permitidos.
- Os ramos principais são sustento, organização e circulação. Por exemplo, irrigação exige
  armazenamento e acesso a água; escrituração exige organização e facilita rotas.
  Nomes e pré-requisitos exatos são conteúdo a validar no catálogo.
- A interface mostra tecnologias elegíveis, o que falta às bloqueadas e até duas
  consequências próximas, sem exigir navegação por toda a árvore no celular.
- Cada tecnologia está **desconhecida**, **dominada** ou **perdida** para uma civilização.
  Perdida conserva um vestígio na Crônica, mas não pode ser usada antes de redescoberta.
  Pesquisa parcial é um contador separado, preservado ao trocar de projeto.
- Uma tecnologia dominada pode ter sua prática **ativa** ou **inativa**. Inativa não
  produz efeitos nem cobra manutenção. A civilização mantém até três práticas ativas,
  valor inicial, sujeito a balanceamento; o limite não aumenta com as eras.
- O jogador decide qual prática sai para abrir espaço a outra. Uma rede de circulação
  pode melhorar acesso a mercados, mas tirar a proteção da irrigação numa seca.

### Pesquisa por turno

- Há um projeto de pesquisa ativo por civilização. O jogador pode pausá-lo ou trocar
  de projeto sem perder os pontos já aplicados em cada tecnologia.
- A cada turno escolhe investimento de 0%, 10% ou 20% da produção disponível depois
  das obrigações essenciais; faixas e ordem são valores iniciais, sujeitos a balanceamento.
- A conversão inicial é `pesquisa = piso(produção_alocada / 2)`, limitada a 10 pontos
  por turno. Taxa e teto são valores iniciais, sujeitos a balanceamento. Produção que
  não formar um ponto continua disponível no turno.
- Custos da árvore base começam entre 12 e 30 pontos, valores iniciais, sujeitos a
  balanceamento. Concluir a pesquisa muda seu estado para dominada, mas não ativa
  automaticamente a prática.
- Até 5 pontos excedentes passam ao próximo projeto escolhido; teto inicial, sujeito
  a balanceamento. Isso evita que grandes estoques concluam vários projetos num turno.
- Se há privação ou pressão de crise `P ≥ 70`, o cartão recomenda investimento de 0%
  e mostra a renúncia. O motor não substitui a escolha do jogador. `P` e coesão seguem
  a proposta de [00-visao.md](00-visao.md).
- Em uma sessão curta, o cartão “Próximo avanço” mostra custo, prazo aproximado,
  efeito e o que a produção deixará de atender. Projeto e faixa cabem em poucos
  toques (ver [11-experiencia-mobile.md](11-experiencia-mobile.md)).

### Práticas e custo de sustentação

- Cada prática ativa custa de 1 a 3 unidades de produção por turno, faixa inicial,
  sujeita a balanceamento. A manutenção é paga antes de aplicar seu efeito.
- Se falta produção, o motor suspende práticas pela prioridade escolhida pelo jogador;
  empate usa `id` estável. A tela do turno mostra qual efeito deixou de ocorrer.
  O jogador decide se reordena prioridades ou aceita a economia de manutenção.
- Uma prática tem um efeito primário definido pelo catálogo e condições locais.
  Bônus de fontes diferentes respeitam teto por variável: redução máxima inicial
  de 4 pontos na privação `D` e 4 na exposição ambiental `E` por turno, sujeitos
  a balanceamento. Nem tratado nem descoberta emergente ultrapassam esses tetos.
- Exemplo: irrigação dominada exige água nos hexágonos das cidades. Ativa, custa 2
  de produção e reduz em 2 a privação `D` durante seca nessas cidades; números
  iniciais, sujeitos a balanceamento. Sem água ou manutenção, não reduz nada.
- Menor `D` reduz `P` pela fórmula da visão e pode adiar uma crise. O gasto reduz
  a produção disponível para suprimento e pesquisa. O jogador escolhe entre manter
  proteção durante a seca e responder à carência do turno.
- Água vem do mapa (ver [02-mapa-e-tiles.md](02-mapa-e-tiles.md)); produção, privação
  e coesão vêm dos pilares de economia, cidades e sociedade (ver
  [03-economia.md](03-economia.md), [04-cidades-e-populacao.md](04-cidades-e-populacao.md)
  e [06-sociedade-e-governo.md](06-sociedade-e-governo.md)).

### Descobertas emergentes da Entropia

- Descoberta emergente é uma variação de um template tecnológico versionado, não uma
  regra escrita livremente pela IA. O template define tags de elegibilidade, ramo
  da árvore, custo, prática, efeitos permitidos, limites e texto narrativo.
- Após seca registrada e trabalho em terreno árido, por exemplo, a Entropia pode
  propor “captação em terra seca”, variação do template de irrigação. O motor confere
  os fatos citados e valida custo, parâmetros e conflitos antes de criar a oferta.
- A Entropia oferece no máximo um candidato elegível por civilização em cada era,
  valor inicial, sujeito a balanceamento. A oferta usa o orçamento de tensão do
  evento; o mesmo template não se repete na era seguinte.
- Aceitar a oferta torna o projeto pesquisável. Rejeitar não custa produção nem
  bloqueia a árvore base. A oferta expira no fim da era, valor inicial, sujeito
  a balanceamento. O jogador decide se troca um avanço geral por uma adaptação à
  crise vivida (ver [08-entropia-e-eventos.md](08-entropia-e-eventos.md)).
- Faixa inicial para descoberta: 12 a 24 pontos de pesquisa e efeito primário
  limitado a metade do teto global da variável, valores sujeitos a balanceamento.
  Pesquisada, ela ocupa um dos mesmos espaços de prática ativa; nunca concede
  bônus permanente gratuito ou um quarto espaço.

### Conhecimento como assunto diplomático

- Um tratado de intercâmbio nomeia uma tecnologia dominada por uma civilização
  e elegível para pesquisa pela outra. Enquanto vigora, a receptora paga 25% menos
  no custo restante daquela pesquisa, valor inicial, sujeito a balanceamento.
- Pré-requisitos continuam obrigatórios e vários tratados não empilham descontos.
  Encerrar o tratado não apaga pesquisa já feita, apenas retira o desconto futuro.
  Nenhuma tecnologia é transferida instantaneamente.
- O jogador decide se compartilha conhecimento para ganhar tempo ou uma contrapartida,
  sabendo que pode fortalecer um rival. Recusar protege vantagem, mas pode piorar
  relações ou manter uma necessidade própria sem solução.
- Bot só propõe intercâmbio citando tecnologia conhecida, necessidade verificável
  e entradas pertinentes do Ledger de Relações. O motor verifica elegibilidade,
  Mandato, estado diplomático e aceitação; promessa e ruptura entram no Ledger
  (ver [07-diplomacia.md](07-diplomacia.md)).

### Colapso e redescoberta

- Colapso decorre das regras de coesão e era, não do fim da árvore. Tecnologia
  responde à fragmentação com perda de capacidade e escolhas de preservação
  (ver [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Ao escolher uma comunidade sucessora, o jogador preserva até duas tecnologias
  dominadas como legados institucionais, valor inicial, sujeito a balanceamento,
  desde que suas bases materiais sobrevivam no território ou nas instituições.
- Legados seguem dominados, mas suas práticas começam inativas até existir produção
  e condição local. Todas as outras tecnologias dominadas passam a perdidas para
  a sucessora; as nunca dominadas continuam desconhecidas.
- Cada perda deixa vestígio com `id`, era e lugar na Crônica. Uma ruína acessível
  permite ligar redescoberta à história, sem conceder uma tecnologia pronta.
- Redescobrir requer os pré-requisitos dominados. Custa 60% do custo original com
  vestígio acessível e 100% sem ele; percentuais iniciais, sujeitos a balanceamento.
  Desconto de vestígio e tratado tem redução total máxima de 50%, valor inicial,
  sujeito a balanceamento. A prática ainda exige espaço e manutenção.
- O jogador decide se salva irrigação para sobreviver ou escrituração para reconstruir
  rotas. A perda cria um objetivo recuperável, preservando a agência no renascimento.
- Se ausente, o Governador propõe legados segundo o Mandato e a crise. Fallback T0
  escolhe opções elegíveis em ordem estável. O relatório de retorno mostra o que
  foi preservado, perdido e pode ser redescoberto.

### Depois do fim da árvore

- Concluir a última tecnologia base não encerra a partida nem aumenta rendimentos.
  Eras seguem marcos do mundo; manutenção, vizinhos e crises seguem criando escolhas
  (ver [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md)).
- Pesquisa pode financiar uma **revisão** de prática dominada: custo inicial de
  20 pontos, sujeito a balanceamento. Revisão troca um parâmetro dentro do template,
  como menor manutenção por efeito menor, ou maior efeito local por manutenção maior.
- Cada prática guarda só a revisão vigente. Uma nova substitui a anterior sem devolver
  pesquisa. Nenhuma revisão cria efeito ou ultrapassa tetos globais. O jogador decide
  entre adaptar o que já sabe, estudar uma descoberta emergente ou guardar produção.
- Novos ramos podem chegar em versões futuras por migração de catálogo versionado;
  a continuidade de uma partida não depende deles.
- Espaços fixos, manutenção, tetos por variável, revisões substitutivas e perda
  no colapso impedem crescimento descontrolado. Uma sociedade madura ganha opções
  e memória, não multiplicadores sem fim.

### Motor, Governador, Entropia e bots

- **Determinístico:** catálogo, pré-requisitos, custos, conversão de pesquisa,
  conclusão, ativação, suspensão, elegibilidade de eventos, tetos, tratados,
  perda, redescoberta e revisão. Sorteios usam seed explícita e dados versionados.
  Intenções aceitas viram comandos no log, reproduzíveis conforme ADR-0006.
- **Entropia propõe:** `template_id`, alvo, fatos de elegibilidade e parâmetros
  dentro do schema. O motor valida e aplica só efeitos do template; narrativa
  e nome da descoberta não alteram o estado.
- **Governador propõe:** projeto, investimento, prioridade de práticas e legados
  dentro do Mandato (ver [09-governador-e-mandato.md](09-governador-e-mandato.md)).
  O motor rejeita ação sem pré-requisito, recurso ou permissão. Sem IA, T0 prioriza
  reduzir privação quando `P` está alto e mantém o projeto atual em situação estável.
- **Bots diplomáticos propõem:** intercâmbio com fatos do estado e do Ledger.
  Aceitação, duração, desconto e efeito nas relações são regras do motor.
  Saída inválida ou timeout usa fallback T0; replay nunca consulta IA novamente.

## Perguntas abertas
- Árvore fixa, árvore com ramos aleatórios por mundo, ou totalmente emergente?
  - **Recomendação:** árvore base fixa em dados e descobertas por templates validados.
    Pré-requisitos previsíveis facilitam decisões curtas e balanceamento; variações
    ligadas ao mapa e à Crônica dão identidade ao mundo sem efeitos inventados pela IA.
- O que acontece quando a árvore termina num jogo infinito?
  - **Recomendação:** manter revisões substitutivas e descobertas ocasionais como
    escolhas horizontais, sem níveis infinitos de bônus. Assim, crises continuam
    relevantes e cada era pede recomposição de prioridades.
- A preservação no colapso deve ser escolha direta ou efeito automático de instituições?
  - **Recomendação:** deixar o jogador escolher até dois legados entre os elegíveis
    pelas instituições e territórios sobreviventes. Há agência sem salvar uma prática
    que perdeu toda a sua base material.
