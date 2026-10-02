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
| **T0 / T1 / T2** | Camadas de decisão: heurística / modelo de decisão (Jev→Laya) / LLM barato. |
| **BYOK** | *Bring your own key*: o jogador informa a própria API key de LLM. |
| **Saturação** | Projeção de tokens do contexto de uma IA ultrapassando o orçamento; dispara compactação ou reset. |
