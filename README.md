# Basic CRUD Ops with RustAPI

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-RustAPI-orange)

A complete, full-stack reference implementation of a generic CRUD application using **[RustAPI](https://github.com/tuntii/rustapi)**.

This project demonstrates how to build production-ready web applications with RustAPI, featuring server-side rendering, JWT authentication, SQLite database integration, and automated OpenAPI documentation.

## 🚀 Features

- **Full-Stack Implementation**: Server-side rendering with [Tera](https://keats.github.io/tera/) templates.
- **Modern Architecture**: Built on RustAPI for high performance and developer ergonomics.
- **Automated OpenAPI**: Zero-config Swagger UI documentation at `/docs`.
- **Database Integration**: Async SQLite usage with [sqlx](https://github.com/launchbadge/sqlx).
- **Authentication**: Secure user management with Argon2 hashing and JWT sessions.
- **Validation**: Declarative request validation.

## 🛠️ Technology Stack

- **Framework**: [RustAPI](https://github.com/tuntii/rustapi)
- **Database**: SQLite (via `sqlx`)
- **Templating**: Tera
- **Runtime**: Tokio
- **Hashing**: Argon2
- **Auth**: JWT

## 📦 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- `sqlx-cli` (optional, for running migrations manually)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/basic-crud-ops.git
    cd basic-crud-ops
    ```

2.  **Setup Environment:**
    The application defaults to sensible values, but you can create a `.env` file for customization:
    ```env
    DATABASE_URL=sqlite:data.db?mode=rwc
    JWT_SECRET=your-secret-key
    SERVER_HOST=127.0.0.1
    SERVER_PORT=8080
    ```

3.  **Run the Application:**
    ```bash
    cargo run
    ```
    This will compile the project, run database migrations automatically, and start the server.

4.  **Explore:**
    - **Web Interface**: [http://127.0.0.1:8080](http://127.0.0.1:8080)
    - **API Documentation**: [http://127.0.0.1:8080/docs](http://127.0.0.1:8080/docs)

## 🏗️ Project Structure

```
.
├── src/
│   ├── handlers/    # API and View handlers
│   ├── models/      # Database structs and Domain types
│   ├── db.rs        # Database connection and setup
│   ├── main.rs      # Application entry point and state
│   └── ...
├── templates/       # HTML templates (Tera)
├── static/          # Static assets (CSS, JS)
└── migrations/      # SQLx database migrations
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

## 🙏 Acknowledgements

- Built with [RustAPI](https://github.com/tuntii/rustapi).
