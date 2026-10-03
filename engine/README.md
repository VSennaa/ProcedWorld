# Engine

O diretório `engine/` contém o workspace Rust do motor headless determinístico do ProcedWorld.
Ele não acessa rede, relógio ou serviços de IA durante a simulação. Para uma mesma seed e a
mesma sequência de comandos, o hash final deve ser idêntico em qualquer plataforma.

## Estrutura

- `crates/pw-engine`: biblioteca com o núcleo determinístico: PRNG, hash de estado, grade hexagonal,
  geração de mundo, estado/regras e bots T0.
- `crates/pw-harness`: biblioteca e binário de simulação headless. Executa bots, grava log e snapshots
  para replay e imprime o hash final canônico.
- `../data/catalogs/`: catálogos JSON carregados pelo harness para gerar o mundo.

As únicas dependências externas dos crates são `serde` e `serde_json`, usadas para serialização e
leitura dos catálogos.

## Testes locais

Com Rust disponível, execute a partir deste diretório:

```sh
cargo test --workspace
```

O comando cobre todos os testes dos dois crates. A CI também o executa em Ubuntu quando arquivos do
motor, dos catálogos ou do workflow são alterados.

## Harness e replay

Ainda em `engine/`, uma rodada longa de referência é:

```sh
cargo run --release --package pw-harness -- run --seed 20261001 --civs 8 --turns 1000
```

O binário mostra hashes a cada 100 turnos e termina com `FINAL <hash>`. Ele grava o log e os snapshots
em `out/20261001/`. Para verificar o replay desse resultado:

```sh
cargo run --release --package pw-harness -- replay out/20261001
```

## Execução pela VPS

No Windows de desenvolvimento, o Smart App Control pode bloquear executáveis recém-compilados,
incluindo binários de testes e build scripts do Rust. Não tente contornar esse bloqueio: rode a
validação na VPS, que usa a imagem oficial do Rust em Docker e caches de Cargo.

A partir da raiz do repositório, informe o destino SSH por variável de ambiente e passe a raiz como
primeiro argumento do script:

```sh
VPS_SSH=deploy@<VPS_HOST> tools/dev/vps-test.sh . test --workspace
VPS_SSH=deploy@<VPS_HOST> tools/dev/vps-test.sh . run --release --package pw-harness -- run --seed 20261001 --civs 8 --turns 1000
```

Substitua `<VPS_HOST>` pelo alias ou destino SSH já configurado localmente. O script sincroniza somente
`engine/` e `data/`, sem `target/`, e não requer segredos do repositório.

## Critério de saída da Fase 2

A Fase 2 só pode ser encerrada após uma simulação de 1.000 turnos com 8 bots sem crash, replay
bit a bit idêntico e hash `FINAL` estável entre máquinas. O workflow `Engine` executa essa rodada em
Ubuntu, Windows e macOS e falha se os três hashes finais forem diferentes.
