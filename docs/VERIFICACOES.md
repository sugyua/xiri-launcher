# Verificações do Xiri Launcher

## Critério de publicação

Uma release Windows deve ser gerada a partir do commit verificado, depois de `cargo xtask ci` aprovar formato, Clippy com avisos tratados como erros, testes do workspace e compilação dos módulos WebAssembly. O workflow também compila o executável Windows x64 e gera ZIP e SHA-256.

O histórico de execução fica em [GitHub Actions](https://github.com/sugyua/xiri-launcher/actions). Os resultados finais e a identificação da release serão registrados após a conclusão das verificações. Este documento não declara aprovação de uma execução ainda pendente.

## Cenários cobertos pelo código de testes

- Contrato de pacote, versão de schema, extensão e validação de caminhos Windows.
- Filtros por proprietário, fork, arquivo, perfil e launcher; projetos com nomes semelhantes permanecem distintos.
- Paginação incompleta, resposta 304 sem cache e catálogo público vazio.
- Recusa de origens de download fora da release correta e dos hosts autorizados.
- ZIP com travessia de diretórios e colisão de nomes por caixa; extração sem execução dos arquivos.
- Escrita de estado que falha preserva a instalação anterior.
- Repositório removido do perfil mantém sua associação local e o aviso offline.
- Catálogo vazio e com 200 registros, busca e alternância entre grade e tabela.
- Regressões de interface, traduções, rede, processos e instaladores herdadas do projeto original, com as verificações de links ajustadas à identidade Xiri.

## Limites

A validação automática não equivale a testar todos os programas do catálogo, periféricos ou sensores. O Sensor Panel exige monitor secundário compatível e ferramentas de monitoramento. Os pacotes atuais do Destinia têm requisitos próprios. Esses programas não recebem novas garantias de compatibilidade por serem listados no launcher.

O alvo inicial é Windows x64. O workspace conserva a verificação WebAssembly; instaladores MSI e distribuições para outros sistemas não fazem parte da primeira entrega. A atualização do próprio launcher será feita pelo download de uma nova release, sem substituição automática do executável nesta versão.
