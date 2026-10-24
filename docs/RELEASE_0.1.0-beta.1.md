# Xiri Launcher 0.1.0 Beta 1

> Versão Beta. Pode apresentar bugs; teste manual no Windows e instalação real de programas ainda pendentes.

## Como abrir

Baixe **xiri-launcher-0.1.0-beta.1-windows-x64-portable.zip**, extraia o conteúdo completo e abra **xiri-launcher.exe**, na raiz da pasta. **SHA256SUMS.txt** permite conferir a integridade. Não é necessário instalar Rust. Os arquivos automáticos “Source code” contêm apenas fontes.

## Primeira release

- Launcher nativo Rust/egui com identidade Xiri e catálogo público de sugyua no GitHub.
- Grade, tabela, busca, temas e preferências.
- Associação e abertura de EXE/PowerShell local; contrato xiri.json para pacotes Windows com SHA-256 e restrição de origens.
- Correções de validação da interface, imports específicos de Windows, requisitos de compilação e ícone em múltiplas resoluções.
- Publicação permanente do pacote antes disponível apenas em GitHub Actions.

## Verificação e limitações

Pacote original da revisão **0d8d98c019d607267c5dfab1c87becc508f03f7f**, CI **39912020684**, aprovada: formato, Clippy, testes, WebAssembly, prévias da interface, compilação Windows x64 e empacotamento. Testes Windows:386 aprovados,4 ignorados condicionais. SHA-256, CRC do ZIP, VERSION e cabeçalho PE x64 conferidos novamente antes da publicação. Não houve recompilação ou mudança funcional nesta publicação.

SHA-256 do ZIP: `4e0f077f16bfe445874fb73870d99b80aaec45aa96ac2e1e4233a5721331e6ca`.

Abertura/catalogação real, associação e persistência no PC ainda precisam do teste do autor. Projetos listados conservam suas dependências; listar Destinia/Sensor Panel não instala seus requisitos nem comprova compatibilidade. Sem instalador MSI e sem atualização automática do próprio launcher nesta versão. Licenças/atribuições originais preservadas.

[Instruções atuais](https://github.com/sugyua/xiri-launcher#readme) · [CI](https://github.com/sugyua/xiri-launcher/actions/runs/39912020684).
