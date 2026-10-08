# Brief C1 — Segurança da persistência do servidor

Área: `engine/crates/pw-server/` apenas. Origem: ressalvas do juiz da A1 (STATUS, "Pendências do alfa mini").

1. **Tokens de sessão em repouso**: o disco guarda só um verificador (hash criptográfico com sal por
   token, comparação em tempo constante), nunca o token. Dependência nova só se madura e justificada
   no commit (ex.: `sha2`/`argon2`); migração: arquivos antigos com token em claro são convertidos na
   carga, e o teste prova que o token original continua entrando.
2. **`max_worlds` na restauração**: mundos além do limite não são carregados (ordem estável por id),
   com registro no stderr; teste.
3. **Criação de mundo atômica**: snapshot inicial, turno e metadados publicados juntos (diretório
   temporário + rename do diretório), e diretório incompleto de criação interrompida é ignorado na
   carga com aviso; teste.
4. Documentar em `engine/crates/pw-server/README.md` e no SDD 11 (segurança) o formato do verificador.

Regras: Rust só na VPS (`tools/dev/vps-test.sh`), SSH ≥ 60 s, não reformatar arquivos, nada de IP/host.
