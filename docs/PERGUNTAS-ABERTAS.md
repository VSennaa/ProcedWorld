# Perguntas abertas consolidadas

Consolidação das perguntas abertas de GDD, SDD, `docs/STATUS.md` e ADRs ainda propostos ou parcialmente abertos. Recomendações são propostas para orientar a decisão do dono do projeto; não representam decisões aprovadas.

## Bloqueiam a aprovação do SDD

Prioridade ordenada pelo que impede fechar contratos ou aprovar a especificação. GDD aprovado em 2026-10-01; perguntas de produto remanescentes ficam na seção própria.

1. **Qual política de backup será adotada?** Bloqueia SDD.
2. **Quais limites operacionais e gates de promoção serão adotados?** Bloqueia SDD.
3. **Qual janela, fuso e moeda definem o teto diário de custo?** Bloqueia SDD.
4. **Como medir e testar resistência a prompt injection?** Bloqueia SDD.
5. **Qual contrato de transporte e geração/versionamento de tipos será usado?** Bloqueia SDD.

## Design de jogo

### Unidades no turno do jogador (levantado pelo usuário em 2026-10-03)

- **Pergunta:** O jogador precisa dar ordem a cada unidade antes de marcar Pronto (estilo *Civilization*), e quais tipos de unidade civil existem (construtor, caçador, pesquisador)?
- **Origens:** feedback do usuário sobre o cliente rascunho; GDD 01 — Loop e turnos (hoje diz que o jogador "não precisa distribuir cada unidade todo turno"); catálogo `core.units`.
- **Por que importa:** Contradiz a regra atual do GDD 01 e muda a duração da sessão mobile; produção de unidades seguiria a lógica de fila da cidade do *Civilization*.
- **Opções:** (1) Pronto bloqueado até toda unidade ter ordem, com atalhos "fortificar/pular/repetir" e ordens persistentes; (2) só unidades ociosas sem ordem persistente bloqueiam; (3) manter a regra atual (Governador completa ordens faltantes).
- **Recomendação:** (2): unidades com ordem de vários turnos (construir, explorar, fortificar) não pedem atenção; só ociosas bloqueiam, com "pular" de um toque. Avaliar caçador e pesquisador como papéis novos no catálogo.
- **Bloqueia:** revisão do GDD 01 e do cliente (unidades no mapa).

### Árvore de pesquisa no cliente (levantado pelo usuário em 2026-10-03)

- **Pergunta:** A tela de tecnologia entra já no cliente mínimo, mesmo só para visualização?
- **Origens:** feedback do usuário; GDD 05 — Tecnologia; catálogo `core.tech_tree`.
- **Opções:** (1) Tela só leitura agora, interação depois; (2) esperar a fase Beta.
- **Recomendação:** (1).
- **Bloqueia:** nada; entra no backlog do cliente.

### Economia em ciclo infinito

- **Pergunta:** Como evitar inflação de rendimentos e estoques ao longo de uma partida sem fim?
- **Origens:** GDD 03 — Economia.
- **Por que importa:** Sem limites sustentáveis, a economia pode perder escolhas relevantes ou tornar custos irrelevantes.
- **Opções:** (1) Limites por tile e estoque; (2) manutenção crescente por cidade, melhoria e rota; (3) combinação de limites e manutenção, sem multiplicadores automáticos por era.
- **Recomendação:** Simular limites de produção e estoque com manutenção recorrente; calibrar por saldo/população e tempo de recuperação, sem escalar apenas pelo avanço do turno.
- **Bloqueia:** aprovação do GDD.

### Revoltas

- **Pergunta:** Quais limiares separam revolução moderada de severa?
- **Origens:** GDD 04 — Cidades e população; GDD 06 — Sociedade e governo.
- **Por que importa:** Conecta políticas de governo à estabilidade, população e distribuição territorial.
- **Opções:** limiares fixos, faixas graduais ou avaliação por múltiplos fatores.
- **Recomendação:** Permitir restrições legais com efeitos e custos visíveis; definir severidade por faixas de indicadores observáveis, calibradas no harness.
- **Bloqueia:** aprovação do GDD.

### Ordem de ações automatizadas

- **Pergunta:** Como a seed posiciona os envios automatizados em relação às ações humanas?
- **Origens:** GDD 01 — Loop e turnos; ADR-0003 — Turnos simultâneos; ADR-0008 — Turno sem relógio.
- **Por que importa:** A ordem pode dar vantagem competitiva recorrente a bots ou Governadores.
- **Opções:** (1) Bots antes dos humanos; (2) bots depois; (3) ordem rotativa por civilização e seed, incluindo abertura e fechamento.
- **Recomendação:** Usar uma ordem determinística rotativa derivada da seed e medir no harness se há vantagem desproporcional.
- **Bloqueia:** aprovação do GDD.

### Presets do Mandato

- **Pergunta:** Quais são os valores exatos dos presets e limiares de aviso do Governador?
- **Origens:** GDD 09 — Governador e Mandato.
- **Por que importa:** Presets afetam decisões automáticas, notificações e previsibilidade para o jogador.
- **Opções:** (1) Definir todos antes da aprovação; (2) aprovar categorias e calibrar valores no balanceamento.
- **Recomendação:** Aprovar categorias e semântica agora; registrar valores iniciais como parâmetros calibráveis no harness.
- **Bloqueia:** nenhuma; valores podem ser calibrados na implementação, desde que o GDD fixe a semântica.

### Assumir civilização automatizada

- **Pergunta:** É necessária uma restrição adicional para um jogador assumir controle de uma civilização conduzida por bot, como aguardar N eras?
- **Origens:** GDD 10 — Ciclo infinito e eras.
- **Por que importa:** Afeta continuidade da partida, equilíbrio e expectativa de outros participantes.
- **Opções:** (1) Assunção imediata; (2) período de carência; (3) apenas por convite ou vaga aberta.
- **Recomendação:** Permitir assunção imediata em vaga aberta, com transição de Governador preservada no histórico e notificação aos participantes.
- **Bloqueia:** nenhuma.

## IA

### Hospedagem do Laya

- **Pergunta:** Onde será hospedado o Laya na migração de Jev, considerando que a VPS de referência não tem GPU?
- **Origens:** STATUS — Perguntas abertas; ADR-0005 — hospedagem do Laya em aberto; ADR README — ADR-0005.
- **Por que importa:** Determina latência, custo, operação e requisitos de hardware da porta de decisão.
- **Opções:** (1) Laya via Runware; (2) servidor dedicado com GPU; (3) CPU em host maior, após medição; (4) manter Jev até existir alternativa viável.
- **Recomendação:** Adiar a escolha até spike comparativo de custo/latência; manter `DecisionPort` e Jev funcionando como configuração inicial.
- **Bloqueia:** nenhuma para GDD/SDD inicial; bloqueia a migração Jev → Laya.

### Crivo contra prompt injection

- **Pergunta:** Como medir e testar a resistência a prompt injection em negociações em linguagem natural?
- **Origens:** GDD 07 — Diplomacia.
- **Por que importa:** Texto malicioso não pode converter-se em instrução privilegiada nem em efeito mecânico.
- **Opções:** (1) Fixtures adversariais versionadas e critérios de rejeição; (2) avaliação manual; (3) conjunto automatizado mais revisão periódica.
- **Recomendação:** Fixar fixtures gravadas com casos de jailbreak, conteúdo citado e tentativa de alterar estado; exigir que toda saída passe por validação tipada e grounding.
- **Bloqueia:** aprovação do SDD.

## Técnica e stack

### Protocolo cliente-servidor

- **Pergunta:** O protocolo entre Godot e servidor usará JSON ou formato binário, e como os tipos serão gerados e versionados?
- **Origens:** ADR-0007 — pontos a validar antes de aceitar; ADR-0001 — compatibilidade de protocolo.
- **Por que importa:** Afeta compatibilidade cliente-servidor, tamanho de payload e manutenção dos contratos.
- **Opções:** (1) JSON com schemas versionados; (2) binário com schemas e geração de tipos; (3) JSON inicial com migração documentada se medições exigirem.
- **Recomendação:** Começar com JSON tipado e versionado; medir payload e custo antes de adotar binário.
- **Bloqueia:** nenhuma para GDD; decisão deve constar do SDD.

## Infra e segurança

### Teto diário de custo

- **Pergunta:** Qual é a janela do teto diário, fuso horário, moeda exibida e conversão entre provedores?
- **Origens:** GDD 11 — Experiência mobile.
- **Por que importa:** Teto ambíguo pode interromper chamadas antes do esperado ou tornar custos incompreensíveis.
- **Opções:** (1) Janela UTC fixa; (2) dia civil no fuso do jogador; (3) janela móvel de 24 horas.
- **Recomendação:** Usar janela móvel de 24 horas para enforcement e exibir estimativa em moeda de referência com câmbio/provedor claramente indicado.
- **Bloqueia:** aprovação do SDD.

### Proxy e certificados TLS

- **Pergunta:** Qual proxy reverso e estratégia de certificados TLS serão usados?
- **Origens:** SDD 13 — Infra e deploy.
- **Por que importa:** Protege o tráfego cliente-servidor e define renovação, health checks e operação.
- **Opções:** (1) Proxy maduro já disponível no host; (2) proxy empacotado na stack; (3) terminação TLS externa.
- **Recomendação:** Escolher solução madura com renovação automática e health explícito, registrando a escolha em ADR após spike curto.
- **Bloqueia:** nenhuma para GDD; SDD precisa registrar a decisão antes de deploy.

### Registry e promoção de imagens

- **Pergunta:** Qual registry será usado, quem promove tags e como o host obtém as imagens?
- **Origens:** SDD 13 — Infra e deploy.
- **Por que importa:** Define cadeia de publicação, integridade de artefatos e privilégio das credenciais no host.
- **Opções:** (1) Registry associado ao CI; (2) registry privado independente; (3) build local no host.
- **Recomendação:** Registry com digest imutável e credencial de leitura mínima no host; promoção de tag controlada pelo processo de release.
- **Bloqueia:** nenhuma para GDD; SDD precisa definir o fluxo antes de deploy.

### Backups

- **Pergunta:** Qual frequência, destino, retenção e criptografia serão adotados para backups?
- **Origens:** SDD 13 — Infra e deploy.
- **Por que importa:** Determina a capacidade de recuperação de estado, log e snapshots após falha ou perda do host.
- **Opções:** (1) Backup diário externo; (2) diário mais snapshots frequentes; (3) retenção curta no host sem cópia externa.
- **Recomendação:** Backup diário consistente, cópia externa criptografada e teste mensal de restauração isolada.
- **Bloqueia:** aprovação do SDD.

### Swap de 2 GB

- **Pergunta:** A VPS receberá 2 GB de swap para diagnóstico e builds em contêiner?
- **Origens:** SDD 13 — Infra e deploy; ADR-0007 — consequências dos spikes.
- **Por que importa:** Swap pode reduzir falhas por pressão de memória, mas envolve mudança operacional no host.
- **Opções:** (1) Configurar swap; (2) evitar builds no host e não adicionar swap; (3) decidir após medição operacional.
- **Recomendação:** Não depender de swap em produção; decidir a configuração após medição e autorização operacional específica.
- **Bloqueia:** nenhuma para GDD/SDD; qualquer alteração no host exige autorização explícita.

### Gates operacionais e promoção

- **Pergunta:** Quais limites de p95, RAM, disco e retenção bloqueiam promoção, e qual tag/homologação precede produção?
- **Origens:** SDD 13 — Infra e deploy.
- **Por que importa:** Sem critérios mensuráveis, promoção e rollback ficam subjetivos.
- **Opções:** (1) Fixar limites agora; (2) medir baseline no harness e então aprovar limites; (3) promover sem gates quantitativos.
- **Recomendação:** Medir baseline antes de tornar números gates; definir SemVer com sufixo de fase e homologação com health, restore e replay.
- **Bloqueia:** aprovação do SDD.

## Produto

Nenhuma pergunta aberta de produto aparece diretamente nas seções consultadas. A janela de custo diário, que afeta a experiência do jogador, está agrupada em **Infra e segurança** por depender de regra de cobrança/medição a especificar no SDD.

## Decididas em 2026-10-01

- ADR-0007 aceito: Rust, Godot 4 e PostgreSQL; GDD aprovado e projeto na Fase 1.
- Chave do modelo de decisão Jev pertence ao operador; Laya será considerado somente ao final (ADR-0002 e ADR-0005).
- Magia: 3–5 fenômenos raros e sistêmicos; interferir na Entropia custa rituais, sacrifícios ou pesquisas proibidas.
- Snapshots versionados no início do mundo, no fim de cada era e a cada 50 turnos; turno começa em 0.
- Presença usa janela técnica de reconexão de 60 s e registra presentes na abertura do turno.
- Chunk de 48 KiB até benchmark no Android; leis podem restringir migração automática.
