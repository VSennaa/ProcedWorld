# STATUS — handoff entre sessões

> Atualizar ao fim de toda sessão (ver CLAUDE.md §9). Seção "Agora" sempre reflete o estado real.

## Agora

- **Fase**: 0 — Concepção (GDD)
- **Branch ativa**: `develop` (bootstrap mergeado). Próximo trabalho: `docs/gdd/<pilar>`.
- **Próximo passo**: redigir o GDD pilar a pilar, começando por `00-visao.md` e `01-loop-e-turnos.md`,
  levando as perguntas abertas de cada arquivo ao usuário.

## Feito

- 2026-10-01 — Bootstrap do repositório: CLAUDE.md, ADRs 0001–0007, esqueleto de GDD/SDD, ROADMAP,
  GLOSSARY, `infra/README.md`, higiene do repositório (`.gitignore`, `.gitattributes`, `.editorconfig`,
  template de PR). Branches `develop` e `docs/sdd/project-bootstrap` publicadas.
- 2026-10-01 — VPS inspecionada (somente leitura no host): Debian 13, Docker + Compose, usuário
  `deploy`, ufw só com 22, faixa 8100–8199 reservada para o ProcedWorld em `/opt/infra/PORTS.md`.
  Clone de trabalho criado em `~deploy/ProcedWorld`.

## Perguntas abertas (para o usuário)

1. Chave do modelo de decisão (Jev/Runware): do operador do servidor ou do jogador? (ADR-0002)
2. Ordem de ação de bots/Governadores dentro do turno simultâneo (ADR-0003).
3. Ratificar a stack proposta (ADR-0007).
4. Hospedagem futura do Laya, dado que a VPS atual não tem GPU (ADR-0005).

## Riscos conhecidos

- VPS com 2 GB de RAM: compilação Rust + PostgreSQL + agente de código podem disputar memória.
  Considerar swap ou build no CI antes da Fase 2.
- Para dar push a partir da VPS, a chave pública do `deploy` precisa estar cadastrada como
  Deploy Key (com escrita) no GitHub (ver `infra/README.md`).
