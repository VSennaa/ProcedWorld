# Assets — protótipo de biomas, UI e entidades

Arte vetorial original feita por código, conforme GDD 13 e os 12 biomas do GDD 02 §2.
**Decidido:** orientação pointy-top e direção 2D estilizada. **Proposta:** esta paleta,
texturas, espessuras e símbolos; a aprovação visual definitiva permanece aberta.

## Gerar e visualizar

Na raiz do repositório, com Python 3 e somente biblioteca padrão:

```sh
python tools/assets/gen_tiles.py --seed 20261001
python tools/assets/gen_tiles.py --seed 20261001 --civilization-color "#56B4E9"
python tools/assets/gen_icons.py --seed 20261001
python tools/assets/gen_entities.py --seed 20261002 --civilization-color "#56B4E9"
python tools/agents/check-encoding.py assets/README.md
```

Abra `assets/preview.html` diretamente no navegador. Não precisa de servidor, rede, fontes
externas ou JavaScript. A página mostra todas as variações, paletas, overlays compostos
sobre terreno e dois mapas ilustrativos 10×8, repetidos em fundos claro e escuro.
Os mapas são uma composição de demonstração; não implementam a geração de mundo do motor. A seção
de UI reúne os ícones em fundos claro e escuro, com as três variantes de cor.
O estudo de entidades exibe unidades, edifícios, melhorias e marcadores de cidade.

## Arquivos

- `palette.json`: paleta compartilhada; cada bioma tem `[base, traço, destaque]` nos modos
  `standard` e `colorblind`. A chave `icons` define as cores semânticas padrão e alternativa,
  mais a tinta única para estados monocromáticos. Os geradores leem esse arquivo sem sobrescrevê-lo.
- `icons/standard/*.svg`: 26 ícones planos 48 × 48 na paleta padrão: recursos, indicadores
  sociais, tempo, agentes de IA, Ledger e ações diplomáticas.
- `icons/colorblind/*.svg`: os mesmos ícones na paleta alternativa. A silhueta permanece
  específica por conceito; cor nunca é o único sinal de significado.
- `icons/monochrome/*.svg`: os mesmos ícones em uma tinta, para status, botões compactos e
  superfícies onde a cor semântica não deve competir com o conteúdo.
- `icons/manifest.json`: seed registrada, dimensões, espessura de traço e relação entre cada
  arquivo, rótulo e papel de cor. `tools/assets/gen_icons.py` gera estes arquivos e substitui
  somente a seção `icons-preview` de `preview.html`.
- `entities/standard|colorblind|monochrome/`: ícones vetoriais 48 × 48 de todas as unidades e
  edifícios dos catálogos `core.units` e `core.buildings`, além de fazenda, mina, pastagem,
  serraria, porto e estrada. Silhuetas reforçam a categoria; cor não é o único sinal.
- `entities/*/cidades/`: quatro tamanhos propostos por população (pequena, média, grande e
  metrópole), combinados com os estados normal, protesto, revolta e cerco. A cor de civilização
  é parâmetro `--civilization-color` (`#RRGGBB`); formas também identificam estados.
- `entities/manifest.json`: seed, catálogos, tamanhos, estados e variante. O gerador
  `tools/assets/gen_entities.py` usa somente biblioteca padrão, é determinístico e atualiza
  somente a seção `entities-preview` de `preview.html`.

Os quatro portes de cidade são uma escala visual proposta; não definem faixas nem fórmulas de
população. A aprovação da iconografia e dos tamanhos permanece aberta conforme o GDD 13.
- `tiles/standard/<bioma>-<1|2|3>.svg`: 36 tiles na paleta padrão.
- `tiles/colorblind/<bioma>-<1|2|3>.svg`: 36 tiles na paleta alternativa para daltonismo.
  As mesmas formas identificam biomas nas duas paletas; cor não é o único indicador.
- `tiles/overlays/rio-<aresta>.svg`: seis meias faixas de rio na aresta.
- `tiles/overlays/fronteira-<aresta>.svg`: seis bordas tracejadas, para fronteiras externas
  de um território. `fronteira.svg` reúne todas as arestas para seleção/demonstração.
  `--civilization-color` aceita `#RRGGBB`; a paleta sugere quatro cores distinguíveis.
  O halo escuro preserva contraste. Identificar civilizações também por nome/emblema na UI
  continua necessário; este protótipo não implementa a UI de civilizações.
- `tiles/overlays/nevoa-lembrado.svg`: véu translúcido com relógio, para informação antiga.
- `tiles/overlays/nevoa-desconhecido.svg`: cobertura totalmente opaca com interrogação.
- `tiles/minimapa-<standard|colorblind>.svg`: composições 10×8 dos mesmos tiles.
- `tiles/manifest.json`: seed, geometria, ordem das arestas, variantes, biomas e cor de fronteira.
- `preview.html`: galeria responsiva; o gerador substitui somente a seção entre marcadores
  `tiles-preview:start` e `tiles-preview:end`, preservando outras seções existentes.

## Encaixe e composição

Raio `r = 64`, largura `w = sqrt(3) × 64 ≈ 110,851251684`, altura `128`.
Todos os tiles e overlays têm o mesmo `viewBox`, sem margem externa. O vértice superior
é `(w/2, 0)`. Arestas no sentido horário: **NE, E, SE, SW, W, NW** (índices 0–5).
A oposta é `(índice + 3) % 6`.

No grid odd-r, a origem superior esquerda de cada imagem é:

```text
x = w × (coluna + 0,5 × (linha % 2))
y = 96 × linha
```

Linhas ímpares deslocam meio hexágono para a direita. Não arredonde posições para pixels
inteiros. O mapa 10×8 tem `viewBox="0 0 1163.938142686 800"`. Não há separação geométrica
entre vizinhos; o preview pinta primeiro uma base com sobreposição de 0,4 unidade por lado
para evitar fios de antialiasing em zoom fracionário, depois desenha os tiles sem contorno.
Na integração, use o mesmo preenchimento de fundo ou uma malha compartilhada para evitar
frestas de rasterização entre imagens independentes.

Ordem: terreno → rios → fronteiras → névoa. Os overlays de aresta são recortados no hexágono:
para um rio compartilhado, aplique a aresta correspondente **nos dois tiles vizinhos**.
Cada tile fornece metade da espessura; não existe margem que altere o encaixe.
Fronteiras devem aparecer somente entre donos diferentes, não em toda célula do território.
O estado lembrado deve cobrir a última informação conhecida, nunca dados atuais secretos.

## Determinismo e acessibilidade

O algoritmo `tiles-v1` deriva amostras de SHA-256 sobre uma lista JSON canônica contendo
seed e chaves. Não usa relógio, `hash()` de processo ou PRNG global. Mesma seed, paleta,
parâmetros e versão do gerador produzem os mesmos bytes. Cada bioma possui três arquivos;
`choose_variant(seed, biome, coluna, linha)` escolhe um deles de maneira estável.
A seed também varia discretamente a posição e escala das formas nos três arquivos.
Não há dependência de ordem de chamadas. Este é um gerador de arte fora do motor.

Ondas, coníferas, copas largas, acácias, dunas, tufos, rochas, juncos, picos e cristais
diferenciam os terrenos por forma. A galeria exibe os tiles também com altura de 96 px.
Rios contínuos, fronteiras tracejadas e símbolos de visibilidade redundantes ajudam a leitura
sem depender de distinções vermelho/verde. A alternativa não substitui testes com jogadores.

## Ícones de UI

Todos têm `viewBox="0 0 48 48"`, traço de 3 unidades, extremidades e junções arredondadas. A escala
de referência é 48 × 48, mas os traços e as silhuetas foram simplificados para leitura a 24 × 24.
O gerador `icons-v1` é determinístico: a geometria é fixa e a seed informada fica registrada no
manifest, para que a mesma seed, paleta e versão resultem nos mesmos SVGs. Não usa fontes, imagens,
JavaScript ou bibliotecas externas.

Os itens de Ledger são três arquivos explícitos: `ledger-confianca`, `ledger-ressentimento` e
`ledger-divida`. Os símbolos de Mandato, Governador, Entropia e custo de IA distinguem as camadas
de decisão sem recorrer a texto dentro do ícone. Cores e nomes são proposta de protótipo; GDD 13
ainda deixa a iconografia definitiva em aberto.

## Licença

**CC0 1.0 Universal — obra própria.** O código de `tools/assets/gen_tiles.py` e
`tools/assets/gen_icons.py`, a paleta, os SVGs gerados e as seções de preview deste protótipo são dedicados ao domínio público,
na máxima extensão permitida por lei, sem exigência de atribuição. Nenhuma imagem,
fonte, textura ou outro asset de terceiros foi incorporado.
Texto legal: <https://creativecommons.org/publicdomain/zero/1.0/legalcode>.

Pergunta aberta: aprovação da paleta e das ilustrações para uso definitivo no cliente.
