# Guia de revisão dos SDDs

> Roteiro para revisar os SDDs em blocos dependentes. Cada alternativa marcada como **(Recomendado)** é uma proposta, nunca uma decisão já tomada. Itens em **aceitar em bloco** são técnicos: preservam decisões já aceitas, invariantes e nomenclatura canônica de `REVISAO-CRUZADA.md`.

## Ordem de revisão

| Ordem | Par | Por que revisar junto |
|---|---|---|
| 1 | 00 + 01 | A visão fixa fronteiras e contratos; o núcleo prova que eles cabem em um `step` puro. |
| 2 | 02 + 16 | O mapa canônico determina tanto o que existe quanto o que cada civilização pode conhecer. |
| 3 | 03 + 17 | Fechamento de turno, presença, sessão, identidade e sucessão formam um único ciclo de vida. |
| 4 | 04 + 08 | Portas de IA só são seguras e econômicas com contexto, retenção e compactação definidos. |
| 5 | 05 + 06 | Governador e Entropia são as duas automações que propõem ações antes do fechamento. |
| 6 | 07 + 15 | Diplomacia depende das regras de economia, sociedade, guerra e contratos que ela aciona. |
| 7 | 09 + 14 | Persistência só é confiável se replays, backups, privacidade e testes puderem comprová-la. |
| 8 | 10 + 12 | O protocolo é o contrato que o cliente mobile implementa e torna compreensível. |
| 9 | 11 + 13 | Chaves, custos, operação, publicação e recuperação definem a fronteira de segurança real. |
| 10 | 18 | A DSL fecha o formato seguro dos catálogos consumidos pelos demais SDDs. |

## 00 + 01 — Visão geral e núcleo de simulação

**O par define:**

- a topologia autoritativa e as fronteiras entre orquestração, portas e motor;
- o estado canônico, `AcceptedCommand` e a sequência global `accepted_sequence`;
- `step(state, accepted_commands, seed, versions)` como transição pura e reproduzível;
- PRNG, hash, versões de regras e recuperação por log e `WorldSnapshot`;
- orçamentos e invariantes que todos os demais subsistemas devem respeitar.

**Perguntas para o dono**

1. Qual orçamento passa a ser compromisso mensurável de CI quando houver baseline?
   - **Alertas até o harness medir (Recomendado):** evita números arbitrários; adia gates quantitativos.
   - Fixar p95, memória e custo agora: dá meta imediata; pode induzir limites irreais.
   - Sem orçamento formal: reduz trabalho inicial; deixa regressões crescerem sem sinal.

2. Qual política única retém conteúdo não mecânico (texto do jogador, narrativa, `IntentEvidence` e itens supersedidos)?
   - **Log mecânico e hashes pelo save; bruto sanitizado com prazo configurável e expurgo verificável (Recomendado):** preserva replay/auditoria; exige política e operação de expurgo.
   - Reter tudo indefinidamente: maximiza suporte; aumenta risco e custo de privacidade.
   - Reter apenas hashes/intenção validada: minimiza dados; limita investigação de incidentes.

**Aceitar em bloco (técnico)**

- Servidor autoritativo, event sourcing e motor determinístico já são decisões aceitas.
- `AcceptedCommand`, `ActionIntent`, `IntentEvidence`, `GroundingRef`, `RulesetRef`, `StateHash` e `WorldSnapshot` são os nomes canônicos.
- O replay aplica somente `AcceptedCommand`; evidência de IA não é revalidada para mudar história.
- Turno começa em `0`; snapshots versionados ocorrem no início, fim de era e a cada 50 turnos.
- `C`, `L`, `S`, `D`, `G`, `W`, `E`, `P`, `Cf`, `R`, `Dv`, `A` e `U` usam exatamente os sentidos de `gdd/12-variaveis-e-formulas.md`.

## 02 + 16 — Hexágonos, mapa e visibilidade

**O par define:**

- coordenadas hexagonais, identificadores e propriedades canônicas de tiles;
- geração procedural, validade/fairness de mundo e versão do gerador;
- chunks de mapa, revisões e custo de transferência ao cliente;
- conhecimento observado, lembrado e oculto por civilização;
- autorização de `GroundingRef` e projeções sem vazamento de fog of war.

**Perguntas para o dono**

1. Que escala de mundo deve orientar o primeiro benchmark?
   - **Definir mínimo, máximo e teto de tiles antes do harness (Recomendado):** permite provar `TileId`/overflow; requer escolher um alvo inicial.
   - Só tamanho padrão: simplifica o MVP; pode esconder falha em mundos tardios.
   - Escala ilimitada configurável: dá liberdade; inviabiliza orçamento e testes agora.

2. O que é estado canônico do tile?
   - **Guardar dados-base e derivar bioma por catálogo versionado (Recomendado):** reduz duplicação; exige versões preservadas no save.
   - Persistir bioma, fertilidade e clima finais: facilita leitura; aumenta migração e risco de divergência.
   - Derivar tudo a cada leitura: simplifica save; pode perder efeitos históricos necessários.

3. Como começar chunks e atualizações de mapa?
   - **Chunks completos versionados, com delta só após medição (Recomendado):** contrato simples; consome mais rede no começo.
   - Delta desde o início: economiza rede; aumenta complexidade de reconciliação.
   - Snapshot inteiro do mundo: muito simples; não atende escala mobile.

4. Qual informação lembrada deve permanecer visível fora da visão atual?
   - **Última observação com turno e incerteza explícitos (Recomendado):** é legível e não revela presente oculto; requer projeção adicional.
   - Revelar somente visão atual: é rigoroso; torna planejamento menos confortável.
   - Manter tudo observado como atual: é conveniente; viola fog of war.

**Aceitar em bloco (técnico)**

- Grid hexagonal e coordenadas axiais/cúbicas são decisões aceitas.
- Cada partida persiste versões de schema, gerador e catálogo; mudança sem migração explícita não altera save.
- O limite provisório de chunk é **48 KiB** até benchmark Android.
- Visibilidade controla projeções e grounding; o motor continua vendo o estado completo.
- Fixtures geométricas cobrirão linhas polares, rios, lagos, wrap e conectividade.

## 03 + 17 — Turnos, sessões, identidade e contas

**O par define:**

- presença técnica sem relógio de jogo, `Pronto` e fechamento de turno;
- sessões, dispositivos, credenciais e autorização por civilização;
- ordem determinística das ações automatizadas e dos comandos aceitos;
- entrada tardia, assunção de bot, sucessão e desativação de conta;
- auditoria de presença e preservação de log, Ledger e Crônica em transições.

**Perguntas para o dono**

1. Qual política completa de presença e reconexão vale no fechamento?
   - **Heartbeat; ausente após queda observada; retorno/desfazer `Pronto` só enquanto turno aberto (Recomendado):** preserva turno sem relógio; exige implementar auditoria de transição.
   - Sessão aberta vale indefinidamente: simples; uma aba abandonada pode congelar o mundo.
   - Fechar por prazo de jogo: resolve espera; contradiz ADR-0008.

2. Como ordenar automações em relação a humanos?
   - **Ordem rotativa por civilização derivada da seed, com abertura/fechamento registrados (Recomendado):** distribui vantagem; precisa de harness de viés.
   - Bots sempre antes: previsível; favorece automação.
   - Bots sempre depois: simples; favorece humanos recorrentes.

3. Qual regra de tomada, entrada tardia e conta apagada?
   - **Vaga aberta permite assunção imediata; conta desativada vira bot; log, Ledger e Crônica permanecem (Recomendado):** mantém continuidade; requer pré-condições e projeções de ownership explícitas.
   - Carência de N eras: reduz troca oportunista; adiciona espera e parâmetro arbitrário.
   - Só convite do dono: dá controle; pode deixar mundos sem sucessor.

4. Qual credencial entra no MVP e quantos dispositivos controlam?
   - **Passkey com recuperação controlada; um controlador por civilização e até cinco sessões por conta (Recomendado):** bom para mobile e evita voto duplo; recuperação exige fluxo seguro.
   - Senha tradicional e vários controladores: familiar; amplia ataque e concorrência.
   - Provedor externo obrigatório: reduz implementação local; cria dependência de fornecedor.

**Aceitar em bloco (técnico)**

- Janela técnica de reconexão é 60 s; presentes são registrados na abertura do turno.
- A janela não é regra de jogo nem altera o `step`.
- Comandos já aceitos nunca são removidos por reconexão; novos entram no fim da sequência.
- Governador só pratica diplomacia reversível explicitamente autorizada pelo Mandato.
- Observação pública fica fora do MVP até as regras de visibilidade permitirem.

## 04 + 08 — Camadas de IA, memória e contexto

**O par define:**

- portas T0/T1/T2, intenções tipadas, grounding, validação e fallback determinístico;
- isolamento de `UntrustedPlayerText` contra prompt injection;
- documentos canônicos e memória working, episodic, semantic e procedural;
- compactação, previsão de saturação, reset e Crônica regenerável;
- uso de tokens, prazo, custo e evidência sem transformar IA em fonte da verdade.

**Perguntas para o dono**

1. Como validar a integração T1 Jev/Laya antes de fixar o adaptador?
   - **Manter Jev inicial e autorizar spike de schema, timeout, idempotência e custo (Recomendado):** preserva ADR-0005 e reduz incerteza; não decide a migração Laya.
   - Trocar agora por outro contrato: pode simplificar fornecedor; reabre direção já aceita sem evidência suficiente.
   - Não testar até a implementação: acelera papel; concentra risco na Fase 3.

2. Como testar prompt injection e texto livre?
   - **Fixtures adversariais automatizadas mais revisão periódica (Recomendado):** cria regressão verificável; exige manter corpus saneado.
   - Só avaliação manual: é flexível; não protege contra regressão.
   - Só filtros automáticos: é barato; pode não cobrir ataques novos.

3. Como reter evidência e texto bruto de IA/jogador?
   - **Sempre intenção validada e hashes; bruto sanitizado, protegido e expirável (Recomendado):** replay permanece suficiente e suporte é possível; requer expurgo auditável.
   - Reter bruto por toda a partida: melhora diagnóstico; aumenta superfície de privacidade.
   - Reter somente hashes: minimiza dados; dificulta contestação e suporte.

4. Qual autonomia tem a Doutrina e qual visibilidade tem a Crônica?
   - **Doutrina só como sugestão citada; Crônica interna mundial com visão filtrada por civilização (Recomendado):** preserva Mandato e fog; exige projeções por leitor.
   - Doutrina altera Mandato automaticamente: automatiza mais; viola autoridade configurável do jogador.
   - Crônica mundial pública integral: é simples; pode vazar informação oculta.

**Aceitar em bloco (técnico)**

- T0 é fallback universal; IA nunca escreve estado diretamente.
- Toda chamada externa resulta em comando validado antes do `step`.
- O contexto é derivado e reconstruível; perder memória degrada qualidade, não corrompe jogo.
- Limiares de compactação/reset e valores de token ficam versionados e calibráveis por harness.
- Laya só será considerado ao final; a chave T1 pertence ao operador.

## 05 + 06 — Governador e Entropia

**O par define:**

- Mandato, prioridades, reservas, proibições, avisos e fallback do Governador;
- seleção de ações por utilidade `U` dentro de limites aplicados pelo motor;
- Entropia como diretora de eventos elegíveis com orçamento de tensão e templates;
- interferência rara e sistêmica, registrada como comandos e nunca como efeito livre;
- relatórios de ausência, revoluções e explicação causal no retorno ao jogo.

**Perguntas para o dono**

1. Quais ações e intensidades entram na revolução?
   - **Três faixas em catálogo; moderada propõe Mandato reversível e severa impõe Mandato do novo regime (Recomendado):** respeita GDD decidido; parâmetros seguem calibráveis.
   - Limiar fixo único: é simples; não representa escalada social.
   - Avaliação livre por IA: parece rica; viola determinismo e catálogo fechado.

2. Quais verbos de interferência da Entropia entram no lançamento?
   - **`predict` e `appease` primeiro (Recomendado):** são auditáveis e preservam agência; reduz variedade inicial.
   - Os quatro verbos propostos: oferece mais possibilidades; aumenta conteúdo e balanceamento.
   - Nenhum no MVP: simplifica; remove uma escolha já desejada para magia/eventos.

3. Toda transição de Era exige foco, renúncia e legado limitado?
   - **Sim, uma escolha sistêmica com até dois legados ativos (Recomendado):** torna trade-offs claros; exige catálogo e UX de transição.
   - Legados automáticos: reduz fricção; reduz agência estratégica.
   - Sem legados: simplifica; enfraquece continuidade entre eras.

4. Como o retorno após longa ausência deve ser relatado?
   - **Agrupar por tema causal com links de turno (Recomendado):** é legível no celular; requer agrupador de apresentação.
   - Listar cada turno: é completo; sobrecarrega o jogador.
   - Mostrar apenas crises: é curto; esconde decisões úteis.

**Aceitar em bloco (técnico)**

- Mandato é autoridade do jogador; motor bloqueia qualquer excesso.
- Entropia só escolhe template elegível; `ActivateEntropyEvent` validado chega ao núcleo.
- Magia terá 3–5 fenômenos raros; interferir custa rituais, sacrifícios ou pesquisas proibidas.
- Curvas, proteções, orçamento e presets são dados calibráveis, não constantes de código.
- Sem confirmação, ação irreversível pendente do Governador resulta em `no_op`.

## 07 + 15 — Diplomacia e regras de domínio

**O par define:**

- máquina de estados diplomática, tratados, guerra, contraproposta e colapso;
- `LedgerEntry` causal com confiança `Cf`, ressentimento `R`, dívida `Dv` e aceitação `A`;
- economia, cidades, tecnologia, sociedade, combate e ordem de resolução;
- fórmulas compartilhadas de `D`, `G`, `W`, `E`, `P`, `C`, `L` e suas invariantes;
- contratos mistos, escassez, ambições e conflitos de comandos a testar no harness.

**Perguntas para o dono**

1. Quais freios são obrigatórios para economia de ciclo infinito?
   - **Capacidade, estoques limitados e manutenção recorrente; escassez gera custo e instabilidade (Recomendado):** conserva escolhas longas; exige calibração de fórmulas.
   - Só limites por tile/estoque: controla produção; pode deixar expansão barata.
   - Só manutenção crescente: cria custo; pode gerar crescimento abstrato demais.

2. Qual escopo inicial de diplomacia, facções e ambições sociais?
   - **Comércio, ajuda, passagem, pacto, trégua e quatro objetivos de guerra; ambições agregadas situacionais (Recomendado):** conecta eventos à sociedade sem simular indivíduos; adia aliança/vassalagem e personagens profundos.
   - Incluir aliança, vassalagem e personagens já no MVP: é mais rico; amplia muito regras e conteúdo.
   - Só paz/guerra: é enxuto; não realiza o Ledger diplomático proposto.

3. Como resolver conflitos simultâneos entre comandos?
   - **Matriz versionada por tipo/capacidade, com desempate verificável e motivo de rejeição/adiamento (Recomendado):** dá legibilidade sem mudar ADR-0003; exige casos de teste.
   - Ordem de chegada apenas: é simples; cria vantagem de velocidade.
   - Resolver livremente por IA: parece contextual; não é reproduzível.

4. Quais regras precisam ser fechadas antes do primeiro catálogo de combate/contrato?
   - **Posturas fechadas, custos/reservas declarados e `r = min(r_lados)` para pacote misto (Recomendado):** torna falha proporcional auditável; exige validar casos de ambos os lados.
   - Fórmulas livres por conteúdo narrativo: dá flexibilidade; impede invariantes confiáveis.
   - Adiar contratos mistos: reduz escopo; limita tratados econômicos iniciais.

**Aceitar em bloco (técnico)**

- Ledger preserva histórico depois do decaimento; obrigações ativas não somem.
- `P_c`, `P_civ` e `C'` obedecem literalmente a `gdd/12-variaveis-e-formulas.md`.
- Leis podem restringir migração automática.
- Pesos/limiares de `A`, crescimento e revolta são baseline de harness, não decisão congelada.
- Tratados de humano ausente só são ratificados pelo Governador se o Mandato permitir; caso contrário expiram/repudiam deterministicamente.

## 09 + 14 — Persistência e estratégia de testes

**O par define:**

- log relacional, snapshots, recuperação, catálogos e migrações de mundos;
- backups, RPO/RTO, cadeia histórica e separação entre dados mecânicos e evidência;
- golden replays, propriedades, determinismo entre arquiteturas e fixtures de IA;
- testes de histórias longas, diplomacia causal e eventos encadeados;
- métricas de desempenho e critérios para avisos, gates e regressões.

**Perguntas para o dono**

1. Qual backup e RPO/RTO o SDD deve prometer?
   - **RPO de até um turno selado, backup diário externo cifrado e restauração isolada mensal (Recomendado):** recupera mundo eterno com prova; requer destino e operação definidos.
   - Backup diário local: é barato; não cobre perda do host.
   - Snapshots muito frequentes sem cópia externa: reduz RPO; aumenta custo sem desastre coberto.

2. Eras e versões antigas ficam recuperáveis por quanto tempo?
   - **Cadeia completa restaurável no MVP; compactação sem perda e versões legadas até migração determinística (Recomendado):** protege replay; aumenta armazenamento.
   - Apagar após resumo: reduz custo; remove auditoria e reversibilidade.
   - Migrar saves ao atualizar: simplifica runtime; pode mudar a história de partida.

3. Qual promessa inicial de testes e determinismo?
   - **CI em x86_64 e ARM, serialização canônica versionada e fixtures sintéticas saneadas (Recomendado):** prova o invariante principal; exige manter matrizes de ambiente.
   - Só uma arquitetura: é mais rápido; não prova portabilidade.
   - Respostas reais não saneadas: aumenta realismo; cria risco de licença e privacidade.

4. Que harness de mundo tardio deve desbloquear gates?
   - **Cenários longos de economia, diplomacia e eventos; medir custo por turno na escala escolhida antes de gates (Recomendado):** transforma propostas em evidência; adia números até baseline.
   - Fixar SLA sem simular: dá prazo; é especulativo.
   - Sem harness longo: reduz trabalho; deixa regressões sistêmicas ocultas.

**Aceitar em bloco (técnico)**

- Snapshots não substituem o log; recuperação deve validar hashes e replay.
- Fixtures de IA nunca chamam provedor real em CI.
- Auditoria diplomática exige referências causais desde o início; decomposição numérica entra ao ratificar `A`.
- Métricas sem baseline começam como aviso, não teste vermelho.
- Evidência não mecânica segue a política única de retenção escolhida no par 00 + 01.

## 10 + 12 — Protocolo e cliente mobile

**O par define:**

- transporte, envelope versionado, idempotência, recibo e reconciliação por snapshot/delta;
- autenticação no protocolo, sessão, `Pronto`, notificações e erros;
- mapa, pauta, inspeção, cache descartável e reconexão no Godot;
- orçamento de payload, aparelho Android de referência e orientação de tela;
- acessibilidade e explicação curta de mudança, causa, prazo e opções ao jogador.

**Perguntas para o dono**

1. Qual contrato de transporte entra na v1?
   - **JSON tipado com schemas/envelope versionados; binário só após benchmark (Recomendado):** acelera integração Rust–Godot; payload pode ser maior.
   - Binário com geração de tipos já: economiza bytes; aumenta toolchain e depuração.
   - JSON sem schema: é rápido; fragiliza compatibilidade.

2. Push é necessário no primeiro MVP, e quais eventos despertam o jogador?
   - **Sim, via outbox transacional; somente decisão pendente, crise, proposta relevante e fim de ausência (Recomendado):** separa aviso de estado e evita duplicata; acrescenta operação de entrega.
   - Sem push: reduz integração; enfraquece sessões curtas/ausência.
   - Push para cada turno: é simples; causa ruído e gasto.

3. Qual alvo Android e política de cache valem no MVP?
   - **Aparelho intermediário mais perfil de memória restrita; cache apagável de projeção já vista (Recomendado):** mede experiência real sem modo offline; abre mais lento após limpeza.
   - Cache persistente amplo: abre rápido; aumenta risco em aparelho comprometido.
   - Sem cache: reduz dados locais; piora reconexão e uso móvel.

4. Qual escopo visual e orientação entram primeiro?
   - **Android priorizado; web/desktop só Pauta, Mapa, inspeção e depuração; sistema e botão para paisagem apenas no mapa (Recomendado):** concentra o MVP; limita paridade inicial.
   - Paridade total: facilita desktop; posterga alvo mobile.
   - Só retrato: simplifica; limita leitura espacial do mapa.

**Aceitar em bloco (técnico)**

- Cliente é projeção, nunca modo offline jogável ou fonte de verdade.
- `accepted_sequence`, recibos, cursor e versão de schema permitem reconciliação idempotente.
- Chunks ficam em 48 KiB até benchmark Android; demais limites de payload são experimentais.
- Token não entra em cache de projeções nem em logs.
- Notificação é enviada fora do `step`; ao abrir, o app busca estado autenticado.

## 11 + 13 — Segurança, chaves, infraestrutura e deploy

**O par define:**

- BYOK, envelopes criptográficos, rotação, exclusão e acesso mínimo a segredos;
- contabilidade separada de T1/T2, teto de custo e fallback ao esgotar orçamento;
- topologia operacional, CI, imagens, deploy, health, rollback e observabilidade;
- proxy/TLS, registry, promoção, capacidade e recuperação operacional;
- fronteiras que não podem expor chaves, endereços, hostnames ou segredos em logs/repositório.

**Perguntas para o dono**

1. Como o teto diário de T2 deve funcionar para o jogador?
   - **Janela móvel de 24 h; reserva antes da chamada; valor exibido com moeda/provedor explícitos; T0 ao esgotar (Recomendado):** é previsível no uso; exige contabilidade por chamada.
   - Dia UTC fixo: simples; pode surpreender no fuso local.
   - Dia civil do jogador: é familiar; complica enforcement global.

2. Qual cofre/gestão de chave-mestra e política BYOK entram antes de persistir chaves reais?
   - **Cofre versionado/auditável com acesso mínimo; uma chave por `(jogador, provedor)` e purga verificável em backups (Recomendado):** limita exposição; requer ADR e operação do cofre.
   - Chave-mestra no banco: simplifica; enfraquece segregação de segredo.
   - Uma chave para todos provedores: reduz UX; pode enviar segredo ao destino errado.

3. Como publicar e promover imagens?
   - **Registry ligado ao CI, digest imutável, leitura mínima no host e promoção controlada por release (Recomendado):** melhora integridade; exige credencial e processo de release.
   - Registry privado independente: mais controle; adiciona operação.
   - Build local no host: simples no começo; consome recursos e reduz rastreabilidade.

4. Qual caminho operacional de TLS e promoção deve ser aprovado?
   - **Proxy maduro com renovação automática; homologação SemVer com health, restore e replay; gates numéricos após baseline (Recomendado):** cria rollback verificável; deixa métricas finais pendentes.
   - Terminação TLS externa e promoção manual: menos stack local; aumenta dependência externa.
   - Promover sem gates: é rápido; torna incidentes e rollback subjetivos.

**Aceitar em bloco (técnico)**

- T1 é pago pelo operador; Laya não faz parte da implementação inicial.
- Sem chave, timeout ou teto, o jogo avança com T0/T1 conforme disponível.
- Swap não é dependência de produção; qualquer alteração operacional exige autorização específica posterior.
- Backups seguem a política decidida no par 09 + 14, com retenção de envelopes compatível com expurgo.
- Documentos usam placeholders operacionais; não registram endereços, hostnames, portas reais ou segredos.

## 18 — Linguagem de catálogos e templates

**O documento define:**

- formato declarativo de eventos, tecnologia, tratados, governo e conteúdo mecânico;
- AST/contratos fechados, efeitos permitidos e avaliação pura;
- ordenação, PRNG explícito, limites de profundidade, nós, alvos e operações;
- hash, versões, publicação, migração e catálogos legados por partida;
- rejeição de código arbitrário, I/O e conflitos mecânicos não especificados.

**Perguntas para o dono**

1. Qual formato-fonte canônico deve servir à autoria humana?
   - **YAML restrito normalizado para AST/JSON canônico antes do hash (Recomendado):** é amigável e reproduzível; requer validador/normalizador rigoroso.
   - JSON puro: é menos ambíguo; é menos confortável para catálogos extensos.
   - Múltiplos formatos: dá flexibilidade; multiplica superfície de compatibilidade.

2. Como garantir compatibilidade de saves após atualização?
   - **Fixar hash/versão por partida e manter catálogos legados legíveis; migrar só de modo explícito (Recomendado):** preserva replay; aumenta retenção de conteúdo.
   - Atualizar todos os mundos: simplifica runtime; muda regras históricas.
   - Reescrever logs antigos: reduz versões; viola event sourcing.

3. Qual escopo da primeira DSL?
   - **Enum mínimo de operações para templates de evento, tecnologia, tratado e governo (Recomendado):** é testável; limita conteúdo inicial.
   - DSL geral: maximiza autoria; aumenta risco e prazo.
   - Código em plugins: é flexível; viola ausência de execução arbitrária.

4. Como resolver operações concorrentes no mesmo alvo/campo?
   - **Rejeitar no validador, salvo composição declarada (Recomendado):** evita efeito implícito; requer escrever composição quando necessária.
   - Última operação vence: é simples; depende de ordem e surpreende autores.
   - Somar sempre: é conveniente; falha para tipos não aditivos.

**Aceitar em bloco (técnico)**

- O intérprete é puro, limitado e não executa código nem I/O.
- Todo template declara pré-condições, opções, efeitos e referências causais.
- Cada consumidor declara fallback T0 tipado; fallback não inventa efeito nem bloqueia turno.
- Limites iniciais são tetos provisórios calibrados pelo harness.
- Catálogos referenciados acompanham manifesto e backup da partida enquanto a retenção formal exigir.

## Fechamento da revisão

Ao concluir os blocos, registrar cada resposta como decisão, ADR quando alterar arquitetura/oper operação, ou parâmetro de balanceamento quando depender do harness. Nada marcado como recomendação neste guia se torna decidido sem resposta explícita do dono.
