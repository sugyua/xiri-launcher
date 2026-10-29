# Xiri Launcher para Windows — 0.1.0 Beta 2

> **Versão Beta: compilação e testes automatizados aprovados; teste manual no Windows e instalação real de programas ainda pendentes.**

Launcher dos programas e jogos de [sugyua](https://github.com/sugyua), baseado no código aberto do [ArtCraft Launcher](https://github.com/storytold/craft-launcher). Preserva o desenho da interface nativa Rust/egui, com identidade Xiri e catálogo consultado no GitHub.

## Baixar e abrir

**[Baixar Xiri Launcher 0.1.0 Beta 2](https://github.com/sugyua/xiri-launcher/releases/tag/v0.1.0-beta.2)** — o pacote Windows inclui `xiri-launcher.exe` e SHA256SUMS.txt. [Beta 1 anterior](https://github.com/sugyua/xiri-launcher/releases/tag/v0.1.0-beta.1).

Extraia o ZIP completo e abra **`xiri-launcher.exe`**, na raiz da pasta extraída. Não é necessário instalar Rust. O botão **Code → Download ZIP** baixa somente o código-fonte e não inclui o executável.

## Destinia: baixar e instalar atualizações sem abrir o GitHub

O repositório do Destinia é **público**. Na seção **Destinia · atualizações**, o Xiri verifica as Releases públicas automaticamente ou pelo botão **Verificar**, sem login, token ou GitHub CLI.

Escolha a pasta existente do Destinia, feche o jogo e clique **Baixar e instalar atualização**. O Xiri solicita confirmação, baixa o ZIP Windows da Release com verificação SHA-256, prepara os arquivos, substitui os que possuem o mesmo caminho e adiciona os novos, **sem apagar arquivos extras**. Ele preserva `game`, saves e perfis; guarda backup das versões antigas dos arquivos substituídos e tenta reverter alterações em caso de falha.

A instalação escolhida precisa conter `VERSION`, `game`, `host` e `host_zoom`. Bibliotecas e dados originais continuam sendo necessários. [Passo a passo e segurança](docs/DESTINIA_UPDATES.md).

A versão web do Destinia não é alterada pelo Xiri. O ícone exibido para Destinia é **o PNG original do APK Android** (`game/res/drawable-hdpi/icon.png`), copiado sem alterações e com [procedência documentada](assets/app-icons/DESTINIA_SOURCE.md). Consulte o [README do Destinia](https://github.com/sugyua/destinia) para saber o status web realmente publicado.

## Atualizar o próprio Xiri Launcher

O Xiri também verifica se há uma Release mais recente do **próprio Xiri Launcher** no repositório público `sugyua/xiri-launcher`. A tela principal mostra **Verificar atualização do Xiri** e, quando uma versão mais nova está disponível, **Atualizar Xiri Launcher**.

Ao clicar para atualizar, o Xiri baixa o ZIP Windows portátil oficial, confere `SHA256SUMS.txt` e a versão do pacote, prepara um executável auxiliar, fecha a instância antiga, substitui apenas os arquivos do pacote com backup/reversão em caso de falha e abre a versão nova. O botão exige uma ação explícita do usuário: não força atualização silenciosa. **Não exige abrir o GitHub nem instalar o GitHub CLI.** As configurações e arquivos adicionais do usuário são mantidos.

A instalação integrada requer o Xiri **portátil Windows x64**, extraído com `xiri-launcher.exe` e `VERSION` no mesmo diretório. A opção não altera os arquivos do Destinia ou a versão web. Se algo estiver ausente, o Xiri informa o impedimento sem substituir o executável.



| Diretório | Finalidade |
| --- | --- |
| `crates/core` | Contrato de catálogo, manifestos e releases |
| `crates/native` | GitHub, cache, pacotes e execução local |
| `crates/ui-egui` | Interface compartilhada, grade, tabela e configurações |
| `apps/xiri-launcher` | Aplicativo desktop |
| `assets/xiri` | Identidade própria Xiri |

## Recursos presentes no código

- Catálogo público do perfil sugyua, com paginação e cache.
- Consulta das releases e associação de programas locais EXE ou PowerShell.
- Contrato `xiri.json` para pacotes ZIP Windows, com SHA256SUMS.txt.
- Grade, tabela, busca, temas e configurações baseados na interface original.

A presença no código não equivale a validação de execução. O progresso da compilação está em [Actions](https://github.com/sugyua/xiri-launcher/actions).

## Dependências

Windows x64 para o alvo inicial. Rust estável >= 1.95 e ferramentas C++/SDK Windows para compilar. Rust é necessário apenas para compilação; o ZIP compilado não exige instalar Rust. Os programas distribuídos têm seus próprios requisitos; consulte seus READMEs. Os pacotes Destinia não são considerados autossuficientes.

## Compilar

```powershell
cargo build --release -p xiri-launcher
```

## Desenvolvimento e distribuição

A Beta 2 usa o workflow `Xiri Windows` para compilar, testar e publicar a versão exata com checksum. A Beta 1 foi compilada na revisão `0d8d98c` (CI Windows 39912020684) e permanece disponível. Testes automatizados não comprovam acesso real ao repositório privado, instalação de programas ou funcionamento em todos os PCs.

## Publicação

Adaptação solicitada por sugyua em 22/10/2026. Créditos originais e licenças MIT/Apache-2.0 preservados. A marca Xiri é independente do ArtCraft. Consulte NOTICE e ATTRIBUTION.md.
