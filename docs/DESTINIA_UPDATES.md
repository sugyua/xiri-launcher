# Atualizar Destinia diretamente pelo Xiri Launcher — Windows

O projeto [Destinia](https://github.com/sugyua/destinia) é **público**. Esta integração não precisa do GitHub CLI, login, senhas ou tokens e funciona consultando as Releases públicas de `sugyua/destinia`.

## Como usar
1. Abra o **Xiri Launcher 0.1.0 Beta 2** no Windows x64.
2. Na seção **Destinia · atualizações**, veja a versão disponível ou clique **Verificar**. O Xiri consulta o GitHub na inicialização e nos intervalos de atualização já configurados, podendo notificar quando há uma nova Release.
3. Clique **Escolher pasta** e selecione a raiz da instalação existente do Destinia. Ela deve conter as pastas `game`, `host`, `host_zoom` e o arquivo `VERSION`.
4. **Feche o Destinia**. Clique **Baixar e instalar atualização** e confirme a pasta e a versão no diálogo.
5. O Xiri baixa o ZIP Windows correto e confere o **SHA-256 publicado no GitHub**. Antes de escrever na instalação, verifica os caminhos do ZIP e extrai todos os arquivos em um diretório temporário.
6. O Xiri substitui os arquivos com o mesmo caminho no pacote e inclui novos arquivos, **sem apagar arquivos extras** da sua instalação. Um backup dos arquivos substituídos é guardado em uma subpasta `.xiri-destinia-backup-…` na raiz do jogo.

Se houver erro durante a instalação, o Xiri tenta restaurar automaticamente os arquivos anteriores; se a restauração também falhar, mantém o backup e mostra uma mensagem de erro. **O processo não remove a pasta `game`, os saves, perfis ou arquivos `.sav`/`.dat`.**

## Proteções e limites

- Não instala o ZIP de código-fonte, somente o pacote `destinia-<versão>-windows-x64.zip` com SHA-256 correspondente.
- Rejeita ZIP com caminhos perigosos, symlinks, nomes repetidos, conteúdo fora da pasta da Release, arquivos obrigatórios ausentes ou excesso de tamanho.
- Não grava em pastas `game`, `save`, `saves`, `userdata` ou `profiles`; não remove arquivos que não existam no pacote. Backups são preservados para recuperação.
- Não substitui os programas em uso com sucesso aparente: feche ambos os hosts Destinia antes de confirmar. Testes físicos continuam necessários.
- Os executáveis Destinia dependem dos arquivos originais `game` e das bibliotecas externas/MSYS2 descritas em `INSTALL.md` no pacote do Destinia. O Xiri não instala essas dependências externas.
- A versão web do Destinia é independente; a atualização Windows não publica código novo no site.

A presença de saves apenas fora do GitHub não garante por si só sua integridade. O Xiri os protege explicitamente e mantém backups dos arquivos que efetivamente substitui.
