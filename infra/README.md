# Infra

> Este repositório é **público**: nada de IP, hostname, chave ou `.env` real aqui.
> Use `<VPS_HOST>` nos documentos e um alias no seu `~/.ssh/config`.

## Host de referência (dev/testes)

- Debian 13, Docker Engine + Compose plugin, ufw (só 22 aberta), fail2ban.
- **2 vCPU, 2 GB RAM, 30 GB disco, sem GPU** (restrição registrada no ADR-0005 e no ADR-0007).
- Usuário de trabalho: `deploy` (grupos `sudo`, `docker`). Login root por senha desativado; use `deploy`.
- **Fonte da verdade de portas**: `/opt/infra/PORTS.md` **no host**. Nenhuma porta é publicada sem
  estar registrada lá.

## Convenções (resumo do PORTS.md do host)

- Cada projeto é uma stack em `/opt/stacks/<projeto>/`; o ProcedWorld usa `/opt/stacks/procedworld/`.
- Só o proxy reverso expõe 80/443 (ainda fechadas). Todo o resto fica em loopback ou rede Docker.
- **Docker ignora o ufw**: sempre `ports: ["127.0.0.1:81xx:porta"]`, nunca `"81xx:porta"`.
- Bancos e caches **sem** `ports:`; rede interna `procedworld_internal`. Rede compartilhada com o proxy: `proxy`.
- Acesso externo a painéis/DB: túnel SSH (`ssh -L 5432:127.0.0.1:5432 deploy@<VPS_HOST>`).
- Segredos em `/opt/stacks/procedworld/.env` com `chmod 600`, fora do git.

### Faixa do ProcedWorld: 8100–8199 (reservas, ainda não em uso)

| Porta | Serviço previsto | Exposição |
|---|---|---|
| 8100 | servidor autoritativo (API/WebSocket) | via proxy (443) |
| 8101 | admin/health | loopback |
| 8110 | harness de simulação / relatórios | loopback |
| 8150 | memória/contexto de IA (se separado) | loopback |
| — | PostgreSQL | sem porta publicada |

Ao publicar qualquer porta: atualizar `/opt/infra/PORTS.md` no host **no mesmo momento** e seguir o
checklist da seção 7 daquele arquivo.

## Desenvolvimento na VPS

Clone de trabalho: `~deploy/ProcedWorld` (separado da stack de deploy em `/opt/stacks/procedworld`).

```bash
ssh deploy@<VPS_HOST>
cd ~/ProcedWorld && git fetch && git status
```

Para dar **push** a partir da VPS, cadastre a chave pública do `deploy`
(`~deploy/.ssh/id_ed25519_github.pub`) no GitHub em *Settings → Deploy keys* do repositório,
marcando **Allow write access**. O clone já está configurado para fazer push via SSH com essa chave.

## Proxy reverso e quadro dos agentes (2026-10-02)

- **Caddy** (container `caddy`, rede `proxy`) é o único serviço com porta pública (80/443), com HTTPS
  automático (Let's Encrypt) no hostname da VPS. Config em `/opt/infra/caddy/Caddyfile` (no host, `chmod 600`).
- Hoje ele só serve o **quadro kanban dos agentes** (`pw-board`, nginx somente leitura) atrás de HTTP Basic.
  Endereço: `https://<hostname da VPS>/board.html`.
- **Senha rotacionada a cada sprint**: no host, `/opt/infra/caddy/rotate-board-password.sh` (cópia em
  `infra/scripts/`) gera uma senha nova, guarda só o hash bcrypt, recarrega o Caddy e imprime a senha uma vez.
- O quadro também continua acessível só por túnel em `127.0.0.1:8110`.

## Ainda não existe (vem no SDD/Fase 2+)

- `docker-compose.yml` da stack, proxy reverso com TLS, CI/CD de deploy, backups do PostgreSQL.
