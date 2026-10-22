# Xiri Launcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Entregar um Xiri Launcher Windows x64 com a interface do ArtCraft e catálogo público de `sugyua`, incluindo instalação de pacotes compatíveis e abertura de programas locais.

**Architecture:** Preservar o workspace Rust e a separação entre core, backend nativo e egui. Converter o catálogo fixo em dados próprios carregados pelo backend e enviados à interface por eventos. Reutilizar verificações de download e instalação, adicionando contrato explícito de pacote Xiri.

**Tech Stack:** Rust >= 1.90, egui/eframe 0.36, serde, HTTP existente, Windows PowerShell para empacotamento, GitHub Actions.

**Spec:** [Especificação aprovada](../specs/2026-10-22-xiri-launcher-design.md).

## Global Constraints

- Alvo inicial: Windows x64; manter compilação wasm do workspace.
- Preservar grade, tabela, detalhes, busca, configurações e estados da interface original.
- Marca própria: Xiri Launcher, `sugyua/xiri-launcher`, `%LOCALAPPDATA%\Xiri Launcher\apps`.
- Consultar apenas repositórios públicos de `sugyua`; excluir forks, arquivados, launcher e perfil.
- Sem credenciais embutidas ou token obrigatório; cache e ETag para consultas públicas.
- Instalar apenas pacote suportado com checksum SHA-256 publicado e entrada explícita.
- Não presumir autossuficiência dos pacotes existentes do Destinia.
- Preservar dados do usuário e não substituir programas em execução.
- Sem panic, unsafe ou elevação silenciosa; manter limites de rede e extração.
- Manter licenças/avisos e retirar símbolos registrados ArtCraft dos recursos utilizados pelo produto.
- Não alterar os repositórios Destinia existentes.

## Review Focus

1. Manifesto muda depois da release: entrada e checksum usados na instalação precisam pertencer ao pacote validado (tarefa 4).
2. GitHub responde parcialmente ou limita a API: preservar catálogo anterior e abertura local (tarefa 3).
3. Repositório some ou muda de nome: manter instalação e dados locais sem remover arquivos automaticamente (tarefas 2 e 5).
4. Caminhos Windows com espaços, UNC, ADS e diferenças de caixa: validar entradas e preservar argumentos sem execução via shell (tarefas 1 e 4).
5. Interface vazia ou com muitos repositórios: não travar nem mostrar aplicativos ArtCraft como alternativa (tarefa 5).

## Mapa de arquivos

- `crates/core/src/catalog.rs`: modelo de aplicativo próprio e identidade Xiri.
- Novo `crates/core/src/xiri_manifest.rs`: contrato e validação do manifesto de pacote.
- `crates/core/src/protocol.rs`: evento de catálogo e comandos de associação local.
- Novo `crates/native/src/repositories.rs`: listagem GitHub paginada e metadados.
- Novo `crates/native/src/catalog_cache.rs`: cache persistente independente de instalações.
- `crates/native/src/source.rs`, `http.rs`, `backend.rs`: releases, HTTP limitado e integração assíncrona.
- Novo `crates/native/src/xiri_package.rs`: validação, extração e promoção de ZIP.
- `crates/native/src/install.rs`, `manifest.rs`, `process.rs`, `paths.rs`: integrar pacotes e estado instalado.
- `crates/ui-egui/src/model.rs`, `app.rs`, `views.rs`, `details.rs`, `icons.rs`, `bar.rs`: renderizar catálogo dinâmico e identidade.
- `apps/artcraft-launcher/src/`, `build.rs`, manifests Cargo, `packaging/windows/`: renomear shell/pacote para Xiri.
- `.github/workflows/ci.yml`, `release.yml`, README e atribuições: verificar e distribuir.

## Tarefa 1: Contrato do catálogo e dos pacotes

**Arquivos:** modificar `crates/core/src/{catalog,protocol,lib}.rs`; criar `crates/core/src/xiri_manifest.rs` e `crates/core/tests/xiri_manifest.rs`.

**Interfaces:** `CraftApp` passa a possuir seus textos (`String`) e aliases (`Vec<String>`); consumidores usam referências/clones, sem `Box::leak`. `XiriManifest` tem `schema_version: u32`, `visible: bool` (padrão true), `title: Option<String>`, `category: AppCategory`, `description: Option<String>`, `icon: Option<String>`, `windows: Option<WindowsPackage>`. `WindowsPackage` tem `asset: String`, `entry: PackageEntry`; `PackageEntry` tem `kind: EntryKind` (exe ou powershell) e `path: String`. Exportar `parse_xiri_manifest(bytes: &[u8]) -> Result<XiriManifest, ManifestError>` e `validate_entry_path(path: &str) -> Result<(), ManifestError>`.

- [ ] Escrever testes: schema 1 aceito; versão desconhecida recusada; título ausente permitido; manifesto sem entrada permite listagem; `../x`, `/x`, `C:\x`, UNC, `file:stream`, caminho vazio e componentes reservados Windows recusados; `MinhaVersao/Painel.ps1` aceito.
- [ ] Executar `cargo test -p artcraft-launcher-core --test xiri_manifest`; confirmar falha pela ausência do contrato.
- [ ] Implementar tipos e validação; exportar módulo; migrar modelo, fixtures e consumidores de `Copy`/`'static` para dados próprios, mantendo o workspace compilável. Constantes da identidade ficam separadas do catálogo dinâmico; remover dependência de `SUITE` em produção. Usar fallback de ícone próprio para apps sem ícone válido.
- [ ] Executar testes core e `cargo check --workspace`; confirmar aprovação. Se Rust não estiver instalado, preparar CI na tarefa 6 e registrar a falta de evidência local, sem afirmar que compilou.
- [ ] Commit: `feat: define owned catalog and Xiri package manifest`.

## Tarefa 2: Origem GitHub pública e paginada

**Arquivos:** criar `crates/native/src/repositories.rs`, `crates/native/tests/repositories.rs` e fixtures em `crates/native/tests/fixtures/`; modificar `http.rs`, `source.rs`, `lib.rs`, `crates/core/src/catalog.rs`.

**Interfaces:** `RepositoryClient::fetch_catalog(&self) -> Result<Vec<CatalogRecord>, CatalogError>`; `CatalogRecord { app: CraftApp, manifest: Option<XiriManifest>, default_branch: String }`. Usar infraestrutura HTTP existente, com transporte substituível por fixture. A identidade do registro é o nome completo `sugyua/repo`, sem comparação aproximada.

- [ ] Testar duas páginas, descrição nula, nomes com caixa diferente, filtros de proprietário/fork/arquivado/perfil/launcher e `visible=false`. Assertar que `destinia` e `Destinia_PCport` são preservados como itens distintos; repo removido não gera comando de desinstalação.
- [ ] Rodar `cargo test -p artcraft-launcher-native --test repositories`; confirmar falha inicial.
- [ ] Consultar `/users/sugyua/repos?per_page=100&page=N`, interrompendo em página incompleta e impondo teto de 20 páginas/2.000 itens e 4 MiB por resposta. Buscar `xiri.json` na branch padrão por endpoint GitHub validado, com teto 64 KiB; 404 significa manifesto ausente. Manifesto inválido desabilita instalação e preserva acesso ao repo com erro visível.
- [ ] Remover uso de feeds, screenshots e serviços ArtCraft na produção Xiri. Consultar releases estáveis no repo correto. Restringir downloads ao proprietário e repositório do registro, validando redirecionamentos ao CDN GitHub autorizado, sem ampliar hosts arbitrariamente.
- [ ] Rodar os testes de repositórios e os testes existentes de source/HTTP; confirmar aprovação.
- [ ] Commit: `feat: discover sugyua repositories and releases`.

## Tarefa 3: Cache e integração assíncrona

**Arquivos:** criar `crates/native/src/catalog_cache.rs`; modificar `backend.rs`, `paths.rs`, `http.rs`, `crates/core/src/protocol.rs`; criar `crates/native/tests/catalog_cache.rs`.

**Interfaces:** `CatalogCache::load(path: &Path) -> Result<Option<CachedCatalog>, CacheError>` e `save(&self, path: &Path) -> Result<(), CacheError>`; `CachedCatalog` contém registros, ETags e instante da última consulta. Adicionar `Event::CatalogLoaded { apps: Vec<CatalogRecord>, stale: bool, warning: Option<String> }`, posicionando o tipo compartilhado no core para não inverter dependências.

- [ ] Testar 304, offline, 403/429 com Retry-After, JSON truncado, cache corrompido e falha após primeira página. Assertar que resposta parcial não substitui snapshot completo e cache inválido não causa panic.
- [ ] Rodar `cargo test -p artcraft-launcher-native --test catalog_cache`; confirmar falha inicial.
- [ ] Implementar persistência por arquivo temporário e substituição; publicar cache antes da consulta remota em worker. Não bloquear primeiro frame. Registrar falhas sem credenciais. Separar catálogo, instalações e preferências em diretório Xiri; não migrar nem remover dados ArtCraft.
- [ ] Emitir evento completo apenas depois de terminar paginação e validação; manter apps instalados acessíveis quando removidos do catálogo remoto. Expor aviso de cache desatualizado sem repetir notificações a cada frame.
- [ ] Rodar testes de cache e backend; confirmar aprovação.
- [ ] Commit: `feat: cache catalog and preserve offline access`.

## Tarefa 4: Instalação portátil e execução explícita

**Arquivos:** criar `crates/native/src/xiri_package.rs`, `crates/native/tests/xiri_package.rs`; modificar `install.rs`, `manifest.rs`, `process.rs`, `paths.rs` e `crates/core/src/protocol.rs`.

**Interfaces:** `validate_package(root: &Path, manifest: &XiriManifest) -> Result<ValidatedPackage, PackageError>`; `install_zip(archive: &Path, destination: &Path, manifest: &XiriManifest) -> Result<Installed, PackageError>`. Exportar `LaunchTarget { kind: EntryKind, path: PathBuf }`, construído apenas após validar entrada dentro da pasta instalada. Persistir versão, checksum e manifesto do pacote instalado.

- [ ] Testar ZIP com travessia, caminho absoluto/ADS, symlink, conflito por caixa, entrada ausente, pacote excedendo limites, checksum incorreto, processo ativo e atualização interrompida. Assertar preservação de pasta de dados e instalação anterior após falha.
- [ ] Rodar `cargo test -p artcraft-launcher-native --test xiri_package`; confirmar falha inicial.
- [ ] Reutilizar verificação SHA-256 existente. Exigir `xiri.json` incluído na raiz do ZIP e coberto pelo checksum; usar manifesto remoto apenas para selecionar o asset. Se nome do asset/entrada divergir do contrato remoto, abortar e explicar. Não executar conteúdo antes da validação.
- [ ] Extrair em staging: teto de 20.000 arquivos, 4 GiB expandidos, razão de expansão máxima 200 e tamanho de download máximo 2 GiB; conservar limites existentes mais restritivos. Rejeitar links, colisões e caminhos ambíguos antes de escrever. Instalar binários em subpasta versionada e guardar dados fora dela; promoção só após validação e registro consistente, com recuperação de staging abandonado.
- [ ] Abrir EXE por `std::process::Command` sem shell. Para PowerShell, usar executável do sistema com argumentos separados `-NoProfile -File <path>`; não compor linha de comando interpolada. Associar programas locais por caminho selecionado e tipo explícito, sem executar texto livre no shell; nunca remover arquivos de uma associação local.
- [ ] Rodar testes de pacote e regressões de instalação/processos. Confirmar que repos sem checksum, entrada ou pacote compatível não são instaláveis.
- [ ] Commit: `feat: install verified Xiri packages and launch explicit entries`.

## Tarefa 5: Interface e identidade Xiri

**Arquivos:** modificar `crates/ui-egui/src/{app,model,views,details,bar,icons,dialogs,settings_dialog}.rs`, catálogos i18n, shell desktop e web; substituir recursos de marca utilizados em `assets/app-icon/`; atualizar `ATTRIBUTION.md` e `NOTICE` sem retirar créditos.

**Interfaces:** UI consome `Event::CatalogLoaded` e mantém seleção por ID de repo. Estados explícitos: carregando, sem release, pacote incompatível, requer dependências, disponível, instalado, atualização, erro e cache desatualizado. Comandos usam ID estável e entrada validada da tarefa 4.

- [ ] Criar testes egui com 0, 1 e 200 itens: busca, filtros, alternância grade/tabela, seleção preservada e remoção remota de item instalado. Assertar ausência dos apps ArtCraft como fallback e botão instalar desabilitado quando contrato estiver incompleto.
- [ ] Rodar `cargo test -p artcraft-launcher-ui-egui`; confirmar falha nos cenários novos.
- [ ] Integrar lista dinâmica preservando widgets, tipografia, cores, espaçamento e navegação existentes. Traduzir novas mensagens, incluindo pt-BR, seguindo o mecanismo i18n original. Ícones remotos só serão imagens limitadas/validadas; erro usa símbolo Xiri neutro, sem executar SVG remoto.
- [ ] Aplicar nome Xiri Launcher, ícone próprio com X, links `sugyua`, títulos, bandeja e pastas próprias. Remover botões sociais/Discord ArtCraft; manter créditos na seção Sobre. Renomear crates/binários para `xiri-launcher-*`, app para `xiri-launcher` e identidade para `io.github.sugyua.xiri-launcher`, atualizando imports e testes. Não reutilizar identificadores MSI do original.
- [ ] Rodar testes UI; compilar e abrir no Windows para conferir grade, tabela, detalhes, configurações e bandeja. Registrar captura e qualquer cenário que não pôde ser verificado.
- [ ] Commit: `feat: present dynamic catalog with Xiri identity`.

## Tarefa 6: Distribuição, documentação e publicação

**Arquivos:** modificar manifests Cargo, `apps/*/build.rs`, `packaging/windows/package.ps1`, arquivos WiX e workflows; reescrever `README.md`; criar `docs/XIRI_MANIFEST.md`, `docs/VERIFICACOES.md` e workflow de build Xiri se o original não puder ser reaproveitado sem segredos ArtCraft.

**Interfaces:** pacote final `xiri-launcher-<version>-windows-x64-portable.zip`, com `xiri-launcher.exe`, licenças e SHA256SUMS.txt separado. Workflow sem credenciais pessoais; uso apenas das permissões mínimas do token GitHub Actions. Sem assinatura fictícia.

- [ ] Verificar instalação local de Rust/MSVC. Se ausente, usar CI Windows para compilar e CI Linux para checagens compatíveis; não instalar ferramentas de sistema silenciosamente. Atualizar workflows para nomes Xiri e neutralizar jobs exclusivos de infraestrutura ArtCraft.
- [ ] Rodar `cargo xtask ci`; executar verificações de packaging exigidas pelo AGENTS.md nos arquivos alterados. Validar compilação wasm e testes de ícones/metadados Windows. Registrar resultados e falhas com comando e saída.
- [ ] Empacotar ZIP Windows x64 e checksum; verificar extração, inicialização, identidade Xiri, atualização/reabertura e cenário sem rede. Não afirmar inspeção visual se o ambiente não permitir executá-la.
- [ ] Escrever README no padrão Destinia com arquitetura, recursos realmente entregues, dependências, execução/compilação, verificações, limitações e publicação. Documentar schema 1 com exemplos EXE e PowerShell, checksum e layout do ZIP. Sensor Panel já foi publicado; não duplicar seu repo nem modificar Destinia nesta tarefa.
- [ ] Revisar diff completo, licenças e ausência de credenciais/recursos de marca em uso; confirmar alterações apenas no checkout Xiri. Atualizar AGENTS.md para a identidade e fontes autorizadas do fork preservando as regras técnicas aplicáveis.
- [ ] Commit: `build: package and document Xiri Launcher for Windows`.
- [ ] Criar repo público `sugyua/xiri-launcher` autorizado pelo pedido original, configurar origin próprio mantendo `upstream` do ArtCraft, enviar branch e publicar release somente com pacote compilado/verificado. Se checks falharem, corrigir antes da release e informar os limites que persistirem.

## Autorrevisão do plano

Cobertura: identidade/interface (5), catálogo/manifesto (1–2), releases e instalação (2/4), persistência/falhas (3/4), publicação local já realizada e distribuição Xiri (6). Os cinco riscos de revisão têm testes atribuídos. Tipos compartilhados ficam no core; nenhuma dependência core → native. Migração de nomes mecânicos é concentrada na tarefa 5 para manter comandos anteriores coerentes. Cada etapa deve terminar compilável e com evidência dos testes; os comandos passam a usar `xiri-launcher-*` depois do rename.

## Execução proposta

Recomendada: execução direta nesta conversa. As etapas compartilham tipos e interfaces; um implementador evita coordenação de migrações concorrentes. Revisão final independente pode ser feita após autorização dessa modalidade. Alternativa: execução por subagentes com revisão de cada tarefa, consumindo mais contextos. A implementação começa após a revisão deste plano e escolha do método pelo usuário.
