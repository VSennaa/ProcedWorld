# Estudo — integrações técnicas

> Pesquisa em 2026-10-01. Este estudo orienta SDDs; não aprova biblioteca,
> protocolo, custo ou mudança de arquitetura. Preços e aliases de modelos devem
> ser conferidos de novo imediatamente antes de implementar, pois provedores os
> alteram sem compatibilidade de preço garantida [DS1].

## Premissas do ProcedWorld

- O servidor é autoritativo e o motor permanece uma função determinística;
  rede, banco, relógio e IA são bordas, nunca entradas implícitas de `step`.
- Comandos aceitos, resultados externos normalizados e versões de catálogo
  precisam entrar no log antes da resolução; replay não deve chamar provedor.
- O celular é uma projeção do estado autorizada pelo servidor, não uma segunda
  simulação concorrente. Isso preserva ADR-0001 e ADR-0006.
- A recomendação abaixo prioriza conexões recuperáveis, mensagens idempotentes,
  snapshots e deltas, em vez de sincronização de cena ou RPC do engine.

```text
Godot Android
  | WSS: autenticação, resume, comandos idempotentes, snapshot/delta
  v
Axum (borda HTTP/WebSocket) --> fila/ator serial por mundo
  |                                 |
  | PostgreSQL: log, snapshot,       v
  | sessão, tokens FCM          núcleo Rust puro
  |                                 |
  +--------------------------> valida intenção -> comando -> step -> hash
                                      |
                         portas de IA: DecisionPort / LLMPort
                           |                     |
                   Jev/System One          DeepSeek compatível
                   (T1; T0 fallback)       (T2; T0 fallback)
                                      |
                           outbox de notificação -> FCM -> Android
```

O diagrama descreve a separação proposta, não um produto pronto. O núcleo não
aguarda FCM nem a rede: uma falha de entrega apenas deixa de acordar o usuário.

## Godot 4 ↔ servidor Rust

### WebSocket no Godot

**O que é.** `WebSocketPeer` é a conexão WebSocket RFC 6455 de baixo nível do
Godot; pode enviar frames de texto ou binários e exige `poll()` periódico [G1].
`WebSocketMultiplayerPeer` integra WebSocket à `MultiplayerAPI` do engine [G2].

**Como funciona.** Para um servidor externo em axum, o cliente abre `wss://`,
faz o handshake de aplicação, recebe snapshot ou delta e envia envelopes de
comando com `client_command_id`. O socket transporta bytes; o servidor decide
sessão, revisão, autorização e ordem de aceitação. O `MultiplayerAPI` padrão
não é contrato para servidores não-Godot e pode mudar sem aviso [G3].

**Aproveitar.** Usar `WebSocketPeer` diretamente, frames de texto no início e
um subprotocolo versionado, por exemplo `procedworld.v1`. Explicitar limites de
mensagem, ping/fechamento, reconexão com `resume`, revisão do snapshot e erro
tipado. Na exportação Android, habilitar a permissão `INTERNET`, exigida pelo
Godot para qualquer comunicação de rede [G2].

**Evitar.** Não conectar `WebSocketMultiplayerPeer`/RPC de nós Godot a axum;
não usar a ordem de chegada, callback, frame rate ou relógio do telefone para
resolver o turno; não manter comandos apenas na memória do cliente.

**Afeta.** GDD 01 (turnos), GDD 11 (mobile); SDD 03 (sessão), SDD 10
(protocolo) e SDD 12 (cliente).

### Serialização e tipos compartilhados

**O que é.** JSON é texto legível e fácil de inspecionar; MessagePack é uma
família binária dinâmica especificada independentemente [S1]. Protobuf usa
schemas `.proto` e gerador de código para dados estruturados extensíveis entre
linguagens [S2]. FlatBuffers é formato/schema orientado a acesso sem cópia,
quando a plataforma e as bindings o suportam [S3].

**Como funciona.** JSON permite evoluir um envelope com `protocol_version`,
`kind`, `request_id`, `payload` e campos opcionais. MessagePack reduz bytes mas
não fornece, sozinho, um schema compartilhado. Protobuf/FlatBuffers partem de
IDL, geram tipos e exigem disciplina de evolução de campos. O cliente Godot e
Rust ainda precisam concordar sobre inteiros, opcionalidade, enum desconhecido
e canonicalização para hashing; o hash autoritativo continua do servidor.

| Opção | Pontos fortes | Custo/risco | Uso recomendado |
| --- | --- | --- | --- |
| JSON + JSON Schema | depuração, payload humano e contratos de IA | maior tráfego e parsing | protocolo v1 e intenções |
| MessagePack | menos bytes mantendo modelo dinâmico | schema/evolução ficam por conta da equipe | só após perfil de tráfego |
| Protobuf | IDL e geração de tipos multi-linguagem | toolchain e binding Godot a validar | candidato a v2, após spike |
| FlatBuffers | leitura potencialmente sem cópia | ergonomia e binding GDScript são o risco | não adotar sem gargalo medido |

**Aproveitar.** Começar com JSON versionado e JSON Schema como contrato
canônico. Gerar no Rust validação/structs a partir do schema quando houver
ferramenta aprovada, e gerar no Godot apenas DTOs finos ou validar com parser
local. Registrar fixtures de snapshot, delta, rejeição e reconexão; elas são a
fonte de compatibilidade entre os dois lados.

**Evitar.** Não compartilhar structs Rust diretamente por FFI, nem serializar
objetos Godot/`Variant` como protocolo público. Não trocar para binário apenas
por suposição de desempenho, nem fazer JSON canônico de rede equivaler ao hash
do motor.

**Afeta.** GDD 01 e GDD 11; SDD 01 (núcleo), SDD 10, SDD 12 e SDD 14 (testes).

### godot-rust / gdext

**O que é.** `godot-rust`/`gdext` são bindings comunitários de Rust para Godot
4 sobre a API C GDExtension [R1]. O projeto se declara independente do Godot e
informa suporte experimental para Android e iOS, com documentação/tooling ainda
incompletos [R2].

**Como funciona.** Uma biblioteca dinâmica Rust é carregada por arquivo
`.gdextension`; ela expõe classes chamáveis pelo GDScript. O guia exige Godot
4.1 ou posterior para o exemplo de integração [R3]. Isso é FFI local, não a
ponte de tipos do servidor nem uma forma de tornar o motor do telefone
autoritativo.

**Aproveitar.** Usar somente após spike para uma necessidade local comprovada:
codec muito quente, geração de malha/tiles ou biblioteca Rust sem alternativa
segura em GDScript. Manter a interface pequena, a lógica visual substituível e
CI de exportação Android para cada versão de Godot.

**Evitar.** Não usar gdext como caminho inicial para compartilhar o núcleo Rust;
isso duplicaria o risco de build/ABI mobile e sugeriria indevidamente que o
cliente pode resolver o mundo. Não depender dele para rede, autenticação ou
segredos.

**Afeta.** GDD 02 (mapa) e GDD 11; SDD 01, SDD 10 e SDD 12.

## Multiplayer por turnos: referências

### Unciv

**O que é.** Unciv documenta multiplayer baseado em upload/download de arquivo
de save; seu guia também permite hospedar um servidor próprio [U1]. Participantes
compartilham um identificador de jogo para entrar na mesma partida [U1].

**Como funciona.** A sincronização centrada em save simplifica partidas
assíncronas: o participante baixa o estado, joga e publica o save seguinte.
Ela pressupõe versões/mods compatíveis entre participantes para reduzir erros
de interpretação [U2].

**Aproveitar.** A simplicidade operacional de um snapshot verificável, o ID de
partida e a recuperação por download integral inspiram o fallback do
ProcedWorld: se um delta falhar, pedir snapshot autoritativo e substituir a
projeção local.

**Evitar.** Não tornar save mutável do cliente a fonte de verdade nem aceitar
upload de estado para o servidor. Isso não fornece a ordem de comandos,
validação, replay, segurança de chaves ou o motor autoritativo exigidos aqui.

**Afeta.** GDD 01, GDD 09 e GDD 11; SDD 03, SDD 09, SDD 10 e SDD 12.

### Freeciv

**O que é.** Freeciv é um 4X livre com cliente e processo de servidor; a própria
documentação recomenda servidor separado para a partida persistir se o cliente
do host encerrar [F1].

**Como funciona.** No modo descrito, humanos movem simultaneamente, depois IAs
jogam após todos os humanos concluírem; o servidor tem timeout configurável
[F2]. Em multiplayer assíncrono, ações recebidas no turno são processadas em
ordem de chegada; Freeciv reconhece a vantagem de rapidez e oferece modos/
opções para mitigá-la [F3].

**Aproveitar.** Separar servidor de cliente, publicar estado/prontidão e gravar
ordem aceita. A crítica de Freeciv à vantagem de “quick fingers” confirma que
o ProcedWorld deve mostrar conflito de capacidade e usar a fase de combate
simultâneo já definida, sem fingir que chegada de pacote é justiça.

**Evitar.** Não introduzir timeout real porque Freeciv o oferece: ADR-0008
define turno sem relógio. Não permitir que tarefas de IA travem o fechamento;
T0 deve produzir a intenção de fallback.

**Afeta.** GDD 01, GDD 07 e GDD 11; SDD 03, SDD 05, SDD 07 e SDD 10.

## Portas de decisão e de LLM

### Jev / System One e a alegação Runware

**O que é.** A documentação encontrada para System One descreve acesso HTTP a
Jev para escolhas, scores e probabilidades sim/não, com JSON de estado e
perguntas nomeadas [J1]. Ela define o endpoint hospedado como
`https://system-one.dev/v1`, e `POST /v1/systemone` [J1].

**Como funciona.** Uma requisição contém `model`, `state` e de 1 a 32 perguntas
dos tipos `choice`, `score` ou `noul`; `choice` recebe opções nomeadas, `score`
recebe de 2 a 10 níveis ordenados [J1]. A resposta traz a escolha/score ou
probabilidade, distribuição e confiança conforme a primitiva [J1]. Cabeçalho
`Idempotency-Key` é opcional e o `X-Request-Id` identifica a requisição no
ledger da plataforma [J1].

**Estado de verificação.** Não foi encontrada fonte oficial que associe Jev ou
Laya à **Runware**, nem que confirme `runware:laya@1`, durante esta pesquisa.
As fontes oficiais encontradas associam `/v1/systemone` a System One/TypeSafe,
não a Runware [J1]. Portanto os identificadores “Jev via Runware” e “Laya via
Runware” estão **não verificados** e exigem spike sem chave real antes de virar
contrato de `DecisionPort`.

**Aproveitar.** Modelar `DecisionPort.evaluate(state, questions, idempotency)`
como adaptador: permitir `choice`, `score` e probabilidade, validar domínio,
registrar id do provedor, versão resolvida, orçamento e saída normalizada antes
de converter em intenção. Fixar ID de versão quando a reprodutibilidade for
necessária, pois aliases podem resolver versões diferentes no tempo [J1].

**Evitar.** Não passar texto do jogador como instrução, não deixar probabilidade
alterar estado diretamente e não assumir que confiança é calibração. Não chamar
o endpoint em replay ou CI; gravar a decisão normalizada e usar T0 em timeout,
erro, teto ou schema inválido.

**Afeta.** GDD 07, GDD 08 e GDD 09; SDD 04 (IA), SDD 05, SDD 06, SDD 07,
SDD 08, SDD 09 e SDD 14.

### Jev Router no OpenRouter

**O que é.** O Jev Router é um roteador de modelo do OpenRouter identificado por
`typesafe/jev-router`; ele seleciona modelo e esforço de raciocínio de uma lista
controlada [O1]. Não é o mesmo contrato do endpoint de decisões do System One.

**Como funciona.** A documentação o apresenta no endpoint Chat Completions e
informa que o campo `model` da resposta revela o modelo que efetivamente serviu
a chamada [O1]. Isso roteia uma chamada de linguagem; não transforma Jev em
gerador de prosa nem substitui `DecisionPort` do jogo.

**Aproveitar.** Usá-lo, se aprovado, apenas fora da partida para escolher modelo
de ferramenta/agente de desenvolvimento ou como experimento de roteamento de
`LLMPort`; registrar modelo resolvido e custo. Manter allowlist e teto.

**Evitar.** Não usar o Router como juiz autoritativo, não confiar em alias para
replay e não misturar seu resultado com o contrato T1 de escolha tipada.

**Afeta.** SDD 04, SDD 08, SDD 13 e SDD 14; indiretamente GDD 08 e GDD 09.

### DeepSeek

**O que é.** DeepSeek oferece modelos por API; a documentação de preços atual
separa tokens de entrada com cache hit, entrada sem cache e saída, e avisa que
produtos/preços podem mudar [DS1]. A compatibilidade com API OpenAI é alegada
em anúncio oficial anterior da DeepSeek [DS3], mas deve ser confirmada por spike
contra os endpoints e recursos exatos necessários ao `LLMPort`.

**Como funciona.** O cache em disco é habilitado por padrão. Um hit exige que o
prefixo completo já tenha sido persistido; a documentação expõe
`prompt_cache_hit_tokens` e `prompt_cache_miss_tokens` em `usage` [DS2]. O
cache é best-effort, pode ser limpo após horas/dias sem uso e não torna a saída
determinística [DS2].

**Preços observados.** Na tabela acessada em 2026-10-01, `deepseek-flash` tinha
por 1M tokens: hit US$ 0,003 off-peak / US$ 0,006 peak; miss US$ 0,15 / US$ 0,30;
saída US$ 0,60 / US$ 1,20. A mesma página lista para a segunda coluna de modelo
hit US$ 0,022 / US$ 0,044; miss US$ 0,66 / US$ 1,32; saída US$ 1,98 / US$ 3,96
[DS1]. Como o extrato não nomeia com segurança essa segunda coluna, não usá-la
como orçamento até conferência manual na página de preços. Horários peak/off-peak
informados ali são UTC e dias úteis [DS1].

**Aproveitar.** Montar mensagens com prefixo estável: versão de regras,
instruções fixas, schema e documentos canônicos primeiro; anexar estado variável
depois. Medir hits/misses por chamada e por civilização. Tratar o cliente OpenAI
compatível como detalhe de adaptador, mantendo timeout, retry idempotente,
limite de tokens e parser de intenção no servidor.

**Evitar.** Não presumir 100% de hit, custo estável ou saída reexecutável. Não
enviar API key ao aparelho, prompt bruto ao log, nem deixar texto narrativo
decidir efeitos mecânicos.

**Afeta.** GDD 07, GDD 08, GDD 09 e GDD 11; SDD 04, SDD 05, SDD 06, SDD 08,
SDD 11, SDD 13 e SDD 14.

## Android, push e chaves

### FCM a partir de servidor auto-hospedado

**O que é.** Firebase Cloud Messaging é o serviço de entrega de mensagens do
Firebase. No Android, o app registra token e o envia ao servidor da aplicação;
para tratar dados e mensagens em primeiro plano, a documentação pede estender
o serviço correspondente [P1].

**Como funciona.** O servidor guarda token FCM associado ao dispositivo/conta,
gera uma notificação a partir de uma transição já persistida e envia uma mensagem
de dados mínima. O Android recebe, apresenta canal/notificação conforme seu
estado e, ao abrir, busca snapshot/delta autenticado do ProcedWorld. Android 8+
suporta/recomenda canais de notificação [P1]. FCM orienta que uma data message
em background tem poucos segundos para mostrar notificação; payload deve ser
pequeno e não é transporte de estado confiável [P2].

**Aproveitar.** Criar uma outbox transacional com evento `turn_ready` ou
`mandate_attention`, deduplicada por mundo/turno/categoria; um worker externo
ao `step` envia para FCM e registra tentativa/resultados. Payload: só ID opaco,
turno, categoria e revisão; o app sempre reconcilia via API. Permitir preferência
do usuário e agrupar crises conforme GDD 11.

**Evitar.** Não colocar estado, Mandato, narrativa, API key, token de sessão ou
termos diplomáticos no push. Não fazer o avanço do turno depender de entrega e
não tratar token FCM como identidade/autorização. Não usar a antiga Server Key;
a documentação oficial atual deve ser consultada no spike de credencial HTTP v1.

**Afeta.** GDD 01, GDD 09 e GDD 11; SDD 03, SDD 10, SDD 12, SDD 13 e SDD 17.

### Android Keystore e envelope no servidor

**O que é.** Android Keystore guarda chaves criptográficas de forma mais difícil
de extrair; o material pode permanecer não exportável e ter uso restrito por
algoritmo/finalidade/autenticação [K1]. Alguns aparelhos vinculam chaves a TEE
ou StrongBox, mas isso depende do hardware e deve ser consultado por `KeyInfo`
[K1].

**Como funciona.** Para sessão local, gerar uma chave simétrica no Keystore e
usá-la para cifrar token de sessão ou cache apagável, com rotação ao logout. A
chave BYOK não deve precisar existir no telefone após onboarding: cliente entrega
uma vez por TLS ao servidor autenticado; o servidor cifra o segredo com uma DEK
aleatória (AEAD), cifra/“embrulha” a DEK com KEK de KMS/segredo operacional e
persiste `ciphertext`, nonce, tag, versão da chave e AAD de escopo. Para uso,
desembrulha em memória, chama o provedor e zera buffers quando viável.

**Aproveitar.** AAD deve vincular, por exemplo, `account_id`, `provider` e
`key_version`, impedindo troca silenciosa de ciphertext entre contas. Separar
KEK do banco, rotacionar por versão, registrar só IDs/resultado da rotação,
limitar descriptografia ao worker de IA e revogar/apagar criptograficamente na
remoção da chave. A Keystore protege material local, não torna dispositivo
comprometido confiável [K1].

**Evitar.** Não guardar BYOK em `SharedPreferences`, cache, crash report,
notificação, log ou APK. Não chamar provedor a partir do app. Não usar uma
chave global hard-coded como KEK, nem inventar criptografia; selecionar AEAD/KMS
em ADR e fazer revisão de segurança antes de produção.

**Afeta.** GDD 09 e GDD 11; SDD 04, SDD 11, SDD 12, SDD 13 e SDD 17.

## axum + tokio + sqlx para o servidor por turnos

### Papel de cada componente

**O que é.** Axum expõe extrator `WebSocketUpgrade` e faz upgrade chamando um
callback assíncrono [A1]. Tokio é runtime assíncrono de Rust [T1]. SQLx fornece
pool assíncrono; `PgPool` reutiliza conexões e permite adquirir conexão ou
iniciar transação [Q1].

**Como funciona.** Axum autentica e decodifica na borda. Cada mundo tem um ator
ou fila serial que recebe pedidos já validados superficialmente; ele abre uma
transação curta, verifica revisão/idempotência/autoridade, anexa comando, resolve
quando elegível, persiste hash/snapshot/outbox e só então publica delta. Tokio
coordena sockets, workers de IA e outbox, mas não decide ordenação sem a fila do
mundo. SQLx limita conexões; quando o pool atinge seu máximo, tarefas aguardam
uma conexão disponível [Q2].

**Padrões recomendados.**

1. `WorldActor(world_id)` é o único escritor lógico do mundo; não segurar lock
   de mundo através de chamada de IA, FCM, rede ou espera humana.
2. A transação de aceitação insere `client_command_id` único por sessão e
   `log_seq` monotônico; repetição retorna o resultado original, não novo efeito.
3. A resolução lê somente dados já persistidos/normalizados; grava comando,
   hash, snapshot eventual e outbox na mesma unidade transacional possível.
4. A chamada T1/T2 ocorre antes como job com deadline; resultado vira intenção
   registrada ou fallback T0. Nunca roda dentro de `step`.
5. WebSocket é assinante de projeção. Desconexão só remove assinante; o estado
   persistido e o turno sem relógio continuam coerentes.
6. Separar pools/limites para tráfego de jogo, jobs auxiliares e migrações;
   dimensionar por medição na VPS, não pela concorrência teórica do Tokio.

**Aproveitar.** `WebSocketUpgrade`/`on_upgrade` para a conexão e `PgPool` para
recursos compartilhados, mantendo handlers finos. A documentação do axum mostra
que socket pode ser dividido para leitura/escrita concorrentes [A2]; usar isso
somente depois de associar um único canal ordenado de saída à sessão.

**Evitar.** Não fazer handler HTTP executar um turno inteiro em paralelo com
outro handler do mesmo mundo; não criar transação longa em torno de IA; não
usar broadcast como garantia de persistência, nem basear `log_seq` em timestamp.
Não executar consultas sem limite/timeout numa VPS pequena.

**Afeta.** Todos os sistemas de sessão e persistência: GDD 01, GDD 07, GDD 08,
GDD 09 e GDD 11; SDD 01, SDD 03–11, SDD 13, SDD 14 e SDD 17.

## Recomendações para o ProcedWorld

1. Especificar no SDD 10 um protocolo JSON/WSS v1 com envelope versionado,
   `client_command_id`, revisão, `resume`, snapshot e delta; implementar teste
   de fixtures Rust↔Godot antes de considerar MessagePack/Protobuf.
2. Fazer o servidor axum aceitar WebSocket somente como transporte e centralizar
   cada mundo em fila/ator serial persistente; `log_seq` e idempotência são
   autoridade, não a ordem de tarefas Tokio.
3. Persistir `CommandAccepted`, resultado externo normalizado, hash e outbox
   antes de publicar projeção; reconexão sempre pode cair para snapshot integral.
4. Criar `DecisionPort` com adaptador System One e T0; marcar Runware/Laya como
   spike pendente porque a associação e o identificador não foram verificados.
5. Criar `LLMPort` DeepSeek com prefixo estável, telemetria de cache hit/miss,
   teto por civilização e fixture gravada; revalidar preço/modelo no momento da
   implementação e tratar compatibilidade OpenAI como teste de contrato.
6. Usar FCM somente para despertar: outbox idempotente, payload opaco mínimo e
   reconciliação autenticada ao abrir; nenhuma entrega é pré-condição do turno.
7. Implementar Android Keystore apenas para sessão/cache local e envelope
   criptográfico no servidor para BYOK; formalizar algoritmo, KMS/KEK, rotação e
   exclusão em ADR antes de armazenar qualquer chave real.
8. Não adotar godot-rust no caminho crítico inicial. Abrir spike de exportação
   Android somente se houver perfil que prove gargalo de GDScript/codec.
9. Incluir no SDD 14 testes de repetição, desconexão, delta faltante, replay
   sem IA, timeout de porta e colisão de comandos; CI não chama FCM ou provedores.

## Fontes

- [G1] Godot, [WebSocketPeer](https://docs.godotengine.org/en/4.0/classes/class_websocketpeer.html).
- [G2] Godot, [WebSocketMultiplayerPeer](https://docs.godotengine.org/en/4.1/classes/class_websocketmultiplayerpeer.html).
- [G3] Godot, [MultiplayerPeer](https://docs.godotengine.org/en/4.0/classes/class_multiplayerpeer.html).
- [S1] MessagePack, [especificação](https://github.com/msgpack/msgpack/blob/master/spec.md).
- [S2] Protocol Buffers, [visão geral](https://protobuf.dev/overview/).
- [S3] FlatBuffers, [documentação](https://flatbuffers.dev/).
- [R1] godot-rust, [livro gdext](https://godot-rust.github.io/book/).
- [R2] godot-rust, [repositório gdext](https://github.com/godot-rust/gdext).
- [R3] godot-rust, [Hello World](https://godot-rust.github.io/book/intro/hello-world.html).
- [U1] Unciv, [documentação de multiplayer](https://github.com/yairm210/Unciv/blob/master/docs/Other/Multiplayer.md).
- [U2] UncivCN, [guia de multiplayer](https://club.unciv.cn/Unciv/Community/Guides/Multiplayer-tutorial/).
- [F1] Freeciv, [FAQ](https://github.com/thejhh/freeciv/blob/master/doc/FAQ).
- [F2] Freeciv, [README](https://github.com/thejhh/freeciv/blob/master/doc/README).
- [F3] Freeciv, [FAQ: multiplayer assíncrono](https://github.com/thejhh/freeciv/blob/master/doc/FAQ).
- [J1] System One, [referência da API](https://docs.system-one.dev/en/docs/api).
- [O1] OpenRouter, [Jev Router](https://openrouter.darenbot.com/docs/guides/routing/routers/jev-router).
- [DS1] DeepSeek, [modelos e preços](https://api-docs.deepseek.com/quick_start/pricing/?helper=penn&method=individual).
- [DS2] DeepSeek, [cache de contexto](https://api-docs.deepseek.com/guides/kv_cache/).
- [DS3] DeepSeek, [anúncio V3/API compatibility](https://api-docs.deepseek.com/news/news1226/).
- [P1] Firebase, [FCM no Android](https://firebase.google.com/docs/cloud-messaging/android/get-started).
- [P2] Firebase, [criptografia de mensagens FCM](https://firebase.google.com/docs/cloud-messaging/encryption).
- [K1] Android Developers, [Android Keystore](https://developer.android.com/privacy-and-security/keystore).
- [A1] axum, [WebSocketUpgrade](https://docs.rs/axum/latest/axum/extract/struct.WebSocketUpgrade.html).
- [A2] axum, [módulo WebSocket](https://docs.rs/axum/latest/axum/extract/ws/).
- [T1] Tokio, [tutorial](https://tokio.rs/tokio/tutorial).
- [Q1] SQLx, [pool](https://docs.rs/sqlx/latest/sqlx/pool/).
- [Q2] SQLx, [Pool](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html).
