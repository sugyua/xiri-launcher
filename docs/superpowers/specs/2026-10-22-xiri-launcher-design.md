# Xiri Launcher — especificação

## Objetivo e aprovação do desenho

O usuário quer um launcher Windows para seus programas e jogos feitos com IA, com a interface do ArtCraft Launcher e o nome Xiri Launcher. O catálogo deve vir do perfil GitHub `sugyua`. Em 22/10/2026, aprovou adaptar o código Rust original, preservar o layout, substituir a marca e utilizar as releases disponíveis para instalar e atualizar.

Esta especificação detalha o desenho aprovado para revisão antes do plano de implementação.

## Base e identidade

Usar o checkout de `storytold/craft-launcher`, preservando Rust, egui/eframe, composição das telas, grade, tabela, detalhes, busca, configurações e estados de instalação. Não reconstruir em Electron ou webview. O alvo inicial de entrega é Windows x64.

Substituir nomes visíveis, identidade de pacote, executável, ícones, cabeçalho, bandeja, caminhos de dados e links de suporte por Xiri Launcher e `sugyua/xiri-launcher`. Criar identidade Xiri própria sem os símbolos registrados do ArtCraft. Manter licenças MIT/Apache e avisos de autoria exigidos, atribuir fontes e outros recursos reutilizados. A identidade visual estrutural será preservada; logos e conteúdo serão próprios.

## Catálogo GitHub

Consultar `https://api.github.com/users/sugyua/repos` com paginação. Considerar repositórios públicos pertencentes a `sugyua`; excluir o próprio launcher, forks, repositórios arquivados e o repositório de perfil. Não pedir token ao usuário para consultar conteúdo público. Não embutir credenciais locais.

Nome e descrição virão dos metadados do repositório. Um manifesto opcional `xiri.json` na branch padrão permitirá informar título, categoria (programa ou jogo), ícone, descrição e ponto de entrada do pacote. Validar o manifesto; nomes e caminhos não poderão escapar da pasta de instalação. Sem manifesto, apresentar o repositório e os downloads disponíveis sem presumir como executar um arquivo.

Não deduplicar projetos por semelhança de nome: `destinia` e `Destinia_PCport` são repositórios distintos e poderão aparecer. A inclusão ou exclusão explícita por manifesto permitirá ao autor organizar o catálogo depois.

## Releases, instalação e execução

Consultar a release estável mais recente de cada programa. Repositório sem release terá estado claro e link para seus fontes, sem botão de instalação habilitado. Release sem pacote Windows compatível terá link de downloads e explicação.

Preservar a verificação SHA-256 do ArtCraft. Para instalação gerenciada, exigir checksum publicado e pacote suportado. A primeira integração própria será ZIP portátil com manifesto e ponto de entrada válido. Arquivos baixados só poderão vir das releases do repositório correspondente em `sugyua` e do CDN autorizado do GitHub.

Extrair em diretório temporário, rejeitar caminhos absolutos, travessia de diretórios e links que escapem do destino; validar antes de promover a instalação. Guardar arquivos gerenciados em `%LOCALAPPDATA%\Xiri Launcher\apps`. Dados de usuário e configurações dos programas serão preservados em atualizações. Não substituir programa em execução. Não executar instaladores com elevação silenciosa.

Scripts PowerShell precisarão de tipo de entrada explícito no manifesto; programas EXE precisarão de caminho relativo explícito. Não escolher automaticamente o primeiro EXE ou script de uma release. Uma instalação local poderá ser associada pelo usuário com caminho e comando definidos.

Destinia depende de recursos e dependências externos, conforme seu README. Seus ZIPs atuais não serão tratados automaticamente como instalações autossuficientes. O catálogo informará os requisitos e dará acesso aos downloads até haver pacote compatível.

## Persistência e falhas

Separar cache de catálogo, estado de instalações e preferências em pasta própria do Xiri. Usar cache HTTP/ETag quando disponível. Na falta de conexão ou limite da API, mostrar o catálogo em cache e erro compreensível, mantendo a abertura de programas instalados. Impor limites a tamanho de JSON, imagens, downloads e extração. Não registrar tokens ou credenciais em logs.

## Publicação dos projetos locais

Na inspeção inicial de `D:\IA`, havia somente `Meus Aplicativos\Sensor Panel` com código e `Meus Aplicativos\Nexus Launcher` vazia. Sensor Panel contém dois scripts PowerShell, um EXE preexistente, um inicializador VBS, instruções e um log. Publicar fontes e EXE existente em `sugyua/sensor-panel`, excluir o log e explicar a procedência e os limites de verificação do binário. Não publicar uma pasta vazia como aplicativo.

O README seguirá a organização do Destinia: título e estado, download, arquitetura com tabela de arquivos, recursos existentes, novidades documentais, dependências, abertura, desenvolvimento/distribuição, melhorias, futuro e publicação. Não inventar versões, testes de hardware ou funcionalidades. Não modificar os repositórios Destinia existentes nesta etapa.

## Verificação e entrega

Testar paginação, filtros do catálogo, manifesto inválido, ausência de release, escolha de pacote, checksum incorreto, ZIP malicioso, atualização e persistência de dados. Verificar cenários offline e respostas de limite da API. Validar a interface Windows em grade, tabela, detalhes e configurações.

Rodar `cargo xtask ci` conforme o projeto original, além das verificações de empacotamento exigidas para arquivos alterados. Rust não foi localizado no PATH durante a inspeção inicial; verificar instalações existentes antes de definir compilação local ou em GitHub Actions. Um build aprovado em CI não substituirá a inspeção visual Windows.

Entregar repositório próprio, README em português e pacote Windows somente após compilação e verificações. O checkout original baixado para análise ainda não constitui o Xiri Launcher implementado.
