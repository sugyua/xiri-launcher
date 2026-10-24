# Xiri Launcher para Windows — 0.1.0 Beta 2

> **Versão Beta: compilação e testes automatizados aprovados; teste manual no Windows e instalação real de programas ainda pendentes.**

Launcher dos programas e jogos de [sugyua](https://github.com/sugyua), baseado no código aberto do [ArtCraft Launcher](https://github.com/storytold/craft-launcher). Preserva o desenho da interface nativa Rust/egui, com identidade Xiri e catálogo consultado no GitHub.

## Baixar e abrir

**[Baixar Xiri Launcher 0.1.0 Beta 2](https://github.com/sugyua/xiri-launcher/releases/tag/v0.1.0-beta.2)** — o pacote Windows inclui `xiri-launcher.exe` e SHA256SUMS.txt. [Beta 1 anterior](https://github.com/sugyua/xiri-launcher/releases/tag/v0.1.0-beta.1).

Extraia o ZIP completo e abra **`xiri-launcher.exe`**, na raiz da pasta extraída. Não é necessário instalar Rust. O botão **Code → Download ZIP** baixa somente o código-fonte e não inclui o executável.

## Destinia privado: consultar e baixar sem abrir o GitHub

O Xiri agora tem uma faixa **Destinia** na tela principal. Depois de autenticar o [GitHub CLI](https://cli.github.com/) no Windows uma única vez com `gh auth login` (conta com acesso ao repositório privado), o Xiri pode consultar as versões do Destinia e **baixar o ZIP Windows e seu arquivo .sha256 diretamente**, verificando SHA-256 e integridade básica do pacote. Use **Verificar versão**, **Baixar ZIP verificado** e **Abrir pasta do ZIP**.

O download fica em `%LOCALAPPDATA%\\Xiri Launcher\\destinia-downloads\\<tag>`. **Não há instalação automática nesta Beta**: o Xiri nunca sobrescreve seus saves, a pasta `game`, BATs ou instalações existentes. A extração/instalação do ZIP verificado ainda é manual, após fechar o jogo e fazer backup. [Passo a passo e limites](docs/DESTINIA_UPDATES.md).

A versão web do Destinia não é modificada pelo Xiri. Consulte [README Destinia](https://github.com/sugyua/destinia) para a revisão web realmente publicada.

## Arquitetura

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
