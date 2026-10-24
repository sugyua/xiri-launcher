# Destinia no Xiri Launcher (Windows) — Beta 0.1.0-beta.2

O repositório `sugyua/destinia` é **privado**. O catálogo público do Xiri não mostra repositórios privados; esta integração oferece uma faixa independente de **Destinia** no topo da janela. O Xiri não pede senha, token ou login nas configurações nem copia segredos para o cache.

## Primeira configuração no Windows

1. Instale o [GitHub CLI](https://cli.github.com/) e confirme que `gh --version` funciona no Prompt/PowerShell.
2. Faça a autenticação uma única vez: `gh auth login` (conta autorizada a visualizar `sugyua/destinia`).
3. Abra `xiri-launcher.exe` Beta 0.1.0-beta.2. Ele consulta as Releases do Destinia no início e nas verificações periódicas, ou use **Verificar versão**.
4. Use **Baixar ZIP verificado**. O Xiri baixa o ZIP **Windows** da Release privada e seu arquivo `.sha256` usando o `gh`, confirma o hash SHA-256 e os nomes esperados. O progresso é processado em thread, sem travar a interface.
5. Use **Abrir pasta do ZIP**; os downloads verificados ficam em `%LOCALAPPDATA%\Xiri Launcher\destinia-downloads\<tag>\`.

**Importante:** esta fase **não substitui automaticamente os arquivos do jogo**. Feche o Destinia, faça backup e extraia o ZIP Windows na instalação apropriada com os arquivos originais `game` e suas dependências. A pasta de saves continua externa e deve ser preservada. Não extraia o ZIP de fontes para jogar.

## Limitações e segurança

- A autenticação e permissões do repositório privado são gerenciadas somente pelo GitHub CLI. Se houver erro de acesso, use `gh auth status` e confirme as permissões; não cole tokens em chats, READMEs ou issues.
- O Xiri não executa programas baixados nem atualiza em silêncio. Isso evita substituir saves, BATs pessoais, arquivos `game`/DLLs ou instalações em uso.
- O download é fixado ao repositório `sugyua/destinia`, tag formatada e nome de ZIP da versão; SHA-256 é obrigatório e conferido antes de disponibilizar. Pacotes incompletos não são anunciados como sucesso.
- Testes automáticos do parser e do workflow não substituem execução em um Windows físico com login e permissão para o repositório privado.
- A **versão web do Destinia é independente**. O Xiri baixa apenas a versão Windows; o link da versão web e seu status estão no README do Destinia.
