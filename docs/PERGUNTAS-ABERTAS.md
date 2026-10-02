# Perguntas abertas consolidadas

Consolidação das perguntas abertas de GDD, SDD, `docs/STATUS.md` e ADRs ainda propostos ou parcialmente abertos. Recomendações são propostas para orientar a decisão do dono do projeto; não representam decisões aprovadas.

## Bloqueiam a próxima fase

A próxima fase depende da aprovação do GDD e depois da especificação do SDD. Prioridade considera dependências, impacto sistêmico e gates explícitos.

1. **Qual é o escopo inicial dos fenômenos mágicos e como eles interagem com tecnologia, sociedade e Entropia?** Bloqueia GDD.
2. **Quais regras de interferência na Entropia limitam ações, custo catastrófico e recompensas?** Bloqueia GDD.
3. **Como impedir inflação e colapso econômico ao longo de uma partida infinita?** Bloqueia GDD.
4. **Como leis afetam migração automática e quais limiares definem revolução moderada ou severa?** Bloqueia GDD.
5. **Como ordenar ações automatizadas em relação às ações humanas sem vantagem sistemática?** Bloqueia GDD.
6. **Qual política de propriedade e credenciais será usada para a chave Jev/Runware: operador ou jogador?** Bloqueia SDD.
7. **A stack proposta no ADR-0007 será ratificada?** Bloqueia SDD.
8. **Qual será a hospedagem do Laya quando ocorrer a migração de Jev?** Bloqueia SDD.
9. **Como o sistema define presença e trata queda de conexão durante um turno?** Bloqueia SDD.
10. **Quais política de backup, limites operacionais e gates de promoção serão adotados?** Bloqueia SDD.

## Design de jogo

### Fenômenos mágicos: escopo e expressão

- **Pergunta:** Qual catálogo inicial de fenômenos mágicos existe, quais seus gatilhos e como uma sociedade adere, proíbe ou regulamenta cada prática? A magia tem expressão no mapa, como locais, recursos ou anomalias? Como difere da tecnologia em requisitos de adesão, risco e reação dos vizinhos?
- **Origens:** GDD 00 — Visão; GDD 02 — Mapa e tiles; GDD 05 — Tecnologia; GDD 06 — Sociedade e governo; GDD 08 — Entropia e eventos.
- **Por que importa:** Define mecânicas transversais e conteúdo inicial que afetam exploração, progresso, leis e eventos.
- **Opções:** (1) Catálogo pequeno com fenômenos raros e efeitos sistêmicos; (2) fenômenos principalmente narrativos, com poucos efeitos mecânicos; (3) adiar magia para depois do MVP.
- **Recomendação:** Começar com poucos fenômenos mecânicos, com requisitos e riscos explícitos e integração consistente entre mapa, tecnologia e leis.
- **Bloqueia:** aprovação do GDD.

### Interferência na Entropia

- **Pergunta:** Quais ações permitem interferir na Entropia, como se calcula o custo catastrófico e o que concede um sucesso raro sem violar a justiça?
- **Origens:** GDD 08 — Entropia e eventos.
- **Por que importa:** Define risco e agência do jogador sobre o sistema diretor de eventos.
- **Opções:** (1) Sem interferência direta; (2) ações limitadas com custo e risco escalados pela ambição; (3) ações por recurso raro e limites por era.
- **Recomendação:** Limitar as ações a templates validados, com risco determinístico baseado na seed e custo crescente conforme a ambição.
- **Bloqueia:** aprovação do GDD.

### Economia em ciclo infinito

- **Pergunta:** Como evitar inflação de rendimentos e estoques ao longo de uma partida sem fim?
- **Origens:** GDD 03 — Economia.
- **Por que importa:** Sem limites sustentáveis, a economia pode perder escolhas relevantes ou tornar custos irrelevantes.
- **Opções:** (1) Limites por tile e estoque; (2) manutenção crescente por cidade, melhoria e rota; (3) combinação de limites e manutenção, sem multiplicadores automáticos por era.
- **Recomendação:** Simular limites de produção e estoque com manutenção recorrente; calibrar por saldo/população e tempo de recuperação, sem escalar apenas pelo avanço do turno.
- **Bloqueia:** aprovação do GDD.

### Migração e revoltas

- **Pergunta:** Leis podem restringir a migração automática? Quais limiares separam revolução moderada de severa?
- **Origens:** GDD 04 — Cidades e população; GDD 06 — Sociedade e governo.
- **Por que importa:** Conecta políticas de governo à estabilidade, população e distribuição territorial.
- **Opções:** (1) Leis apenas influenciam migração; (2) leis podem restringi-la com custos sociais; (3) sem proibição, apenas incentivos. Para revoltas: limiares fixos, faixas graduais ou avaliação por múltiplos fatores.
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

### Clave do modelo de decisão

- **Pergunta:** A chave Jev/Runware/OpenRouter usada pelo jogo pertence ao operador do servidor ou ao jogador?
- **Origens:** STATUS — Perguntas abertas; ADR-0002 — subdecisão sobre chaves; ADR README — ADR-0002, chaves Proposto.
- **Por que importa:** Define quem paga, controla cotas e assume a gestão do segredo para chamadas T1.
- **Opções:** (1) Chave do operador, com custo repassado/limitado; (2) chave do jogador; (3) aceitar ambas com precedência configurável.
- **Recomendação:** Preferir chave do operador com orçamento global e por jogador, mantendo fallback T0; avaliar chave do jogador como alternativa para servidores auto-hospedados.
- **Bloqueia:** aprovação do SDD.

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

### Ratificação da stack

- **Pergunta:** O usuário ratifica a stack proposta no ADR-0007: Rust, Godot 4, PostgreSQL, Markdown + SQLite FTS5, Docker Compose e GitHub Actions?
- **Origens:** STATUS — Perguntas abertas; ADR README — ADR-0007 Proposto; ADR-0007 — Stack tecnológica.
- **Por que importa:** A stack condiciona contratos, ferramentas, deployment e início de desenvolvimento de produção.
- **Opções:** (1) Ratificar integralmente; (2) ratificar com alterações pontuais; (3) reabrir alternativas de núcleo/cliente/banco.
- **Recomendação:** Ratificar Rust após spikes positivos e decidir explicitamente eventuais exceções antes do SDD final.
- **Bloqueia:** aprovação do SDD.

### Presença e queda de conexão

- **Pergunta:** Como definir precisamente “presente” e o que ocorre se a conexão cair no meio do turno?
- **Origens:** GDD 01 — Loop e turnos; ADR-0008 — Turno sem relógio; ADR README — ADR-0008 aceito, consequência em aberto.
- **Por que importa:** Define quando o Governador assume e quando o mundo pode avançar sem prender os demais jogadores.
- **Opções:** (1) Presença enquanto sessão ativa no mundo, com desconexão removendo o jogador da espera; (2) manter presença até confirmação ou reconexão; (3) janela curta de reconexão antes da substituição pelo Governador.
- **Recomendação:** Definir presença como sessão ativa no mundo; após desconexão, permitir reconexão ao mesmo turno e então transferir o restante da jogada ao Governador por regra explícita.
- **Bloqueia:** aprovação do SDD.

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

## Perguntas não listadas como abertas

O ADR README marca como aceitos ADR-0001, ADR-0003 (com fechamento pelo ADR-0008), ADR-0004, ADR-0006 e ADR-0008. Suas decisões aceitas foram excluídas. ADR-0002 e ADR-0005 aparecem apenas nas subdecisões em aberto; ADR-0007 permanece proposto. O ADR README não registra outros ADRs propostos.