# T ChaT 💬

Um cliente de chat para desktop leve e performático construído com Tauri, Rust e Svelte 5. Este projeto foi desenvolvido como uma exploração de tecnologias modernas para criar uma experiência de usuário fluida e eficiente em recursos.

## 🚀 Principais Funcionalidades

O TChaT não é apenas uma interface de chat, mas uma aplicação completa com funcionalidades pensadas para performance e usabilidade:

* **Interface com Abas Múltiplas:** Conecte-se a múltiplos canais de chat simultaneamente, cada um em sua própria aba.
* **Abas Reordenáveis:** Organize suas conversas arrastando e soltando as abas na ordem que preferir.
* **Persistência de Sessão:** As abas que você deixa abertas são salvas e recarregadas automaticamente quando você inicia o aplicativo novamente.
* **Conexão WebSocket Eficiente:** Utiliza uma única conexão WebSocket para todos os canais, economizando recursos de rede e memória. As inscrições nos canais são feitas dinamicamente sobre esta conexão.
* **Histórico de Mensagens Local:** As mensagens são salvas em um banco de dados SQLite local, permitindo o acesso ao histórico mesmo após reiniciar o app.
* **Scroll Inteligente:**
  * ~~A lista de mensagens é **virtualizada**, garantindo performance excepcional e baixo uso de memória, mesmo com dezenas de milhares de mensagens no histórico.~~
  * O scroll é automático, mas para de forma inteligente quando o usuário rola para cima para ler mensagens antigas.
  * Um botão de "Novas Mensagens" aparece com um contador dinâmico que diminui à medida que as mensagens são visualizadas.
* **Design Responsivo e Moderno:** A interface é limpa, utiliza um tema escuro e se adapta a diferentes tamanhos de janela.

## 🛠️ Tecnologias Utilizadas

Este projeto combina o poder do Rust no backend com a reatividade moderna do Svelte 5 no frontend.

### Backend (Rust)

* **[Tauri](https://tauri.app/)**: Framework principal para a construção do aplicativo desktop.
* **[Rust](https://www.rust-lang.org/)**: Linguagem de programação que garante performance e segurança.
* **[Tokio](https://tokio.rs/)**: Runtime assíncrono para gerenciar as tarefas de rede (WebSocket).
* **[SQLx](https://github.com/launchbadge/sqlx)**: Toolkit SQL para comunicação segura e assíncrona com o banco de dados.
* **[tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)**: Biblioteca para a conexão WebSocket.
* **[Tauri Store Plugin](https://github.com/tauri-apps/tauri-plugin-store)**: Para salvar o estado das abas em um arquivo.

#### Frontend (Svelte 5)

* **[Svelte 5 (Runes)](https://svelte.dev/)**: Framework reativo para a construção da interface.
* **[TypeScript](https://www.typescriptlang.org/)**: Para um código mais seguro e manutenível.
* ~~**[Svelte Virtual List](https://github.com/sveltejs/svelte-virtual-list)**: Componente para a virtualização da lista de mensagens.~~
* **CSS puro**: Para estilização.

## ⚙️ Como Começar

Para rodar este projeto localmente, você precisará ter o [ambiente de desenvolvimento do Tauri](https://tauri.app/v1/guides/getting-started/prerequisites) configurado (Rust e Node.js).

1. **Clone o repositório:**

    ```bash
    git clone https://github.com/Ricardtds/TChaT
    cd TChaT
    ```

2. **Instale as dependências do frontend:**

    ```bash
    npm install
    ```

3. **Execute em modo de desenvolvimento:**

    ```bash
    npm run tauri dev
    ```

## 📦 Compilando para Produção

Para gerar o executável final para a sua plataforma (Windows, macOS ou Linux):

```bash
npm run tauri build
```

O executável e o instalador serão gerados em `src-tauri/target/release/`.

## 📂 Estrutura do Projeto

``` tree
.
├── src/                      # Código do Frontend (SvelteKit)
│   ├── lib/components/
│   │   └── ChatTab.svelte    # Componente que renderiza um chat individual
│   └── routes/
│       └── +page.svelte      # Componente principal que gerencia as abas
├── src-tauri/                # Código do Backend (Rust)
│   ├── migrations/           # Arquivos .sql para o banco de dados
│   ├── src/
│   │   ├── commands.rs       # Funções expostas para o frontend
│   │   ├── lib.rs            # Ponto de entrada da aplicação Rust
│   │   └── models.rs         # Estruturas de dados
│   └── Cargo.toml            # Dependências do Rust
└── ...
```

---

## ✅ To-Do e Roadmap do Projeto

Esta lista acompanha o progresso do desenvolvimento do TChaT e os planos para futuras versões.

### Etapa 1: Fundação e Interface Principal

* [x] **Estrutura do Projeto:** Configurar o ambiente Tauri com Rust no backend e Svelte 5 (Runes) no frontend.

* [x] **Interface com Abas:** Criar uma UI que permita ter múltiplos chats abertos em abas separadas.
* [x] **Conexão de Chat:** Implementar a lógica para conectar a um novo canal de chat e exibi-lo em uma nova aba.
* [x] **Layout Responsivo:** Garantir que a interface principal e o formulário de conexão se adaptem a diferentes tamanhos de janela.

### Etapa 2: Persistência e Gerenciamento de Estado

* [x] **Conexão Única de WebSocket:** Refatorar o backend para usar uma única conexão WebSocket para todos os canais, com inscrições dinâmicas.

* [x] **Banco de Dados Local:** Configurar o `SQLx` com SQLite para salvar o histórico de mensagens localmente.
* [x] **Persistência de Sessão:** Usar o `tauri-plugin-store` para salvar as abas abertas e sua ordem, recarregando-as ao iniciar o app.
* [x] **Carregamento de Histórico:** Implementar o comando no Rust para buscar o histórico de um canal no banco e exibi-lo no frontend.

### Etapa 3: Experiência do Usuário (UX)

* [x] **Reordenação de Abas:** Implementar a funcionalidade de arrastar e soltar (`Drag and Drop`) para que as abas possam ser reordenadas pelo usuário.

* [x] **Scroll Inteligente:** Otimizar o `ChatTab` para rolar automaticamente, mas parar de forma inteligente quando o usuário interage.
* [x] **Notificação de Novas Mensagens:** Adicionar o botão "↓ Novas Mensagens" com um contador.
* [x] **Contador Dinâmico:** Fazer o contador de novas mensagens diminuir à medida que o usuário rola para baixo e as visualiza.
* [x] **Rolagem Horizontal das Abas:** Adicionar scroll horizontal na barra de abas e permitir o uso da roda do mouse.
* [x] **Popup para Adicionar Chat:** Mover o formulário de conexão para um popup para uma interface mais limpa.

* [x] **Ícone e Branding:** Criar um ícone personalizado para o aplicativo.

### Etapa 4: Performance e Otimização

* [x] **Backend Eficiente:** Centralizar o pool de conexões do banco de dados no estado do Tauri para melhor performance.

* [x] **Limite de Mensagens em Memória:** Limitar o número de mensagens renderizadas no Svelte para controlar o uso de RAM.
* [x] **Frontend Otimizado:** Implementar "throttling" no evento de scroll para garantir uma interface sempre fluida.

### 🚀 Próximos Passos (Roadmap Futuro)

* [ ] **Virtualização da Lista de Chat:** Substituir o `{#each}` padrão por uma lista virtualizada (usando `svelte-virtual-list` ou uma implementação nativa) para garantir performance excepcional com históricos de chat muito grandes.

* [ ] **Notificações Nativas:** Usar a API de notificações do Tauri para alertar o usuário sobre novas mensagens quando o aplicativo estiver em segundo plano.

* [ ] **Configurações do Usuário:** Criar uma nova aba ou janela de "Configurações" onde o usuário possa:
  * Alterar o tema (claro/escuro).
  * Definir o número máximo de mensagens a serem guardadas no histórico.
  * Limpar o cache ou o banco de dados de um canal específico.
* [ ] **Suporte a Emotes de Terceiros:** Adicionar suporte para pacotes de emotes populares como BTTV ou 7TV.
* [ ] **Busca no Histórico:** Implementar uma funcionalidade de busca (`Ctrl+F`) para encontrar mensagens no histórico de um canal.
* [ ] **Melhorias de Acessibilidade (A11y):** Revisar todo o aplicativo para garantir que seja totalmente navegável e utilizável via teclado.
* [ ] **Internacionalização (i18n):** Preparar o aplicativo para ser traduzido para outros idiomas.
* [ ] **Auto-Update:** Configurar o "updater" do Tauri para que o aplicativo possa se atualizar automaticamente para novas versões.

## 📄 Licença

Este projeto ainda está de forma privada e portanto nenhuma licença foi atribuida.
