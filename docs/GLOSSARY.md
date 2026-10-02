# Glossário

| Termo | Definição |
|---|---|
| **Civilização** | Sociedade controlada por um jogador ou bot; unidade principal de estado do jogo. |
| **Turno** | A jogada: unidade atômica da simulação. Todos jogam o mesmo turno; avança quando todos os humanos presentes jogaram, sem relógio (ADR-0003, ADR-0008). |
| **Presente / ausente** | Humano conectado ao mundo naquele turno / não conectado. Pelo ausente, o Governador joga na hora (ADR-0008). |
| **Magia** | Fenômeno que pode surgir naturalmente no mundo, decidido pela Entropia, oculto ou intrínseco a sociedades; as nações reagem (aderir, proibir, regulamentar), com consequências (pilar 00). |
| **Fase de resolução** | Parte do turno após as ações: produção, crescimento, Entropia, consolidação. |
| **Era** | Agrupamento de turnos com consolidação de memória e mudança da curva da Entropia. |
| **Ciclo infinito** | Não há vitória obrigatória; civilizações ascendem, colapsam e renascem. |
| **Governador** | Bot do jogo que decide por uma civilização dentro do Mandato (ADR-0002). |
| **Mandato** | Configuração do jogador que restringe o Governador: prioridades, proibições, limites, notificações. |
| **Entropia** | IA diretora de eventos procedurais do mundo, com orçamento de tensão. |
| **Template de evento** | Definição de dados de um evento com efeitos mecânicos validados e parâmetros limitados. |
| **Ledger de Relações** | Registro auditável das interações entre duas civilizações, com decaimento. |
| **Crônica** | História compactada do mundo ou de uma civilização; contexto de IA e conteúdo para o jogador. |
| **Doutrina** | Regras procedurais consolidadas que o Governador aprendeu (camada `procedural`). |
| **Intenção** | Proposta tipada de ação vinda de uma IA, antes da validação do motor. |
| **Comando** | Ação validada, gravada no log e aplicada pelo `step`. |
| **AcceptedCommand** | Contrato canônico do comando aceito, com ator, origem, payload, versões e referências de grounding; é a entrada mecânica do replay (proposta de SDD). |
| **accepted_sequence** | Sequência atribuída pelo servidor, única globalmente no mundo, que ordena os comandos aceitos no log (decisão de nomenclatura RC-02). |
| **GroundingRef** | Referência tipada a um fato do estado (`kind`, `id`, `revision`) que permite auditar a justificativa de uma ação (decisão de nomenclatura RC-02). |
| **DomainEvent** | Fato emitido pelo motor como resultado de uma transição; `LedgerEntry` é uma projeção/fato diplomático associada a ele ou ao comando causal (decisão de nomenclatura RC-02). |
| **StateHash** | Hash do estado autoritativo, serializado no campo `state_hash`, usado para verificar determinismo e divergências (decisão de nomenclatura RC-02). |
| **WorldSnapshot** | Snapshot autoritativo do mundo usado para recuperação; distinto de `ClientViewSnapshot`, que é uma projeção para o cliente (decisão de nomenclatura RC-02). |
| **RulesetRef** | Referência a um conjunto de regras por `id`, `version` e `content_hash` (decisão de nomenclatura RC-02). |
| **IntentEvidence** | Evidência auditável de uma resposta ou intenção de IA; pode ser associada ao comando, mas não o substitui nem altera seu efeito no replay (proposta de SDD, alinhada à correção RC-03). |
| **PresencePolicy** | Política de presença e prontidão da sessão; contrato detalhado ainda é proposta, embora a janela técnica de reconexão de 60 s e o registro dos presentes na abertura do turno estejam decididos (RC-08). |
| **SnapshotPolicy** | Política versionada que determina snapshots no início do mundo, no fim de cada era e a cada 50 turnos (decisão do usuário, RC-05). |
| **Legitimidade (`L`)** | Variável de 0 a 100 no escopo da civilização, definida pelo GDD 06 e usada em regras de sociedade e governo (GDD 12). |
| **Estabilidade local (`S`)** | Variável de 0 a 100 no escopo da cidade; participa da pressão de crise com peso reduzido conforme decisão do usuário (GDD 12). |
| **Pressão de crise (`P`)** | Variável de 0 a 100 calculada por cidade (`P_c`) e por civilização (`P_civ`), com fórmula compartilhada e limiares descritos no GDD 12; os pesos são sujeitos a balanceamento, salvo decisões explícitas. |
| **Prática** | Capacidade tecnológica ativa que tem custo de sustentação; há limite decidido de até 3 práticas ativas por civilização, com valor exato sujeito a balanceamento (GDD 05). |
| **Legado** | Marca persistente e escassa preservada entre eras/colapsos; o limite decidido é de até 2 legados ativos no total, incluindo tecnologia preservada (GDD 05, 10 e 12). |
| **Fenômeno mágico** | Fenômeno que pode surgir naturalmente no mundo por template da Entropia; sociedades podem reagir a ele por adesão, proibição ou regulamentação (GDD 00 e 06). |
| **Interferência na Entropia** | Termo para ação que altera ou condiciona eventos da Entropia; definição mecânica não localizada como decisão e permanece proposta, sem efeito canônico estabelecido. |
| **T0 / T1 / T2** | Camadas de decisão: heurística / modelo de decisão (Jev→Laya) / LLM barato. |
| **BYOK** | *Bring your own key*: o jogador informa a própria API key de LLM. |
| **Saturação** | Projeção de tokens do contexto de uma IA ultrapassando o orçamento; dispara compactação ou reset. |
