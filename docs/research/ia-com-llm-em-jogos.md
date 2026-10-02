# Estudo — IA com LLM em jogos

> **Status:** pesquisa para orientar GDD/SDD; não altera decisões aprovadas.
> **Escopo:** separar linguagem de decisão, preservar o motor determinístico e
> tratar memória, custo, segurança e replays como contratos de produto.

## Leitura executiva

Um LLM é especialmente útil para converter estado em explicação, proposta e
narrativa. Ele é uma fonte fraca para autoridade mecânica: pode alucinar,
esquecer contexto, atrasar ou falhar. A arquitetura já proposta para o
ProcedWorld — T0 determinístico, T1 para escolhas tipadas e T2 para texto e
planejamento — é a divisão mais defensável à luz dos projetos pesquisados.

CICERO é o caso mais diretamente aplicável: o plano estratégico controla o
diálogo, não o contrário. [F1] Smallville mostra como memória, reflexão e
planejamento podem gerar comportamento social legível, mas não torna a memória
uma fonte de verdade. [F3] Voyager favorece bibliotecas de habilidades e
feedback do ambiente, não uma conversa infinita. [F6]

Para este jogo, a regra operacional é: **o modelo propõe; o motor prova,
converte e aplica**. Uma frase bonita nunca pode aceitar tratado, mudar um
tile, gastar recurso, criar fato no Ledger ou alterar o Mandato.

## Modelo de referência para o ProcedWorld

### Responsabilidade por camada

| Camada | Decisão permitida | Entrada | Saída | Falha |
|---|---|---|---|---|
| T0 | regra/utility determinística | estado canônico, catálogo, Mandato | comando canônico | não se aplica |
| T1 — Jev, depois Laya | escolher/ordenar opções já legais | lista fechada e fatos | ID, score ou probabilidade quantizada | T0 |
| T2 — LLM | plano de alto nível, interpretação e prosa | resumo derivado, fatos e dados isolados | intenção em schema e texto | T0 + texto canônico |

T1 não deve receber ação livre: ele só classifica candidatos que T0 já filtrou.
T2 não deve receber ferramentas, acesso a estado mutável nem credenciais. Essa
divisão implementa o contrato de `DecisionPort`/`LLMPort` proposto no SDD 04.

### Pipeline proposto

```text
estado + catálogo + Mandato + Ledger (canônicos e versionados)
                         |
                 contexto mínimo por finalidade
                         |
                  T0 / T1 / T2 propõe
                         |
schema fechado -> grounding por IDs -> validação de regras -> Command
                         |
                    event log -> step determinístico -> hash/replay
```

O texto final pode ser regenerado ou gravado como evidência; o `Command` e sua
versão de regras são o que reproduz a partida. Isso é consistente com o motor
puro e event sourcing exigidos em `CLAUDE.md` e SDD 01/04/09/14.

## CICERO — Meta / Diplomacy

### O que é

CICERO é um agente para Diplomacy, jogo de estratégia no qual jogadores fazem
ordens simultâneas e negociam em linguagem natural. A Meta descreve-o como a
combinação de motor de raciocínio estratégico e modelo de diálogo controlável.
[F1] [F2]

### Como funciona o mecanismo relevante

O sistema estima ações prováveis dos demais a partir do tabuleiro e do histórico
de conversa, refina um plano e condiciona a geração de mensagens a intenções
estratégicas. Assim, a linguagem comunica um plano previamente escolhido, em
vez de decidir livremente o movimento no tabuleiro. [F1] [F2]

O repositório publicado também evidencia que esse resultado veio de treinamento
e infraestrutura de pesquisa substanciais; não é uma receita de integração
leve para um servidor pequeno. [F2]

### Aproveitar no ProcedWorld

- Planejar ou escolher termos primeiro; redigir proposta diplomática depois.
- Exigir que uma intenção cite `ledger_entry`, cidade, rota, tratado ou evento
  que a sustente; exibir ao jogador poucos fatos verificáveis.
- Separar previsão sobre o outro ator de aceitação mecânica: a primeira pode ser
  T1/T2; a segunda permanece fórmula/regra do motor.
- Usar o plano por era para orientar mensagens e Crônica, sem permitir que ele
  sobrescreva Mandato, reservas ou linhas vermelhas.

### Evitar no ProcedWorld

- Não inferir que persuasão textual equivale a compromisso real. Promessa só
  vira obrigação por comando de tratado aceito e entrada no Ledger.
- Não copiar um agente monolítico treinado para Diplomacy nem depender de
  treinamento de grande escala para o MVP.
- Não chamar T2 em toda microdecisão ou para calcular a aceitação de oferta.

### Pilar/SDD afetado

GDD [07-diplomacia.md](../gdd/07-diplomacia.md),
[09-governador-e-mandato.md](../gdd/09-governador-e-mandato.md); SDD
[04-camadas-de-ia.md](../sdd/04-camadas-de-ia.md),
[05-governador.md](../sdd/05-governador.md) e
[07-diplomacia.md](../sdd/07-diplomacia.md).

## Generative Agents / Smallville — Stanford

### O que é

O artigo apresenta agentes generativos em uma cidade-sandbox inspirada em The
Sims; a demonstração contém 25 agentes que agem e conversam em linguagem
natural. [F3] O objetivo declarado é comportamento social crível, não uma
simulação estratégica determinística. [F3]

### Como funciona o mecanismo relevante

O agente registra experiências em linguagem natural, recupera memórias conforme
relevância, sintetiza reflexões de nível mais alto e usa essas informações para
planejar. O próprio estudo reporta que observação, planejamento e reflexão
contribuíram para a verossimilhança no experimento. [F3]

### Aproveitar no ProcedWorld

- Manter a separação `working → episodic → semantic → procedural` já prevista.
- Consolidar observações em fim de período/era, produzindo Crônica e Doutrina
  curtas, com referências aos fatos que as originaram.
- Recuperar memória por escopo e relevância para explicar uma decisão, em vez de
  anexar todo o passado a cada prompt.
- Tratar reflexão como hipótese ou preferência do Governador; nunca como fato
  que substitui estado, catálogo ou Ledger.

### Evitar no ProcedWorld

- Não persistir cada frase como contexto perpétuo; o custo e a contradição
  crescem, e o jogo não pode depender da fidelidade textual de um resumo.
- Não usar uma reflexão para criar dívida, rancor, recursos ou causalidade sem
  evento canônico correspondente.
- Não transformar o comportamento "crível" do sandbox em promessa de justiça
  ou balanceamento entre civilizações.

### Pilar/SDD afetado

GDD [06-sociedade-e-governo.md](../gdd/06-sociedade-e-governo.md),
[07-diplomacia.md](../gdd/07-diplomacia.md),
[10-ciclo-infinito-e-eras.md](../gdd/10-ciclo-infinito-e-eras.md); SDD
[08-memoria-e-contexto.md](../sdd/08-memoria-e-contexto.md) e
[04-camadas-de-ia.md](../sdd/04-camadas-de-ia.md).

## AI Dungeon — Latitude

### O que é

AI Dungeon é uma experiência de aventura narrativa dirigida por IA. Em devlog,
a própria Latitude identifica perda de continuidade quando informação relevante
sai da janela de contexto como um problema central da experiência. [F4]

### Como funciona o mecanismo relevante

O produto expõe memória e informação de mundo para conservar elementos de uma
aventura; a fonte consultada descreve jogadores tentando administrar memória,
informações do mundo e edições para compensar esquecimentos. [F4]

A Latitude também descreve moderação como processo contínuo, alimentado por
feedback, denúncias e dados de uso, não como uma regra simples que resolve todos
os casos. [F5]

### Aproveitar no ProcedWorld

- Fazer do Resumo do Estado, Ledger e Crônica documentos derivados,
  reconstruíveis e inspecionáveis, não uma conversa opaca.
- Medir saturação e executar compactação/reset previsível antes de perder
  continuidade; após reset, reconstruir do estado e do log.
- Projetar classificação, denúncia e revisão de conteúdo narrativo antes da
  abertura ao público, separadas da validação mecânica.

### Evitar no ProcedWorld

- Não pedir ao jogador que mantenha manualmente a memória de uma civilização.
- Não deixar a narrativa livre definir fatos persistentes ou resultado de ação.
- Não adiar política de conteúdo: narrativa com texto livre gera operação de
  moderação, não apenas engenharia de prompts.

### Pilar/SDD afetado

GDD [08-entropia-e-eventos.md](../gdd/08-entropia-e-eventos.md),
[09-governador-e-mandato.md](../gdd/09-governador-e-mandato.md),
[11-experiencia-mobile.md](../gdd/11-experiencia-mobile.md); SDD
[08-memoria-e-contexto.md](../sdd/08-memoria-e-contexto.md),
[11-seguranca-e-chaves.md](../sdd/11-seguranca-e-chaves.md).

## Suck Up!

### O que é

Suck Up! é citado neste brief como referência de interação com NPCs em linguagem
natural. Não foi localizada, nesta pesquisa, documentação oficial ou paper que
descreva publicamente sua arquitetura, memória, custo ou defesas; esses detalhes
ficam **não verificados**.

### Como funciona o mecanismo relevante

O ponto útil a investigar é o gênero: fala aberta pode tentar persuadir NPCs.
Qualquer descrição técnica além disso é **não verificada** nesta fonte de
pesquisa; não deve fundamentar decisão de arquitetura.

### Aproveitar no ProcedWorld

- Usar conversas para sabor, contraproposta e explicação de personalidade,
  sempre sobre termos que o motor já conhece.
- Transformar afirmações do jogador em pedido estruturado, não em prova de que
  o fato ocorreu ou de que um NPC deve obedecer.

### Evitar no ProcedWorld

- Não supor que persona em prompt é controle de acesso.
- Não conceder alteração de estado por conversa persuasiva sem schema,
  autorização, capacidade e grounding.

### Pilar/SDD afetado

GDD [07-diplomacia.md](../gdd/07-diplomacia.md); SDD
[04-camadas-de-ia.md](../sdd/04-camadas-de-ia.md),
[07-diplomacia.md](../sdd/07-diplomacia.md) e
[14-testes.md](../sdd/14-testes.md).

## 1001 Nights / Book of Infinity

### O que é

1001 Nights se apresenta como projeto de pesquisa e jogos experimentais de
storytelling criativo com IA; seu site referencia o trabalho "Language as
Reality" em AIIDE-23. [F7] O mecanismo público detalhado não foi localizado
além dessa apresentação; pontos técnicos adicionais são **não verificados**.

### Como funciona o mecanismo relevante

A ideia de "linguagem como realidade" é um alerta de design: converter palavra
livre diretamente em objeto mecânico aproxima expressão e regra, mas amplia a
superfície para ambiguidade, exploração e inconsistência. A inferência de risco
é deste estudo; a implementação do projeto é **não verificada**. [F7]

### Aproveitar no ProcedWorld

- Permitir que narrativa nomeie e dramatize efeitos de templates, como uma seca
  ou uma promessa quebrada, depois que o motor os fixar.
- Exibir claramente a fronteira entre prosa e consequência: cartões de ação
  mostram custo, alvo, prazo e efeito antes de o comando entrar no log.

### Evitar no ProcedWorld

- Não transformar substantivos ou metáforas de chat diretamente em unidade,
  item, tratado ou mudança de tile.
- Não esconder uma decisão mecânica atrás de uma cena textual; o jogador deve
  poder auditar qual template e quais fatos a sustentaram.

### Pilar/SDD afetado

GDD [08-entropia-e-eventos.md](../gdd/08-entropia-e-eventos.md),
[11-experiencia-mobile.md](../gdd/11-experiencia-mobile.md); SDD
[04-camadas-de-ia.md](../sdd/04-camadas-de-ia.md) e
[18-dsl-catalogos.md](../sdd/18-dsl-catalogos.md).

## Inworld e NPCs com LLM

### O que é

Inworld é uma plataforma comercial de personagens/NPCs com IA; nesta pesquisa
não foi localizada documentação oficial suficiente para afirmar sua arquitetura
atual, limites de latência, precificação ou modelo de memória. Esses detalhes
são **não verificados**.

### Como funciona o mecanismo relevante

O padrão de produto de NPC conversacional exige separar persona, conhecimento,
memória e ação. A separação é uma recomendação de arquitetura deste estudo,
derivada dos riscos de texto não confiável e saídas livres, não uma afirmação
sobre a implementação atual da Inworld. [F10] [F11]

### Aproveitar no ProcedWorld

- Modelar persona como apresentação e postura; modelar fatos como visão do
  estado; modelar ação como intenção fechada e validada.
- Para cada personagem/bot, dar somente a visão permitida do mundo, evitando
  que Crônica global revele informação privada ou névoa de guerra.
- Manter um texto canônico de fallback para NPC/Entropia quando T2 expirar.

### Evitar no ProcedWorld

- Não usar fornecedor/SDK como fonte de autoridade, replay ou persistência
  canônica da partida.
- Não prometer conversa síncrona em toda tela mobile sem orçamento de prazo e
  plano de degradação.

### Pilar/SDD afetado

GDD [07-diplomacia.md](../gdd/07-diplomacia.md),
[09-governador-e-mandato.md](../gdd/09-governador-e-mandato.md); SDD
[04-camadas-de-ia.md](../sdd/04-camadas-de-ia.md),
[08-memoria-e-contexto.md](../sdd/08-memoria-e-contexto.md) e
[16-visibilidade.md](../sdd/16-visibilidade.md).

## Voyager — Minecraft

### O que é

Voyager é um agente de aprendizado contínuo para Minecraft guiado por LLM. O
artigo descreve currículo automático, biblioteca crescente de habilidades
executáveis e prompting iterativo com feedback de ambiente. [F6]

### Como funciona o mecanismo relevante

O agente obtém feedback de execução e erros, refina programas e armazena
habilidades interpretáveis/componíveis para reutilização. O artigo relata que a
biblioteca reduz esquecimento catastrófico no seu cenário experimental. [F6]

### Aproveitar no ProcedWorld

- Converter lições recorrentes em Doutrinas pequenas e versionadas, por exemplo
  preferência de resposta a escassez, mas apenas quando derivadas de resultados
  canônicos observáveis.
- Registrar resultado de intenção, motivo de rejeição e comando aplicado para
  que T1/T2 recebam feedback estruturado no próximo planejamento.
- Tratar a biblioteca como seleção de procedimentos aprovados pelo motor, não
  como código livre gerado pelo modelo.

### Evitar no ProcedWorld

- Não executar código, query ou script produzido por LLM no servidor.
- Não deixar uma regra "aprendida" alterar catálogo, fórmulas ou Mandato sem
  confirmação do jogador e versionamento explícito.
- Não comparar métricas de exploração em Minecraft com qualidade diplomática ou
  justiça de um 4X: objetivos e ambiente são diferentes.

### Pilar/SDD afetado

GDD [05-tecnologia.md](../gdd/05-tecnologia.md),
[09-governador-e-mandato.md](../gdd/09-governador-e-mandato.md),
[10-ciclo-infinito-e-eras.md](../gdd/10-ciclo-infinito-e-eras.md); SDD
[05-governador.md](../sdd/05-governador.md),
[08-memoria-e-contexto.md](../sdd/08-memoria-e-contexto.md) e
[14-testes.md](../sdd/14-testes.md).

## Diplomacia com LLM e benchmarks estratégicos

### O que são

Welfare Diplomacy adapta Diplomacy para recompensa de soma geral e mede
cooperação; seus baselines com LLM alcançaram bem-estar alto, mas foram
exploráveis no experimento reportado. [F8] DiploBench se descreve como um
testbed em andamento, não benchmark final, e ressalta que rodadas exigem muitas
chamadas de API para estabilizar resultados. [F9]

### Como funciona o mecanismo relevante

Esses ambientes combinam comunicação, compromisso, informação incompleta e
ações discretas. Eles são úteis para verificar coordenação e vulnerabilidade,
mas uma taxa de vitória isolada não mede explicabilidade, orçamento, replay ou
resistência a texto adversarial. [F8] [F9]

### Aproveitar no ProcedWorld

- Criar harness local com mapas/seed fixos, bots T0 e fixtures T1/T2 gravadas.
- Medir por cenário: validade de schema, referências de grounding, aceitação de
  tratado, custo/tokens/latência, fallback e hash de replay.
- Variar mapa, posição, postura e Mandato; trocar lados e repetir rodadas antes
  de concluir que uma política é melhor.
- Tratar cooperação como possível alvo mecânico do Ledger, não como confiança
  declarada na conversa.

### Evitar no ProcedWorld

- Não chamar uma grade de testes cara de benchmark estável sem repetições e
  controle de versão do modelo, prompt, regras e seed.
- Não usar vitória, elo ou texto julgado por LLM como único gate de qualidade.

### Pilar/SDD afetado

GDD [07-diplomacia.md](../gdd/07-diplomacia.md),
[12-variaveis-e-formulas.md](../gdd/12-variaveis-e-formulas.md); SDD
[07-diplomacia.md](../sdd/07-diplomacia.md),
[09-persistencia.md](../sdd/09-persistencia.md) e
[14-testes.md](../sdd/14-testes.md).

## Riscos transversais de produção

### Grounding, memória e reflexão

Todo fato que entra em uma intenção deve ser um ID/revisão visível ao ator. A
memória pode ajudar a recuperar por que o ator prefere algo, mas não prova que
o recurso existe, que uma fronteira é válida ou que a outra civilização aceitou
um acordo. Smallville inspira a camada de lembrança; CICERO inspira linguagem
controlada por plano; o estado canônico continua acima de ambos. [F1] [F3]

### Custo e latência

Não há número universal seguro: variam por modelo, provedor, prompt e momento.
Por isso, orçamento deve reservar pior caso antes da chamada e registrar uso
observado depois; ao exceder teto ou prazo, o turno usa T0. A observação de
DiploBench de que partidas completas precisam de muitas chamadas reforça testar
o custo de uma partida, não só de uma resposta isolada. [F9]

Para mobile, T2 deve ser assíncrono e opcional para prosa. A tela principal
precisa mostrar o resultado do motor imediatamente, com narrativa posterior
ou canônica se não houver resposta. A decisão de espera não pode depender de
relógio dentro de `step`.

### Prompt injection por jogadores

Prompt injection ocorre quando texto não confiável tenta substituir instruções
ou induzir ação não pretendida. [F10] A defesa não é um filtro único: a fonte
oficial recomenda separar conteúdo não confiável, limitar capacidade e usar
saída estruturada, além de avaliações e camadas de controle. [F10] [F11]

Aplicação concreta:

1. Serializar a mensagem em bloco delimitado `UNTRUSTED_PLAYER_DATA`; nunca
   concatená-la a instruções, Mandato, memória procedural ou schema.
2. Limitar tamanho, normalizar Unicode e recusar/ignorar campos de controle,
   links e alegações sem IDs verificáveis conforme política de produto.
3. Solicitar somente JSON Schema fechado; validar campos desconhecidos, actor,
   visibilidade, capacidades, custos, Mandato e `FactRef` fora do modelo.
4. Não fornecer tools, segredos, prompt de sistema, acesso de escrita nem
   comandos de motor a T2; a única passagem de efeito é `Intent → Command`.
5. Cobrir jailbreak, ID inventado, alteração de Mandato e pedido de segredo em
   fixtures adversariais sem rede; registrar apenas hashes/códigos redigidos.

Mesmo saída estruturada não torna uma intenção verdadeira: a documentação
ressalta que ainda é necessário tratar recusa, limite de tokens e erros do
modelo. [F11] Logo, a validação do motor é a última barreira, não o prompt.

### Determinismo e replay

Uma resposta de T1/T2 é uma entrada externa, não parte do cálculo puro. Antes
do `step`, normalizar a resposta aprovada e gravar comando, origem, versão de
schema/regras, `context_hash`, motivo de fallback e evidência redigida. O replay
aplica os comandos; não consulta o provedor nem reinterpreta a prosa.

T2 pode ser não determinístico sem comprometer o replay, desde que o comando
aceito esteja registrado. Em execução nova, variância é aceitável apenas dentro
de candidatos legais e orçamento; em replay, qualquer nova chamada de IA é bug.

## Recomendações para o ProcedWorld

1. Ratificar no SDD 04 que T2 gera plano/prosa e intenção tipada, enquanto T0
   calcula legalidade, aceitação diplomática e efeitos; T1 só escolhe entre IDs
   previamente filtrados.
2. Definir para cada `Intent.kind` fatos mínimos, visibilidade e revisão; tornar
   ausência, fato inexistente e fato incompatível motivos enumerados de rejeição.
3. Implementar a diplomacia em duas fases: termos estruturados/motor primeiro;
   interpretação e redação T2 como adaptação opcional, inspirada em CICERO.
4. Fixar documentos canônicos regeneráveis e compactação por era; a Crônica e a
   Doutrina devem carregar referências para estado/log e poder ser descartadas.
5. Criar desde o início `CallBudget` e `TurnBudget` com reserva prévia,
   timeout, teto por jogador/finalidade e fallback T0; medir antes de definir
   números de produto.
6. Criar corpus de fixtures adversariais de texto de jogador e gatear CI contra
   alteração de Mandato, criação de efeito e acesso indevido por injection.
7. Gravar `AcceptedCommand` para todo resultado de IA e exigir golden replay
   idêntico com sucesso, timeout, schema inválido e orçamento esgotado.
8. Dar à Entropia e ao Governador texto canônico de catálogo para que falha ou
   moderação de T2 não interrompa o turno nem esconda a causa mecânica.
9. Criar harness de diplomacia com side swap, seeds versionadas, relatórios de
   Ledger e métricas de custo/fallback; não usar taxa de vitória como único KPI.

## Fontes

- **[F1] Meta AI — CICERO: negociação, planejamento e diálogo controlável.**
  https://ai.meta.com/blog/cicero-ai-negotiates-persuades-and-cooperates-with-people/
- **[F2] Meta AI — página técnica e código do CICERO.**
  https://ai.meta.com/research/cicero/ e
  https://github.com/facebookresearch/diplomacy_cicero
- **[F3] Park et al. — Generative Agents: Interactive Simulacra of Human
  Behavior (paper, arXiv).** https://arxiv.org/abs/2304.03442
- **[F4] Latitude — Heroes Dev Log #10: memória e continuidade em AI Dungeon.**
  https://blog.latitude.io/heroes-dev-logs/10
- **[F5] Latitude — How Our Team Moderates Content on AI Dungeon.**
  https://content.latitude.io/blog/how-our-team-moderates-content-on-ai-dungeon/
- **[F6] Wang et al. — Voyager: An Open-Ended Embodied Agent with Large Language
  Models (paper, arXiv).** https://arxiv.org/abs/2305.16291
- **[F7] 1001 Nights — página do projeto e referência a AIIDE-23.**
  https://www.1001nights.ai/ e https://www.1001nights.ai/zh
- **[F8] Mukobi et al. — Welfare Diplomacy: Benchmarking Language Model
  Cooperation (paper, arXiv).** https://arxiv.org/abs/2310.08901
- **[F9] DiploBench — repositório e limitações declaradas do testbed.**
  https://github.com/sam-paech/diplobench
- **[F10] OpenAI — Safety in building agents: prompt injection e separação de
  dados não confiáveis.**
  https://developers.openai.com/api/docs/guides/agent-builder-safety
- **[F11] OpenAI — Structured model outputs: schema, recusas e casos-limite.**
  https://developers.openai.com/api/docs/guides/structured-outputs
