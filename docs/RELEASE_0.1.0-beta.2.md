# Xiri Launcher 0.1.0 Beta 2 — Atualização direta do Destinia

## Novidades
- Acesso às Releases **públicas** de `sugyua/destinia` dentro do Xiri, sem GitHub CLI e sem abrir o site.
- Ícone do **Destinia original**, extraído de `game/res/drawable-hdpi/icon.png` do APK, reproduzido byte a byte no painel do jogo e no catálogo quando aplicável; sem imagens geradas por IA.
- Consulta automática/periódica e opção de notificação de nova Release do jogo.
- Escolha e memorização da pasta de instalação do Destinia no Windows.
- Botão **Baixar e instalar atualização**, com confirmação da pasta e da versão.
- Download oficial do pacote Windows, SHA-256 validado, verificações de segurança do ZIP e extração temporária antes da instalação.
- Substituição dos arquivos que aparecem no ZIP e inclusão de novos, sem apagar os arquivos exclusivos da instalação.
- Backup dos arquivos substituídos, com tentativa de reversão automática em caso de erro; pastas `game`, saves, perfis e arquivos de dados ficam protegidos.

## Atualização do próprio Xiri

- Botão **Verificar atualização do Xiri**, que consulta as Releases públicas do projeto.
- Quando uma nova versão é detectada, **Atualizar Xiri Launcher** baixa o pacote oficial com checksum SHA-256 e versão conferidos.
- Um auxiliar executado fora da instalação espera o Xiri fechar, troca os arquivos com mecanismo de backup/reversão e abre o Xiri atualizado.
- Preserva configurações e arquivos extras. Somente instalação Windows portátil com o `VERSION` correspondente é elegível; não baixa arquivos de sites não autorizados.
- A atualização depende de uma Release posterior à Beta 2; para instalar a Beta 2 pela primeira vez, o usuário ainda utiliza o ZIP publicado da Release.

## Avisos
O Xiri precisa que o Destinia esteja fechado e que a pasta já contenha uma instalação válida (`VERSION`, `game`, `host`, `host_zoom`). As dependências externas e os arquivos originais não acompanham o pacote. A atualização do Destinia no Xiri não modifica a versão web. A instalação física no PC do proprietário requer validação manual.

Este é um lançamento Beta do Xiri. O pacote inclui `xiri-launcher.exe`, documentação, licenças e checksum SHA-256. A Beta 1 anterior continua disponível.
