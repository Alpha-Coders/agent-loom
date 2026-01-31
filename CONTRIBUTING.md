# Contributing to AgentLoom

Thank you for your interest in contributing to AgentLoom!

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Install dependencies: `npm install`
4. Build: `cargo build`
5. Run tests: `cargo test`

## Development Workflow

### Branch Naming

Create a branch from `main` using this naming convention:

- `feature/short-description` - New features
- `fix/short-description` - Bug fixes
- `docs/short-description` - Documentation updates
- `refactor/short-description` - Code refactoring

### Making Changes

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes
3. Run tests: `cargo test`
4. Run linting: `cargo clippy`
5. Format code: `cargo fmt`
6. Commit with a descriptive message
7. Push to your fork
8. Open a Pull Request against `main`

### Commit Messages

Use clear, descriptive commit messages:

- `feat: add skill validation for frontmatter`
- `fix: resolve symlink creation on Windows`
- `docs: update installation instructions`
- `refactor: simplify skill sync logic`

### Pull Request Process

1. Ensure all CI checks pass
2. Update documentation if needed
3. Add tests for new functionality
4. Request review from maintainers

## Code Style

- **Rust**: Follow standard Rust conventions, use `cargo fmt`
- **TypeScript/Svelte**: Use Prettier formatting
- **Comments**: Explain *why*, not *what*

## Testing

- Run all tests: `cargo test`
- Run specific crate tests: `cargo test -p agentloom-core`
- Add tests for new features and bug fixes

## Project Structure

```
agent-loom/
├── crates/
│   ├── talent-core/     # Core library (agentloom-core)
│   └── talent-cli/      # CLI application
├── src-tauri/           # Tauri backend
├── src/                 # Svelte frontend
└── agents/              # Agent specifications
```

## Questions?

Open an issue for questions or discussion.
