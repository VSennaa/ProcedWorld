# Brief A1 — Persistência em disco e catálogo de eventos/melhorias no servidor

Área: `engine/crates/pw-server/` (e `pw-engine/src/world.rs` só em `client_catalog()` se for o caso).
Contrato: SDD 10 §5.2 e o parágrafo "Alfa mini (2026-10-03)"; SDD 09 (persistência) como referência.

## Entregas

1. `FileStore` implementando o trait de `store.rs` (hoje só `InMemoryStore`): um diretório por mundo
   com snapshot do estado e o log de comandos aceitos (JSON), escrita atômica (arquivo temporário +
   rename). Configuração `PW_DATA_DIR` em `config.rs`; sem ela, continua em memória.
2. Ao iniciar com `PW_DATA_DIR`, o servidor recarrega os mundos salvos; o replay do log a partir do
   estado inicial tem de dar o mesmo hash do snapshot (se divergir, recusa carregar aquele mundo e
   registra o motivo no stderr).
3. Tokens de sessão sobrevivem ao reinício (mesmo diretório), para o cliente reentrar com "Entrar".
4. `catalog` com `event_templates` (`id`, `name`, `category`, texto narrativo se o catálogo tiver,
   `choices` com `id` e rótulo PT-BR) e `improvements` (`id`, `name`, `cost`, `requires_technology`,
   `biomes`), lidos dos catálogos em `data/catalogs/` que o servidor já embute.
5. Testes: salvar e recarregar mantém hash e turno; replay divergente é recusado; catálogo traz os
   campos novos. `cargo test --workspace` verde.

## Regras

- Sem I/O dentro do `step`; persistência é do servidor. Nada de IP/host em arquivos.
- Atualize o README do servidor com `PW_DATA_DIR`.
