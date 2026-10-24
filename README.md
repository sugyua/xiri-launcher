# Xiri Launcher para Windows — 0.1.0 Beta 1

> **Versão Beta: compilação e testes automatizados aprovados; teste manual no Windows e instalação real de programas ainda pendentes.**

Launcher dos programas e jogos de [sugyua](https://github.com/sugyua), baseado no código aberto do [ArtCraft Launcher](https://github.com/storytold/craft-launcher). Preserva o desenho da interface nativa Rust/egui, com identidade Xiri e catálogo consultado no GitHub.

## Baixar e abrir

**[Baixar o ZIP Windows x64](https://github.com/sugyua/xiri-launcher/releases/download/v0.1.0-beta.1/xiri-launcher-0.1.0-beta.1-windows-x64-portable.zip)** · [Release e SHA256SUMS.txt](https://github.com/sugyua/xiri-launcher/releases/tag/v0.1.0-beta.1).

Extraia o ZIP completo e abra **`xiri-launcher.exe`**, na raiz da pasta extraída. Não é necessário instalar Rust. O botão **Code → Download ZIP** baixa somente o código-fonte e não inclui o executável.

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

A release Beta utiliza o executável da revisão `0d8d98c`, aprovada na [CI Windows39912020684](https://github.com/sugyua/xiri-launcher/actions/runs/39912020684). Inclui ZIP com EXE Windows x64 e checksum. Testes automáticos não substituem a validação manual do catálogo, abertura de programas e persistência no PC.

## Publicação

Adaptação solicitada por sugyua em 22/10/2026. Créditos originais e licenças MIT/Apache-2.0 preservados. A marca Xiri é independente do ArtCraft. Consulte NOTICE e ATTRIBUTION.md.
