# Xiri Launcher 0.1.0 Beta 2 — Destinia sem abrir GitHub

- Painel próprio na interface do Windows para consultar Releases do **Destinia privado**, sem abrir navegador.
- Usa autenticação existente do **GitHub CLI** (`gh auth login` apenas na configuração inicial); Xiri não armazena tokens, senhas nem segredos.
- Download direto do ZIP **Windows** e respectivo arquivo `.sha256`, SHA-256 verificado antes da conclusão. Arquivos são guardados isoladamente em `%LOCALAPPDATA%\Xiri Launcher\destinia-downloads\`.
- Botão para abrir a pasta do ZIP verificado.
- Preserva interface/catálogo público e os demais recursos da Beta 1.

## Limitações

**O Xiri não aplica atualizações do jogo automaticamente nesta Beta.** Isso evita sobrescrever saves, arquivos `game`, dependências e configurações pessoais. A extração do ZIP e os testes de gameplay continuam sob responsabilidade do proprietário. É necessário ter `gh` instalado/autenticado no Windows com acesso ao Destinia privado.

Testes automatizados e CI não comprovam um login GitHub real no PC. Confirme o acesso privado e o download em sua máquina. A versão web do Destinia não é atualizada por este launcher. Leia `docs/DESTINIA_UPDATES.md` para detalhes.
