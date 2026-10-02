# ADR-0002 — Governador é um bot do jogo, executado no servidor

- **Status**: Aceito (sub-decisão sobre chaves: Proposto); quando o Governador joga: ver ADR-0008
- **Data**: 2026-10-01
- **Decisores**: usuário

## Contexto

O jogador nem sempre está jogando, mas o mundo continua. Era preciso definir quem age pela civilização
nesses momentos e onde essa IA roda.

## Decisão

- O **Governador** é um bot que faz parte do jogo. O jogador o **configura** pelo **Mandato**
  (prioridades, proibições, limites, postura diplomática, quando notificar o jogador) para que tome
  decisões por ele.
- O Governador **sempre roda no servidor** (consequência do ADR-0001). Ele age quando o jogador está
  ausente, quando o tempo do turno expira, ou em áreas que o jogador delegou explicitamente.
- Bots de civilizações sem jogador usam o mesmo mecanismo do Governador, com um Mandato gerado.
- O motor bloqueia qualquer ação que viole o Mandato; a restrição não depende do prompt.

## Sub-decisão proposta — chaves de API (confirmar no SDD)

Como toda IA roda no servidor, as chamadas de LLM (T2) para a civilização do jogador saem do servidor:

- A API key informada pelo jogador é enviada ao servidor por canal TLS e **armazenada criptografada**
  (criptografia de envelope; chave-mestra fora do banco, em segredo do servidor).
- A chave só é usada em chamadas da civilização daquele jogador, com teto de custo diário configurável.
- Sem chave válida ou com teto atingido, o Governador degrada para T0/T1 — o jogo continua.
- Em aberto: a chave do modelo de decisão (Jev/Runware) é do operador do servidor ou do jogador.

## Consequências

- O servidor guarda segredos de terceiros: exige rotação da chave-mestra, logs sem segredos, e uma
  ação para o jogador apagar a chave.
- O custo por jogador precisa ser medido e exibido.
