# Xiri Launcher para Windows — em desenvolvimento

> **A adaptação está em desenvolvimento. Ainda não há uma versão Windows validada para uso.**

Launcher dos programas e jogos de [sugyua](https://github.com/sugyua), baseado no código aberto do [ArtCraft Launcher](https://github.com/storytold/craft-launcher). Preserva o desenho da interface nativa Rust/egui, com identidade Xiri e catálogo consultado no GitHub.

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

O workflow gera um artefato Windows somente após passar pelas verificações configuradas. Ainda não foi publicada uma release validada.

## Publicação

Adaptação solicitada por sugyua em 22/10/2026. Créditos originais e licenças MIT/Apache-2.0 preservados. A marca Xiri é independente do ArtCraft. Consulte NOTICE e ATTRIBUTION.md.
