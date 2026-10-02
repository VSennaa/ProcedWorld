# SDD 18 — Linguagem de catálogos e templates

> **Status: proposta para revisão do SDD.** GDD aprovado em 2026-10-01 e ADR-0007 aceito.
> Este documento descreve contratos e invariantes;
> detalhes de implementação seguem Rust, Godot e PostgreSQL conforme ADR-0007 aceito.

## 1. Objetivo e escopo

Este subsistema define como catálogos descrevem condições, seletores de alvo e efeitos
mecânicos de templates sem incluir código executável. O consumidor valida os dados offline,
carrega uma versão imutável do catálogo e avalia suas regras de forma pura e limitada.

Os nomes canônicos deste contrato são `PredicateSpec`, `TargetSelectorSpec` e
`ValidatedEffectOp` (docs/sdd/REVISAO-CRUZADA.md). Os templates cobrem eventos da Entropia,
tecnologias, tratados e governos. O vocabulário de operações permitido é explícito e próprio
do motor; cada nova operação exige mudança revisada do contrato e da implementação.

O desenho atende à maleabilidade por dados: a Entropia escolhe e parametriza templates
validados, mas efeitos mecânicos continuam definidos por regras do motor. Tecnologias podem
ter árvore base finita e descobertas emergentes validadas. Diplomacia mantém transições e
aceitação sob controle do motor.

## 2. Responsabilidades e fronteiras

Responsabilidades:

- Definir um formato serializável e fechado para condições, seleção de alvos e operações.
- Validar sintaxe, tipos, referências, limites, compatibilidade e migrações antes de publicar
  um catálogo para partidas.
- Avaliar predicados e seletores contra uma visão de estado imutável e produzir operações
  tipadas, ordenadas e sujeitas a validação do motor.
- Fornecer rastreabilidade entre conteúdo, versão do catálogo, operação aplicada e log de
  comandos, permitindo replay sem consultar IA ou reler conteúdo mutável.
- Rejeitar conteúdo inválido em modo fail-closed; conteúdo não validado jamais é carregado
  como mecânica ativa.

Fora da fronteira:

- Não escolhe a política narrativa, dificuldade ou orçamento de tensão da Entropia.
- Não decide regras de equilíbrio, custos de tecnologias ou semântica de tratados; essas regras
  pertencem aos subsistemas e ao GDD, e aqui são representadas por dados validados.
- Não interpreta texto livre como condição ou efeito e não compila scripts, expressões de usuário,
  SQL, bytecode ou módulos nativos.
- Não chama LLM, rede, relógio, sistema de arquivos ou banco de dados durante avaliação.
- Não grava estado diretamente. O motor valida operações e as converte em comandos gravados.
- Não renderiza apresentação, localização, animação ou UI no cliente.
- Não autoriza a IA a adicionar operações, IDs de alvo ou parâmetros fora do template.

## 3. Contrato de dados

### 3.1 Envelope do catálogo

Todo pacote publicado possui identidade e versão explícitas. Exemplo esquemático:

```yaml
catalog_id: core
catalog_version: 3
schema_version: 1
content_hash: "sha256:..."
minimum_engine_version: 1
templates:
  - id: event.drought.pressure
    kind: event
    revision: 2
    enabled: true
    parameters: { severity: { type: int, min: 1, max: 3 } }
    when: { op: all, args: [] }
    targets: { kind: civilization, scope: eligible }
    effects: []
```

O hash cobre forma canônica e conteúdo mecânico, excluindo campos de apresentação explicitamente
marcados. A implementação deve rejeitar IDs duplicados, versões incompatíveis, hash divergente,
chaves desconhecidas em campos mecânicos e referências ausentes. Campos de apresentação não
podem alterar elegibilidade, alvos, ordenação, parâmetros mecânicos ou resultado.

### 3.2 Tipos fechados

```text
PredicateSpec =
  All(List<PredicateSpec>) | Any(List<PredicateSpec>) | Not(PredicateSpec)
  | Compare(StateFactRef, Comparator, ScalarLiteral)
  | HasTag(EntityRef, TagId) | Exists(TargetSelectorSpec)

TargetSelectorSpec =
  Self | EntityKind(EntityKind, Filter?) | Related(RelationKind, Filter?)
  | ExplicitIds(List<EntityId>)

ValidatedEffectOp =
  AdjustResource(TargetRef, ResourceId, BoundedInt)
  | AddTag(TargetRef, TagId) | RemoveTag(TargetRef, TagId)
  | CreateLedgerEntry(PartyRef, PartyRef, TreatyTermId, Duration)
  | SetTechnologyState(TargetRef, TechnologyId, TechnologyTransition)
  | EmitChronicle(ChronicleTemplateId, BoundedParameters)
```

Esta lista é proposta e não autoriza por si só implementar todas as operações. Cada operação
tem schema versionado, pré-condições, limites e regra de aplicação documentados. `TargetRef`
deve resolver para IDs pertencentes ao conjunto retornado pelo seletor associado. Referências
de texto são IDs de catálogo, nunca nomes convertidos implicitamente.

Os literais permitidos são inteiros com largura e faixa definidas, booleanos, enums e IDs
tipados. Não há float no núcleo de avaliação. Comparadores e campos consultáveis são enums
fechados documentados pelo motor; não existe acesso reflexivo a propriedades arbitrárias.

### 3.3 Template e fluxo

```text
Template<TKind> = {
  id: TemplateId, revision: UInt,
  parameters: ParameterSchema,
  when: PredicateSpec,
  targets: TargetSelectorSpec,
  effects: List<ValidatedEffectOp>,
  limits: ExecutionLimits
}

validateCatalog(raw) -> Result<ValidatedCatalog, List<Diagnostic>>
evaluate(template, readOnlyState, boundParameters, seedContext)
  -> Result<OrderedOperations, EvaluationDiagnostic>
applyOperations(state, OrderedOperations) -> Result<CommandBatch, ApplyError>
```

`ValidatedCatalog` é uma representação imutável construída apenas por `validateCatalog`.
Parâmetros recebidos durante uma partida são novamente validados por schema e limites. A
avaliação retorna propostas tipadas; a camada de regras confere pré-condições finais e emite
comandos. Um erro não pode aplicar parcialmente uma lista: o lote é aceito por inteiro ou
rejeitado por inteiro, salvo se uma operação definir explicitamente semântica atômica própria.

## 4. Gramática, avaliação e limites

A serialização textual é YAML ou JSON restrito a mapas, listas, escalares e enums conhecidos;
aliases, tags customizadas, âncoras recursivas e chaves duplicadas são rejeitados. A forma
normalizada é uma árvore com profundidade limitada, sem macros, interpolação, lambdas, loops,
recursão, chamadas de função por nome ou fragmentos de expressão.

Proposta de limites iniciais por template: profundidade de predicado 16; até 256 nós de
predicado; até 128 IDs explícitos; até 64 operações; até 256 alvos selecionados; até 32 KiB
serializados. Limites são validados no carregamento e novamente na fronteira de entradas
externas. Os valores são propostas a calibrar no harness, não decisões do GDD.

Predicados usam curto-circuito definido: `All` e `Any` percorrem argumentos na ordem canônica
normalizada; `Not` avalia um único argumento. Comparações têm regras de tipo estritas. Campo
ausente não vira zero ou falso: resulta em `UnknownRef` na validação ou `EvaluationError` se
ausência dinâmica for permitida pelo contrato daquele fato.

Seletores calculam o conjunto candidato a partir de uma visão de estado consistente. Resultado
é deduplicado por ID e ordenado lexicograficamente pela representação canônica do ID. Filtros
são avaliados nessa ordem, com limite de visitas. Seletores que excedam limite falham como um
todo; não truncam silenciosamente.

Operações são ordenadas por (prioridade declarada limitada, posição no template, ID estável do
alvo, índice do alvo). Prioridade não permite ultrapassar dependências do motor. Operações que
conflitam no mesmo campo seguem ordem explícita ou são rejeitadas na validação; jamais dependem
da ordem de iteração de mapas ou conjuntos.

### 4.1 Uso de PRNG

A linguagem não contém `random()`. Se uma seleção aleatória for necessária, o chamador do motor
consome o PRNG versionado e grava no comando a escolha resultante e seus parâmetros. Como
alternativa proposta para variações mecânicas deterministas, uma primitiva limitada poderá
receber um `RandomDraw` já materializado pelo chamador; a avaliação não avança nem possui o PRNG.
`RandomDraw` inclui algoritmo/versão e valor dentro de faixa validada. Nenhum template pode
escolher seed, consultar estado futuro ou consumir draws variáveis por curto-circuito. O número
de draws e sua associação ao template são definidos no plano determinístico do motor.

## 5. Modelo de dados e invariantes

Entidades conceituais:

- `CatalogManifest`: ID, `catalog_version`, `schema_version`, hash canônico, versão mínima do
  motor, origem e estado de validação.
- `TemplateDefinition`: ID global, tipo, revisão, schema de parâmetros, AST de condição,
  seletor, operações e limites declarativos.
- `ValidatedCatalog`: snapshot imutável indexado por IDs e hash; sem referências mutáveis ao
  documento de origem.
- `CatalogMigration`: transformação versionada `from_version -> to_version` com relatório de
  alterações e regra de compatibilidade.
- `CommandRecord`: referência ao ID/revisão do template, hash do catálogo, parâmetros aceitos,
  alvos resolvidos, draws materializados e operações/comando final gravado.

Invariantes verificáveis:

1. Só um catálogo validado, cujo hash confere, pode atender uma partida.
2. ID de template é estável; revisão aumenta quando sua semântica mecânica muda.
3. Referências a campos, tags, recursos, tratados e tecnologias devem resolver no mesmo conjunto
   de catálogo compatível.
4. Todo parâmetro está no tipo/faixa declarada; ausência só é possível se o schema disser isso.
5. Cada alvo das operações pertence ao resultado do seletor e à classe de entidade exigida.
6. Nenhuma operação excede os limites do tipo, capacidade ou pré-condição do motor.
7. A avaliação é função dos inputs explícitos: catálogo, estado de leitura, parâmetros e draws.
8. Para inputs idênticos, AST e resultados têm igualdade estrutural independente de máquina.
9. Um erro em validação não produz `ValidatedCatalog`; um erro de avaliação não produz lote parcial.
10. Entradas de catálogo e estado são dados, nunca instruções executáveis.

## 6. Segurança: ausência de código e I/O

A garantia é estrutural: parser aceita apenas uma gramática enumerada; validador converte nós
em enums internos; o interpretador contém despacho exaustivo sobre esses enums. Não há eval,
reflection, FFI, carregador de módulos ou execução de script. IDs resolvem em tabelas imutáveis
pré-carregadas, não em nomes de arquivo, query ou endpoint.

As funções `validateCatalog` e `evaluate` recebem bytes/árvores e dados explícitos. Suas
interfaces não recebem filesystem, socket, relógio, logger com efeitos laterais ou handle de
banco. Efeitos são valores de retorno, nunca chamadas a serviços. Uma auditoria de dependências
deve provar que o módulo de avaliação não tem dependência transitiva de I/O ou execução dinâmica.

O núcleo será implementado em Rust com tipos enum fechados e
`Result`; serialização/parse ocorre fora de `step`. O motor não serializa texto de catálogo
dentro de `step`, e o cliente Godot recebe apenas DTOs aprovados para apresentação. PostgreSQL,
se usado conforme ADR-0007, persiste manifestos, conteúdo versionado e hashes fora da avaliação.

## 7. Versões, publicação e migração

`schema_version` identifica a gramática; `catalog_version` identifica uma publicação do conteúdo;
`revision` identifica a semântica de cada template. Alterar apenas texto de apresentação pode
preservar revisão mecânica, mas muda hash total do pacote; um hash mecânico separado é opcional
e deve ser explicitado no manifesto.

Publicação segue: parse estrito → validação de schema → resolução de referências → checagem de
tipos/pré-condições → orçamento estático → normalização → hashes → relatório → aprovação e
disponibilização imutável. Partidas em curso fixam o manifesto/hash inicial; nunca recebem
atualização silenciosa de catálogo.

Migrações são transformações offline explícitas entre versões. Elas produzem pacote novo e
relatório comparando IDs removidos, revisões alteradas, mudanças de operação, parâmetros,
seletores e compatibilidade de replay. Migração não reescreve logs históricos. Uma partida pode
continuar com o catálogo antigo, desde que o runtime suporte sua versão, ou exigir uma migração
de save aprovada; regra final permanece aberta.

Mudanças incompatíveis requerem nova `schema_version` e migrador determinístico com fixtures.
Versões desconhecidas, migração sem caminho, downgrade sem prova de equivalência ou falha no hash
são rejeitados. Uma revisão antiga fica disponível enquanto houver replay/saves que a referenciem,
conforme política de retenção a decidir.

## 8. Falhas, fallbacks e determinismo

Erros offline incluem sintaxe, chave desconhecida, referência ausente, tipo incorreto, limite
excedido, operação proibida, ID duplicado, versão incompatível e hash divergente. Publicação
falha com diagnósticos contendo código, caminho no documento e IDs envolvidos. Nenhum fallback
transforma conteúdo mecânico inválido em conteúdo ativo.

Em execução, erro de condição, alvo ou operação rejeita a proposta do template. Para evento
externo/Entropia, o orquestrador grava a intenção recebida e usa fallback T0 determinístico ou
nenhum evento, conforme regra do subsistema; jamais tenta avaliar código alternativo vindo da
IA. A regra concreta de fallback por tipo de template deve ser definida pelo SDD consumidor.

Log de comando/evento contém: turno e sequência; ID/revisão do template; hash/schema/catalog
version; parâmetros normalizados; IDs de alvo resolvidos; draws usados; decisão de validação;
comandos resultantes ou código de rejeição; versão do motor e seed versionada já definida pelo
motor. Dados suficientes reproduzem o resultado sem IA. Texto narrativo de IA só é registrado
conforme contrato do subsistema, isolado dos campos mecânicos.

Nunca entra no `step`: chamada de parser, I/O, rede, relógio, chamada de IA, consulta ao banco,
texto livre como DSL, seed implícita, float não determinístico, ordem incidental de hash map,
exceção não normalizada ou interpretação de pacote mutável. O `step` recebe comandos já
validados e materializados, e permanece puro conforme ADR-0006.

Falhas internas de catálogo determinístico são registradas fora do núcleo com turno, código,
IDs e hash. Logs não incluem segredos nem prompts brutos por padrão. Diagnósticos devem ser
estáveis e não depender de locale; texto localizado é camada de apresentação.

## 9. Orçamento

Proposta para o validador offline: executar durante build/publicação, sem limite de latência de
turno; impor limite de arquivo e orçamento total para evitar consumo abusivo de memória/CPU.
Pico de memória deve ser proporcional ao tamanho máximo do pacote; estruturas indexadas não
duplicam AST e documento fonte em produção após validação.

Proposta de runtime: predicados e seletor limitados pelos máximos da seção 4; custo O(n) nos nós
de AST e entidades visitadas, mais ordenação O(k log k) dos alvos; operações O(m). Registrar
tempo por template, nós visitados, candidatos, alvos e operações. O orçamento por turno e os
limites serão calibrados em harness e falham CI quando regressões excederem tolerância definida.

Não há tokens nem custo de IA inerentes à DSL. Se T1/T2 escolherem ou parametrizarem templates,
suas chamadas seguem os orçamentos do Governador/Entropia; apenas IDs e parâmetros permitidos
podem voltar como intenção. Tokens nunca ampliam a gramática. CI usa fixtures gravadas e não
chama provedores reais.

## 10. Estratégia de testes

- Testes de gramática aceitam cada variante permitida e rejeitam código, I/O, chaves extras,
  recursão, referências externas, chaves duplicadas, números fora de faixa e payloads excessivos.
- Testes de propriedade geram ASTs e estados válidos/ inválidos: avaliação nunca altera o estado,
  nunca emite alvo fora do seletor e sempre respeita limites e tipos.
- Propriedades metamórficas verificam que permutar a ordem física de mapas/entidades não muda
  hash nem operações após normalização; ordenar inputs já ordenados mantém o resultado.
- Testes de determinismo executam os mesmos catálogo, estado, parâmetros, draws e comandos em
  builds/arquiteturas suportadas e comparam sequência canônica e hash de estado.
- Fixtures gravadas cobrem cada template publicado e os casos de falha, inclusive seleção vazia,
  referências removidas, conflitos, migração e fallback T0; CI não consulta LLM/API real.
- Golden replays incluem versão/hash de catálogo. Mudança intencional exige revisão do diff
  mecânico e atualização explícita da fixture; replays antigos não são sobrescritos em silêncio.
- Harness de simulação longa mede latência, alocações/memória, número de operações, invariantes
  do estado e estabilidade do orçamento com catálogos representativos e máximos.
- Teste arquitetural verifica dependências e símbolos do interpretador para bloquear eval,
  carregadores dinâmicos, acesso a relógio, rede, arquivos ou banco no módulo puro.

Estes testes rodam no workspace Rust e em CI; integração PostgreSQL
testa apenas publicação/leitura de conteúdo já versionado, e teste Godot valida apresentação
dos DTOs sem reimplementar a semântica mecânica.

## 11. Perguntas abertas

1. A forma canônica do arquivo-fonte deve ser YAML restrito ou JSON? Recomendação: YAML restrito
   para autoria humana e normalização para AST/JSON canônico antes de hash.
2. Qual é a política de compatibilidade de partidas salvas após atualização de catálogo?
   Recomendação: fixar hash por partida e manter versões antigas legíveis; migração explícita só
   quando necessária e sempre sem reescrever o log histórico.
3. Quais operações mecânicas entram na primeira versão além das necessárias aos templates de
   evento, tecnologia, tratado e governo? Recomendação: começar com conjunto mínimo, enum fechado,
   adicionar operação apenas com consumidor e testes de propriedade definidos.
4. Os limites iniciais de profundidade, nós, alvos e operações são adequados ao conteúdo esperado?
   Recomendação: aceitar como tetos provisórios e calibrar pelo harness antes da Fase 2.
5. Como arbitrar conflitos entre operações em um mesmo alvo e campo? Recomendação: rejeitar
   conflitos no validador, salvo quando a semântica de composição estiver definida explicitamente.
6. O runtime mantém catálogos legados durante toda a vida dos saves/replays? Recomendação:
   manter até política formal de retenção, exportando o manifesto e conteúdo referenciado em
   cada backup de partida.
7. Qual subsistema determina o fallback T0 quando template externo falha por tipo de conteúdo?
   Recomendação: cada consumidor declarar fallback tipado; fallback nunca pode inventar efeito
   mecânico nem bloquear o avanço do turno.

## 12. Relações

Este SDD detalha a lacuna apontada em `docs/sdd/REVISAO-CRUZADA.md` para DSL de catálogo.
Relaciona-se ao ADR-0006 (aceito), aos SDDs do núcleo, Entropia, tecnologia, diplomacia e governo;
com ADR-0007 aceito, a implementação usa a stack decidida sem tornar sua escolha parte
deste contrato. Tudo neste documento que não conste de decisão aceita é proposta para revisão.
