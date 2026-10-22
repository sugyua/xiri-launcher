# Publicar um programa no Xiri Launcher

O catálogo é consultado no perfil público `sugyua`. Forks, repositórios arquivados, o repositório de perfil e `xiri-launcher` ficam fora. Sem manifesto, o projeto aparece com nome e descrição do GitHub, links para os fontes e releases e opção de associar um programa local. Não há escolha automática de um EXE encontrado em um ZIP.

## Manifesto na branch padrão

Crie `xiri.json` na raiz do repositório. O mesmo arquivo deve estar **na raiz do ZIP** da release.

```json
{
  "schema_version": 1,
  "visible": true,
  "title": "Meu Programa",
  "category": "program",
  "description": "Descrição curta do programa e dos requisitos de execução.",
  "icon": "assets/icon.png",
  "windows": {
    "asset": "meu-programa-1.0.0-windows-x64.zip",
    "entry": { "kind": "exe", "path": "bin/meu-programa.exe" }
  }
}
```

`schema_version` deve ser 1. `title`, `description`, `icon` e `windows` são opcionais. `visible` é verdadeiro por padrão; falso oculta o projeto. `category` aceita `program` ou `game`. O ícone, quando informado, precisa ser PNG no próprio repositório, com até 512 KiB e 1024 × 1024 pixels. Na falta de ícone válido, aparece a identidade Xiri.

Para PowerShell, use `"kind": "powershell"` e um caminho terminado em `.ps1`, por exemplo `MinhaVersao/Painel.ps1`. O Xiri usa o Windows PowerShell do sistema com `-NoProfile -ExecutionPolicy Bypass -File`, apenas para o processo iniciado. Não altera permanentemente a política do Windows.

Todos os caminhos do pacote usam `/`, são relativos e não podem conter `..`, dispositivos reservados do Windows, caminhos absolutos, links ou nomes conflitantes. Um manifesto inválido desabilita a instalação; os links do projeto continuam disponíveis.

## Release e checksum

Publique uma release estável do GitHub com o ZIP nomeado em `windows.asset` e `SHA256SUMS.txt`. O checksum identifica o ZIP completo:

```powershell
$file = 'meu-programa-1.0.0-windows-x64.zip'
$hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLower()
"$hash  $file" | Set-Content SHA256SUMS.txt -Encoding utf8
```

O ZIP deve conter o manifesto e o ponto de entrada, sem uma pasta externa adicional. O contrato Windows do manifesto incluído precisa coincidir com o contrato consultado no repositório. Se mudar o nome do asset ou a entrada, publique o novo pacote e atualize o manifesto juntos. Caso haja uma divergência temporária, o launcher recusa a instalação e pede atualização do catálogo.

O Xiri verifica SHA-256 antes de extrair. Downloads de executáveis ficam restritos à release correspondente em `sugyua` e aos redirecionamentos autorizados do CDN GitHub. São aceitos, inicialmente, ZIPs Windows x64 com até 2 GiB; a extração limita tamanho expandido, quantidade de arquivos e razão de compressão.

## Dados e atualizações

Os pacotes ficam em `%LOCALAPPDATA%\Xiri Launcher\apps\<repositório>\<sha256>`. Cada atualização usa uma pasta distinta e só passa a ser a instalação ativa depois de validada. Versões anteriores permanecem disponíveis no disco, evitando apagar arquivos que o programa possa ter criado junto dos seus binários.

O programa recebe `XIRI_APP_DATA` apontando para `%LOCALAPPDATA%\Xiri Launcher\apps\<repositório>\data`. Use essa pasta para preferências e progresso compartilhados entre versões. O Xiri não reescreve automaticamente os locais de dados de programas existentes; consulte o README de cada projeto.

Nesta versão, remover a associação faz o aplicativo deixar de constar como instalado, preservando arquivos e dados. A limpeza física de versões antigas é manual. Associar um EXE ou PS1 já existente também preserva seus arquivos: o launcher nunca assume propriedade dessa pasta.

## Projetos com dependências externas

Informe os requisitos na descrição e no README. Um ZIP de fontes ou um pacote que depende de componentes externos não se torna autossuficiente por aparecer no catálogo. O Destinia continua com seus downloads e requisitos originais; a integração não modifica seus repositórios nem inclui seus dados externos.
