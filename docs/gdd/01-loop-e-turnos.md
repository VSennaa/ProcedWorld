# 01 — Loop principal e turnos

## Decidido
- Turnos simultâneos; ações aplicadas sequencialmente na ordem aceita pelo servidor (ADR-0003).
- **Sem relógio** (ADR-0008): o jogo não é em tempo real; o turno é a jogada. O turno avança quando
  todos os humanos **presentes** jogaram. Para o humano **ausente**, o Governador joga na hora.
- **Combate em fase própria** (aprovado em 2026-10-01): declarar ataque reserva a unidade e o custo; o
  dano é calculado depois, com perdas simultâneas por confronto.
- **Ordem de bots e Governadores** (decidido em 2026-10-01): definida pela **seed do mundo**, fixada no
  início da partida, que determina a ordem e o momento de envio de cada civilização automatizada, com
  rotação entre turnos. Reprodutível e sem prioridade fixa para ninguém.
- Fase de resolução depois das ações: produção, crescimento, Entropia, consolidação de memória.

## Proposta

### Três ritmos, uma escolha recorrente

- **Turno, loop curto:** ler mudança, causa e o que vence em quantos turnos; escolher uma resposta à maior
  pressão; ajustar uma prioridade de cidade ou ordem de unidade; responder a uma oferta; tocar em
  **Pronto**. Proteger abastecimento pode adiar uma obra ou deixar a fronteira exposta. O jogador
  decide qual compromisso aceita, não precisa distribuir cada unidade todo turno.
- **Era, loop médio:** no marco de era, examinar coesão, território e relações; escolher foco e
  renúncia, rever o Mandato e confirmar compromissos. Trocar eficiência presente por resiliência
  conecta este loop ao foco proposto em [00-visao.md](00-visao.md) e ao ciclo em
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md).
- **Ciclo, loop longo:** atravessar ascensão, auge, crise, possível colapso e renascimento.
  Preservar uma instituição, comunidade ou tratado importa mais que acumular pontos. No colapso,
  o jogador escolhe uma comunidade sucessora; a Crônica guarda causas e legado.
- A primeira tela da sessão mostra **mudança → causa → em quantos turnos vence → até três respostas**. Cada resposta
  mostra custo imediato, risco e efeito esperado em coesão ou pressão. Mapa e Ledger ficam a um
  toque. Meta de sessão de 2–8 minutos e até oito toques por decisão crítica: valores iniciais,
  sujeitos a balanceamento com [11-experiencia-mobile.md](11-experiencia-mobile.md).

### Presença e prontidão

- Não existe prazo em tempo real. O servidor abre o turno com o estado publicado e espera apenas os
  humanos **presentes** (com sessão ativa no mundo). `AbrirTurno`, comandos aceitos e `FecharTurno`
  entram no log; o `step` recebe a sequência, a seed e as versões de catálogo, sem consultar hora nem
  chamar IA (ADR-0006).
- O jogador pode confirmar um lote de ordens em poucos toques. O servidor valida cada comando e
  grava a sequência de aceitação; o lote não recebe prioridade especial. Ordens rejeitadas mostram o
  motivo antes de Pronto e não cobram custo.
- **Pronto** encerra a edição do jogador naquele turno. Pode ser desfeito enquanto o turno não fechou;
  comandos já aceitos permanecem e novas ordens entram no fim do log. Quem não quer intervir envia
  **Manter plano** como comando explícito.
- Quando todos os humanos presentes estão Prontos e as civilizações automatizadas enviaram suas
  intenções, o turno fecha. Humano ausente não é esperado: o Governador joga por ele, dentro do
  Mandato. Timeout de IA usa T0 sem segurar o turno. O relatório seguinte distingue ordens humanas,
  delegadas e automáticas.
- Proposta para o SDD: se a conexão cair no meio do turno, comandos já aceitos valem e o jogador passa
  a ausente; o Governador completa apenas o que faltar.
- Prazos dentro do jogo (tratados, propostas, crises, obras) são contados **em turnos**, nunca em horas.

### Fases e ordem de resolução

| Fase | Regra proposta | Decisão do jogador |
|---|---|---|
| 1. Abertura | Publicar estado, relatório causal e ofertas; renovar pontos de ação pelas regras. | Qual risco atender ou delegar. |
| 2. Entrada | Validar comandos humanos e intenções de bots/Governadores; aplicar ações sequenciais na ordem aceita. | Gastar capacidade em movimento, obra, política ou negociação. |
| 3. Fechamento | Todos os humanos presentes Prontos; Governador joga pelos ausentes, T0 se preciso; gravar `FecharTurno`. | Confirmar ordens ou Manter plano e marcar Pronto. |
| 4. Conflitos | Resolver ataques declarados e efeitos pendentes de tratados em ordem canônica. | Atacar, defender ou negociar com custo conhecido. |
| 5. Sustento | Calcular manutenção, produção, consumo, crescimento e coesão após conflitos. | Ver se a proteção escolhida bastou. |
| 6. Entropia | Validar evento elegível contra estado pós-sustento e orçamento de tensão; aplicar template. | Responder no turno seguinte, se houver escolha. |
| 7. Síntese | Calcular pressão, verificar colapso/marco de era, consolidar memória e gerar Crônica. | No acesso seguinte, escolher resposta ou rumo. |

- A fase 2 preserva a ordem de aceitação exigida pelo ADR-0003. Na fase 4, **declarar ataque**
  aplica de imediato custo e reserva da unidade; dano e ocupação são calculados no fechamento.
  É a exceção delimitada que impede vitória automática do primeiro atacante, sem retirar a
  sequência registrada dos comandos.
- Resolução interna usa ID estável de hexágono, depois ID estável de confronto; empates usam PRNG
  versionado. Ordem de iteração de coleção nunca decide resultado. Comando que perdeu elegibilidade
  por ação anterior é rejeitado antes de entrar no log de comandos aceitos.
- Sustento liga consumo e abastecimento à privação, coesão à tensão e ameaça à pressão `P` de
  [00-visao.md](00-visao.md), com regras de [03-economia.md](03-economia.md) e
  [04-cidades-e-populacao.md](04-cidades-e-populacao.md). A Entropia recebe o estado pós-sustento e
  só seleciona templates elegíveis de [08-entropia-e-eventos.md](08-entropia-e-eventos.md).

### Conflitos de ordem e combate

- Movimento para hexágono livre, vaga de obra e gasto de reserva são sequenciais. Se dois comandos
  disputam a mesma capacidade, vale o primeiro aceito; o segundo é rejeitado sem custo e com causa
  legível. Esperar tem risco concreto, visível antes de uma nova confirmação.
- Entrar em hexágono hostil exige **Atacar**. O comando indica atacante, alvo e postura
  (`cautelosa` ou `decisiva`). Na aceitação, o motor verifica alcance, estado diplomático, pontos
  de ação e Mandato; gasta um ponto e reserva a unidade. Cada unidade entra em no máximo um
  confronto por turno. Alcance e custos vêm de regras de dados validadas.
- Ataques ao mesmo hexágono formam um confronto. No fechamento, o motor congela participantes
  válidos, calcula força a partir de terreno, suprimento e postura, determina perdas de ambos os
  lados sobre esse quadro único e só depois altera unidades e controle. Postura cautelosa limita
  perdas e chance de tomada; decisiva aumenta ambas. Modificador inicial de força de −20% a +20%
  por postura e −25% a +25% por terreno: valores iniciais, sujeitos a balanceamento com
  [02-mapa-e-tiles.md](02-mapa-e-tiles.md).
- Fórmula inicial por lado: `F = soma(força_base × (100 + bônus_total) / 100)` em inteiros;
  `perda = min(PV_total, limitar(5, 40, arred(30 × F_hostil / (F_própria + F_hostil))))`.
  Bônus total fica entre −40% e +40%; perdas são simultâneas, distribuídas entre unidades por ID
  estável. Esses números são valores iniciais, sujeitos a balanceamento. Controle muda apenas se
  uma coalizão hostil elegível restar no hexágono; se ambas sobreviverem, o defensor mantém controle.
- Se o alvo esvaziar antes dos conflitos, um ataque válido só ocupa o hexágono se postura e
  suprimento permitirem. Múltiplos atacantes disputam pelo mesmo cálculo de força, com empate pela
  seed. Se paz ou perda de alcance tornar o ataque inelegível, não há dano nem ocupação; custo de
  preparação permanece. A interface explicita esse risco antes da declaração.
- Tratados aceitos na fase 2 podem afetar ordens posteriores. Os efeitos que alteram hostilidade
  entram antes do dano na fase 4: uma paz pode invalidar ataques já declarados. Proposta, aceitação
  e efeito ficam no Ledger, conforme [07-diplomacia.md](07-diplomacia.md). O relatório explica
  modificadores, perdas e controle; texto de IA nunca decide o resultado.

### Bots, Governador e ausência

- No início do turno, bot e Governador delegado recebem o mesmo estado publicado e opções válidas.
  Suas intenções citam IDs do estado, Mandato quando houver e entradas do Ledger em diplomacia.
  O motor revalida a intenção no instante da aceitação, pois o mundo pode ter mudado.
- A ordem e o momento de envio de cada civilização automatizada (bots e Governadores) são derivados
  da **seed do mundo**, fixada no início da partida, com rotação entre turnos. Exemplo de regra para o
  SDD: uma permutação sorteada pela seed define a ordem inicial e cada turno a desloca em uma posição.
  O motor revalida cada intenção no instante da aceitação.
- A rotação distribui a vantagem de primeira aceitação entre bots, sem esconder a precedência de
  um humano que envia cedo. Confrontos seguem a fase própria. Métricas devem medir primeira
  aceitação, ganhos de território e rejeições por civilização para avaliar justiça do ritmo.
- Jogador ausente sem comando válido recebe plano mínimo do Governador no fechamento: evitar perda
  irreversível, manter abastecimento e honrar obrigações. Nova guerra, cessão de território e gasto
  acima do Mandato são bloqueados. Se o jogador deu ordens, o Governador só completa áreas
  previamente delegadas; ver [09-governador-e-mandato.md](09-governador-e-mandato.md).
- Ausência longa não acumula ordens ou recursos sem limite. O Governador paga manutenção normal;
  notificações agrupam crises e ofertas que vencem em poucos turnos. No retorno, o jogador vê resumo causal desde a
  última visita e pode rever o Mandato antes do próximo fechamento. Oferta vencida segue a regra
  de validade, e aceitação delegada ainda exige permissão do Mandato e cálculo do motor.
- Bots diplomáticos escolhem proposta e texto com fatos do Ledger. Máquina de estados, aceitação
  e efeitos pertencem ao motor; ofensa, promessa ou auxílio só mudam relações por comando válido.
  A decisão interessante ao jogador é aceitar benefício imediato com obrigação futura ou recusar
  e assumir risco de tensão, não adivinhar uma vontade arbitrária do modelo.

### Escala, crise e renascimento

- A estrutura do turno persiste em todas as eras. Mudam opções e curva de tensão, sem multiplicador
  gratuito por era. O marco abre foco e renúncia, enquanto território e instituições extensos
  aumentam manutenção. Alvo inicial de 6–8 turnos por era: valor inicial, sujeito a balanceamento
  e aos marcos de [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md).
- Na síntese, o motor calcula `P` de [00-visao.md](00-visao.md). `P` de 40–69 pede mitigação;
  `P` de 70–100 por dois turnos abre crise. A Entropia pode propor evento elegível dentro do
  orçamento, mas não provocar colapso só por narrativa. O jogador escolhe consumir reservas,
  negociar ajuda ou aceitar perda local para recompor coesão.
- Coesão em 0 por dois turnos, ou outra condição validada de sociedade, inicia colapso parcial na
  síntese. A abertura seguinte apresenta comunidades sucessoras com recursos e obrigações; o
  jogador escolhe qual acompanhar. Na ausência, Mandato ou T0 preserva mais população e coesão.
- Renascimento preserva legado limitado de instituições, conhecimento e relações, com custos e
  reivindicações. Reservas têm teto ligado à capacidade real; excedente paga manutenção ou se
  perde por regra econômica. Vagas limitadas de legado impedem bônus acumulativo por ciclo. Assim
  proteger uma instituição compete com defender território e pessoas agora.
- Depois de centenas de turnos, o relatório ainda mostra até três causas prioritárias, enquanto
  Crônica resume períodos e o mapa destaca pressão local. A interface corta ruído, não cálculos
  nem custos; ver [11-experiencia-mobile.md](11-experiencia-mobile.md).

### Motor determinístico e intenções de IA

- **Motor:** validação, ordem aceita, custos, movimento, combate, tratados, produção, pressão,
  elegibilidade de evento, era e colapso são regras determinísticas. Log contém comandos, seed,
  versões de regras e hash por turno; replay não chama IA (ADR-0006).
- **IA:** Governador e bots propõem ordens tipadas; Entropia propõe template elegível, parâmetros
  limitados e texto. Modelo não altera estado, calcula dano final ou decide aceitação diplomática.
  Intenção sem fundamento rastreável, inválida ou atrasada é descartada; T0 produz intenção válida
  ou Manter plano. Só comandos convertidos pelo motor têm efeito mecânico.

## Perguntas abertas

- Definição exata de "presente" e de queda de conexão no meio do turno (SDD, protocolo).
- Como a seed posiciona os envios automatizados em relação aos humanos (antes, intercalado, depois)?
  - **Recomendação:** a seed define, por civilização automatizada, uma posição numa sequência que
    inclui a abertura e o fechamento do turno; validar no harness de simulação se algum bot ganha
    disputas de forma desproporcional.
