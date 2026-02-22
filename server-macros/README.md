# API Server Macros

![rust](https://badgen.net/badge/Rust%20Edition/2024/red?scale=1.2) ![rust](https://badgen.net/badge/Rust/1.91.1/blue?scale=1.2) ![cargo](https://badgen.net/badge/Cargo/1.91.1/gray?scale=1.2) ![spring-boot](https://badgen.net/badge/Version/0.1.0/green?scale=1.2)


> “Fear is the path to the dark side. Fear leads to anger. Anger leads to hate. Hate leads to suffering.” — Yoda

## Sobre o projeto


---
## 🚀 Tecnologias Utilizadas e principais dependências do projeto

- **Rust**

---
## 📦 Funcionalidades

- Macros para geração de código e mapeamento de objetos de inicialização do servidor;


---
## 📁 Estrutura do Projeto

```
root
├── assets/                                 # Arquivos estáticos, dados de mock e recursos de testes
│     └── tests/                            # Assets usados especificamente para testes de integração ou unitários
│
├── src/                                    # Código-fonte principal em Rust
│    └── lib.rs                             # Ponto de entrada da aplicação
│
└── tests/                                  # Testes de integração executados com cargo test
```

---
## 📦 Cargo.toml (exemplo ilustrativo)

```toml
[package]
name = "server-macros"
version = "0.1.0"
edition = "2024"
description = "A set of macros for generating code for the microservices framework."
documentation = "https://docs.rs/server-macros"
repository = "https://github.com/kenniston/server-framework/server-macros"
readme = "README.md"
license = "MIT"
keywords = ["code-gen", "macros", "initialization", "mapper", "api"]
categories = ["web-programming", "web-programming::http-server"]
authors = ["Kenniston Arraes Bonfim"]
rust-version = "1.91.1"

[dependencies]
rust-embed = { version = "8.9.0", features = ["interpolate-folder-path", "compression", "debug-embed"] }
actix-web = "4.12.0"
serde = "1.0.228"
serde_json = "1.0.145"
```


