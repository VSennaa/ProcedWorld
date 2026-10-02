# ADR-0005 — Modelo de decisão (Jev → Laya) consumido pelo servidor

- **Status**: Aceito. Laya só substitui o Jev **ao final**, quando o servidor for preparado para distribuição (facilitar o setup de quem hospeda); até lá, Jev. Decidido em 2026-10-01.
- **Data**: 2026-10-01
- **Decisores**: usuário

## Contexto

Decisões frequentes e limitadas (escolha entre opções, score, sim/não) são caras e lentas com LLM.
Modelos de decisão resolvem isso. Jev (TypeSafe AI) é uma API gerenciada; Laya (Convai) tem pesos
abertos (Apache 2.0). Os dois usam o mesmo endpoint da Runware (`/v1/systemone`) e as mesmas três
primitivas, com identificadores `typesafe:jev@latest` e `runware:laya@1`.

## Decisão

- A camada T1 usa **Jev** inicialmente e migrará para **Laya**.
- Toda chamada acontece **no servidor**, atrás da porta `DecisionPort`. O cliente nunca chama esses modelos.
- Trocar de modelo é configuração (`DECISION_MODEL=...`), não código.
- Respostas são gravadas no log de comandos (replay sem nova chamada).

## Restrição de hardware registrada

O host de referência atual tem **2 vCPU, 2 GB de RAM e nenhuma GPU**. Os números publicados do Laya são
em GPU (Tesla T4). Rodar o Laya localmente **nesse host** provavelmente é inviável. Opções quando
chegar a hora: (a) Laya via Runware, (b) servidor com GPU, (c) medir inferência em CPU num host maior.
Decidir com um spike medido antes da migração.

## Consequências

- O SDD define as perguntas T1 como dados versionados (texto da pergunta + opções/escala).
- Probabilidades só controlam ações relevantes depois de calibradas em dados de teste; a calibração
  precisa ser refeita na troca Jev → Laya.
