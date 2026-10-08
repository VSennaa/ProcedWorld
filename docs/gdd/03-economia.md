# 03 — Economia

## Decidido
- **Piso de produção da cidade** (decidido em 2026-10-07): o centro da cidade garante pelo menos
  1 de produção por turno, como no *Civilization*; foco em comida nunca trava a fila de produção.
- **Moeda e escambo**: riqueza é a moeda, e contratos podem ser **pacotes mistos livres** — bens e
  riqueza dos dois lados (ex.: 2 metal + 3 riqueza por 5 comida). O motor valida cada item; o Ledger
  registra o valor de referência em riqueza para comparação. Decidido em 2026-10-01.
- **Depósitos estratégicos finitos com renovação lenta por era** (ver 02-mapa-e-tiles.md §5).
  Decidido em 2026-10-01.

## Proposta

### Papel da economia no turno

- A economia transforma território e trabalho em escolhas de proteção: alimentar cidades, manter
  infraestrutura, guardar reservas ou sustentar acordos. Crescer aumenta capacidade e obrigações.
- Usar cinco rendimentos por hexágono trabalhado: **comida** (consumo e reserva), **produção**
  (construção e reparo), **riqueza** (moeda para manutenção e comércio), **conhecimento** (pesquisa)
  e **cultura** (ação social). Comida, produção e riqueza entram nas contas da cidade; conhecimento
  e cultura alimentam progresso com limites definidos nos seus pilares.
- Cada unidade de população disponibiliza um posto de trabalho. O jogador escolhe uma prioridade
  da cidade — abastecer, construir ou arrecadar — e o motor aloca postos nos hexágonos alcançáveis
  em ordem estável de rendimento relevante, com desempate por coordenada. A tela permite trocar
  um hexágono específico como exceção, sem distribuir todos os postos manualmente.
- Faixa base por hexágono trabalhado: 0–4 de comida, 0–3 de produção, 0–3 de riqueza, 0–2 de
  conhecimento e 0–2 de cultura por turno. **Valores iniciais, sujeitos a balanceamento.** Terreno,
  melhoria e evento compõem bônus aditivos; depois dos modificadores, cada rendimento fica em 0–6.
  O catálogo de [02-mapa-e-tiles.md](02-mapa-e-tiles.md) define combinações válidas de terreno e
  melhoria, sem multiplicadores cumulativos de eras.
- Decisão interessante: trabalhar uma várzea sustenta mais gente hoje; deslocar esse posto para
  uma oficina acelera reparos, mas pode consumir a reserva antes da próxima colheita. O resumo
  mostra o saldo previsto de comida, riqueza e manutenção após a escolha.

### Contas, sequência e privação

- Cada cidade guarda comida; riqueza entra no tesouro da civilização. Produção do turno pode ser
  aplicada a uma obra ou reparo em andamento, dentro de um limite de progresso por turno; sobra
  sem destino se perde. Conhecimento e cultura não viram itens negociáveis. Estoques nunca ficam
  negativos.
- Ordem econômica proposta na fase de resolução de [01-loop-e-turnos.md](01-loop-e-turnos.md):
  calcular rendimentos; pagar manutenção; reservar alimento local; liquidar contratos; consumir
  comida; aplicar obras e crescimento; registrar perdas e Ledger. As ordens aceitas no turno
  seguem a ordem do servidor; a liquidação usa contratos por ID estável.
- Demanda de comida por cidade = 1 por unidade de população por turno. Capacidade do celeiro =
  `3 × população + 4` unidades por cidade; excedente após consumo se perde. **Valores iniciais,
  sujeitos a balanceamento.** O jogador decide entre guardar alimento, exportar excedente previsto
  ou deslocar trabalho para outras atividades. Importação pode cobrir falta no mesmo turno.
- Manutenção base de uma cidade = `1 + teto(população / 4)` de riqueza por turno; cada melhoria
  ativa custa 1 de riqueza, e cada rota internacional ativa custa 1 por parte. **Valores iniciais,
  sujeitos a balanceamento.** Custo adicional de alcance = 1 por cidade separada da capital por
  mais de 4 hexágonos de território controlado. Distância e conexão vêm de
  [02-mapa-e-tiles.md](02-mapa-e-tiles.md); não escalam com o número da era.
- Pagamento segue prioridade fixa: sustento da cidade, reparos de segurança, melhorias e rotas;
  dentro da classe, ID estável. O jogador pode marcar melhoria ou rota para suspensão antes do
  turno. Obrigação sem saldo fica inadimplida, sem dívida automática infinita: a melhoria suspende
  seu bônus; a rota não entrega. Reativação exige pagar manutenção e reparar com produção.
- Decisão interessante: manter uma oficina cara preserva produção futura, enquanto suspendê-la
  libera riqueza para importar alimento. A interface mostra o efeito previsto antes de confirmar.
- Falta de comida é registrada por cidade; crescimento é bloqueado e a coesão sofre conforme
  [04-cidades-e-populacao.md](04-cidades-e-populacao.md) e
  [06-sociedade-e-governo.md](06-sociedade-e-governo.md). Para a pressão de crise de
  [00-visao.md](00-visao.md), a privação `D` é calculada por cidade (falta de comida mais
  manutenção essencial não paga) e agregada na civilização, conforme [12-variaveis-e-formulas.md](12-variaveis-e-formulas.md). A fome tem causa
  numérica rastreável.

### Recursos estratégicos e de luxo

- Depósitos têm tipo, estoque e taxa máxima de extração por turno, definidos na geração do mapa.
  Propor **metal** como estratégico inicial: cada posto alocado a um depósito extrai até 2 unidades
  por turno, limitado ao estoque. Construções avançadas gastam metal ao iniciar ou reparar; uma
  unidade militar equipada pode requerer 1 de metal por turno de reposição, conforme seu template.
  **Valores iniciais, sujeitos a balanceamento.** A população não precisa de metal para comer.
- Um depósito esgotado permanece como lugar histórico; novos depósitos ocultos surgem lentamente a
  cada era (02-mapa-e-tiles.md §5). Técnicas futuras podem ampliar a extração
  útil do estoque remanescente ou permitir substitutos definidos em catálogo, mas não recriam
  unidades gratuitamente (ver [05-tecnologia.md](05-tecnologia.md)).
- Propor **luxo** como categoria de bens com origem identificável, por exemplo especiarias de um
  bioma. Um depósito renovável entrega até 1 unidade por turno se trabalhado e regenera 1 unidade
  a cada 2 turnos, até sua capacidade original; uso contínuo pode exauri-lo. **Valores iniciais,
  sujeitos a balanceamento.** Uma unidade de luxo consumida
  pela civilização dá +2 de coesão naquele turno; segunda origem distinta dá +1; origens extras
  não dão bônus. **Valores iniciais, sujeitos a balanceamento.** Com D ≥ 15, o bônus fica suspenso.
- O jogador escolhe usar o luxo para coesão ou vendê-lo para financiar manutenção. Importar uma
  origem nova pode ajudar uma sociedade tensa, mas cria dependência de rota e parceiro.
- Estoque estratégico tem capacidade `4 × população total + 8` por civilização; luxo não consumido
  ou contratado perde validade ao fim do turno. **Valores iniciais, sujeitos a balanceamento.**
  A tela mostra estoque, extração restante e turnos estimados de uso.

### Contratos de comércio e Ledger de Relações

- Comércio entre civilizações usa contratos bilaterais de duração curta: origem, destino, **itens de
  cada lado** (comida, metal, luxo e/ou riqueza, por turno), duração e rota válida. Um contrato pode
  ser venda (bem por riqueza), escambo (bem por bem) ou pacote misto. Para comparar ofertas, o motor
  calcula um **valor de referência em riqueza** pela faixa de preço do catálogo. Oferta
  só vira contrato após aceitação; capacidade de transporte por rota = 3 unidades por turno.
  Duração proposta: 3 turnos, renovável explicitamente. **Valores iniciais, sujeitos a
  balanceamento.** Bens e riqueza mudam de dono, sem criar rendimento líquido por negociação.
- Na liquidação, o motor reserva comida suficiente para a demanda local do exportador; metal
  comprometido com reparos obrigatórios também fica reservado. Para cada item, entrega =
  `min(quantidade contratada, capacidade livre da rota, estoque disponível de quem entrega)`; a
  capacidade da rota conta só bens, não riqueza. Em pacote misto, se um lado entregar menos, o outro
  entrega na mesma proporção (arredondamento para baixo, inteiro). Faltas não geram bens ou moeda
  negativos. Fórmula exata de proporção fica para o SDD.
- Se comprador não puder pagar, a parte não entregue conta como inadimplência dele; se vendedor
  não tiver estoque exportável ou rota funcional, conta como inadimplência dele. Interrupção
  externa validada é registrada separadamente. Duas inadimplências da mesma parte encerram o
  contrato; não há juros automáticos. **Valor inicial, sujeito a balanceamento.**
- Cada oferta, aceite, entrega, entrega parcial, recusa, interrupção e encerramento produz entrada
  com turno, IDs das partes, contrato, quantidade prometida e realizada, preço e causa no
  **Ledger de Relações** (ver [07-diplomacia.md](07-diplomacia.md)). Propor +1 de confiança por
  contrato concluído, até 3 por era, e −2 por inadimplência imputável; ambos decaem conforme
  regra geral do Ledger. **Valores iniciais, sujeitos a balanceamento.** O motor calcula aceitação
  usando estado diplomático, oferta e histórico; confiança não garante aceitação automática.
- Decisão interessante: comprar comida evita privação agora, mas consome riqueza que manteria
  melhorias; vender metal paga custos, porém atrasa defesa. A tela oferece até três acordos com
  saldo previsto, duração e risco de rota, aceitos em poucos toques.
- Bots podem propor preço, bem e prazo dentro das faixas do catálogo e justificar a intenção com
  falta, excedente, rota e entradas específicas do Ledger. O motor valida oferta e calcula
  aceitação para bots; justificativa e texto do LLM não alteram entrega nem confiança. Ajuda sem
  contrapartida é contrato distinto de preço zero, com limite por era e causa registrada.

### Governador, Entropia e reação do jogador

- O Mandato permite limites separados para gasto em importação por turno, reserva mínima de
  comida, bens exportáveis e parceiros proibidos (ver
  [09-governador-e-mandato.md](09-governador-e-mandato.md)). O Governador propõe realocar postos,
  suspender manutenção ou negociar dentro desses limites; o motor bloqueia violações.
- Fallback T0 propõe primeiro evitar falta de comida prevista, depois pagar manutenção essencial,
  depois preservar a reserva do Mandato. Empates usam IDs estáveis. Não compra de parceiro
  proibido nem excede orçamento; se não houver saída, registra privação e apresenta ao jogador a
  decisão de maior impacto no próximo acesso.
- A Entropia pode propor um template de seca para hexágonos expostos ou bloqueio para rota
  vulnerável, com duração e intensidade limitadas pelo catálogo e orçamento de tensão (ver
  [08-entropia-e-eventos.md](08-entropia-e-eventos.md)). O motor verifica elegibilidade e reduz
  rendimento ou capacidade pela regra do template. O relatório liga evento, perda e resposta.
- Em tela pequena, o resumo mostra turnos de alimento, riqueza líquida prevista e contratos em
  risco. Uma crise oferece até três ações: realocar trabalho, usar reserva ou negociar/suspender
  custo. Detalhes da cidade e do Ledger ficam a um toque adicional; rotinas podem ser delegadas,
  conforme [11-experiencia-mobile.md](11-experiencia-mobile.md).

### Eras, colapso e renascimento sem inflação

- Avanços de era liberam combinações de trabalho, substitutos e obras, sem multiplicar todos os
  rendimentos. O limite por hexágono continua 6 por categoria; melhorias disputam espaço,
  manutenção e produção de reparo. População limita quantos hexágonos rendem.
- Riqueza acumulada por civilização tem limite `6 × população total + 12`; acima dele, o excedente
  não rende juros e se perde ao fechar o turno. Comida e metal têm os limites indicados acima.
  **Valores iniciais, sujeitos a balanceamento.** Preço de contratos fica na faixa catalogada de
  1–5 por unidade. Não há reajuste automático por era ou custo proporcional à riqueza guardada.
- Mais cidades exigem manutenção e aumentam alcance; rotas consomem capacidade e custam para
  funcionar. Um império grande pode ter mais opções sem gerar excedente ilimitado por inércia.
  Concentração em uma fonte cria risco de seca, esgotamento ou bloqueio comercial.
- Falhas sucessivas de abastecimento e manutenção pressionam coesão e podem contribuir para
  colapso parcial nos termos de [00-visao.md](00-visao.md) e
  [10-ciclo-infinito-e-eras.md](10-ciclo-infinito-e-eras.md). O colapso fragmenta titularidade
  de cidades, estoques e depósitos por regras de sucessão, sem duplicar inventários ou contratos.
- O jogador escolhe a comunidade sucessora. Ela recebe somente estoques das cidades sob seu
  controle, até os novos limites; contratos anteriores terminam e deixam entradas no Ledger.
  Ruínas, técnicas preservadas e antigas obrigações podem abrir opções de renascimento, mas
  exigem trabalho, confiança ou recursos para reativar.
- Em cada era, medir saldo médio por população, turnos de reserva, inadimplência e recuperação
  após colapso. Balanceamento muda catálogos versionados entre partidas ou por migração explícita,
  nunca um multiplicador oculto aplicado à partida em andamento.

### Fronteira entre regras e IA

- **Determinístico:** alocação após escolha de prioridade, rendimentos, limites, manutenção,
  reservas, consumo, extração, contratos, Ledger, privação, sucessão econômica e validação do
  Mandato. Aritmética inteira, ordem estável e catálogos versionados seguem ADR-0006.
- **Intenções de IA:** Governador escolhe prioridades e contratos; bots propõem e respondem a
  ofertas; Entropia escolhe template elegível e parâmetros limitados. Cada intenção tipada cita
  fatos do estado e, quando diplomática, IDs do Ledger. O motor aceita ou rejeita, grava o comando
  e aplica só efeitos previstos; texto narrativo não é comando.
- Timeout ou intenção inválida aciona fallback determinístico T0. Replay usa comandos gravados,
  sem nova chamada à IA; mesmo estado, comandos e seed produzem o mesmo saldo e hash.

## Perguntas abertas
- Como evitar que a economia "exploda" num jogo infinito (inflação de rendimentos, teto, manutenção crescente)?
  - **Recomendação:** testar em simulações longas limites por hexágono e estoque, manutenção por
    cidade/melhoria/rota e ausência de multiplicadores de era. Ajustar faixas pelos saldos por
    população e tempo de recuperação, sem escalonar custos só porque o turno avançou.
