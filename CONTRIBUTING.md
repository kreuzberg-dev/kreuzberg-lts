# Contributing to Kreuzberg

Thank you for your interest in contributing to Kreuzberg! This guide will help you get started with development.

## Table of Contents

- [Development Setup](#development-setup)
  - [Task Installation](#task-installation)
  - [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
  - [Common Commands](#common-commands)
  - [Language-Specific Tasks](#language-specific-tasks)
  - [Build Profiles](#build-profiles)
- [Exploring Tasks](#exploring-tasks)
- [CI/CD Integration](#cicd-integration)
- [Code Quality](#code-quality)

## Development Setup

### Task Installation

This project uses [Task](https://taskfile.dev/) for task automation and orchestration. Task is a task runner that simplifies development workflows across multiple languages and platforms.

#### Install Task

Choose the installation method for your platform:

**macOS (Homebrew):**
```bash
brew install go-task
```

**Linux:**
```bash
# Using the installer script
sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b ~/.local/bin
# Or via package managers:
apt install go-task  # Debian/Ubuntu
pacman -S go-task    # Arch
```

**Windows:**
```powershell
# Using Scoop
scoop install task

# Or using Chocolatey
choco install go-task
```

**All Platforms:**
Download the latest release from [https://github.com/go-task/task/releases](https://github.com/go-task/task/releases)

For complete installation instructions, visit the [official Task documentation](https://taskfile.dev/installation/).

### Quick Start

After installing Task, set up your development environment with a single command:

```bash
# One-time setup - installs all dependencies for all languages
task setup
```

This idempotent setup command will:
- Install Rust, Python, Node.js, Ruby, Java, Go, C#, and PHP tools
- Set up language-specific dependencies
- Initialize development environments

You can safely re-run `task setup` anytime without side effects.

## Development Workflow

### Common Commands

The following top-level commands are available for common development tasks:

**Build**
```bash
# Build all language bindings
task build

# Or build everything explicitly
task build:all
```

**Test**
```bash
# Run test suites for all languages
task test

# Or be explicit
task test:all
```

**Code Quality**
```bash
# Format code for all languages
task format

# Check formatting without modifications
task format:check

# Run all linters (with auto-fix)
task lint:all

# Check linting without modifications
task lint:check

# Combined check (lint + format) without modifications
task check
```

**Dependencies**
```bash
# Update all language dependencies
task update

# Clean all build artifacts
task clean
```

### Language-Specific Tasks

Each language binding has its own namespace. Common patterns work across all languages:

**Rust:**
```bash
# Build development version
task rust:build:dev

# Build release version (optimized)
task rust:build:release

# Run tests
task rust:test

# Format code
task rust:format

# Check formatting
task rust:format:check
```

**Python:**
```bash
# Build Python extension
task python:build

# Run Python tests
task python:test

# Format Python code
task python:format

# Install dependencies
task python:install
```

**TypeScript/Node.js:**
```bash
# Build TypeScript bindings
task node:build

# Run tests
task node:test

# Format code
task node:format

# Install dependencies
task node:install
```

**Ruby:**
```bash
# Build Ruby extension
task ruby:build

# Run tests
task ruby:test

# Format code
task ruby:format
```

**Go:**
```bash
# Build Go bindings
task go:build

# Run tests
task go:test

# Format code
task go:format
```

**Java:**
```bash
# Build Java bindings
task java:build

# Run tests
task java:test

# Format code
task java:format
```

**C#:**
```bash
# Build C# bindings
task csharp:build

# Run tests
task csharp:test

# Format code
task csharp:format
```

**PHP:**
```bash
# Build PHP extension
task php:build

# Run tests
task php:test

# Format code
task php:format
```

**WebAssembly:**
```bash
# Build WASM bindings
task wasm:build

# Run tests
task wasm:test

# Clean WASM artifacts
task wasm:clean
```

### Build Profiles

Kreuzberg supports multiple build profiles optimized for different scenarios:

| Profile | Purpose | Use Case |
|---------|---------|----------|
| `dev` | Development builds with debug symbols and minimal optimizations | Local development, fast iteration |
| `release` | Optimized production builds | Performance-critical deployments, releases |
| `ci` | CI-specific builds with extra diagnostics and checks | Continuous integration pipelines |

**Using Build Profiles:**

```bash
# Development build (default, fastest to compile)
task build

# Or explicitly use dev profile
BUILD_PROFILE=dev task build

# Release build (slower to compile, faster to run)
BUILD_PROFILE=release task build

# CI build (with diagnostics)
BUILD_PROFILE=ci task build

# Language-specific with profile
BUILD_PROFILE=dev task rust:build
BUILD_PROFILE=release task python:build
```

### Workflow Commands

For orchestrated multi-language operations:

```bash
# Build all bindings in debug mode
task build:all:dev

# Build all bindings in release mode
task build:all:release

# Build all bindings in CI mode
task build:all:ci

# Run all tests in parallel (faster)
task test:all:parallel

# Run all tests in CI mode (with diagnostics)
task test:all:ci

# Generate all E2E tests from fixtures
task e2e:generate:all

# Run all E2E tests
task e2e:test:all
```

## Exploring Tasks

### List All Available Tasks

View all tasks available in the project:

```bash
# Show simple list of all public tasks
task --list

# Show all tasks including internal ones
task --list-all

# Show tasks for a specific namespace (e.g., rust)
task --list rust
```

### Get Help for a Specific Task

Display detailed information about a task:

```bash
# View task description and syntax
task -h build
task -h rust:build
task -h test:all:parallel

# View variables used by a task
task -h setup
```

### Example Output

```bash
$ task --list
task: Available tasks for this project:

* setup              Install all dependencies and initialize project. Idempotent - safe to re-run anytime.
* build             Build core libraries and language bindings
* test              Run test suites for all languages
* format            Format code for all languages (with modifications)
* format:check      Check code formatting without modifications
* check             Run all checks without modifications (lint + format checks)
* update            Update all dependencies to latest versions
* clean             Clean all build artifacts
* rust:build        Build Rust crate
* rust:test         Run Rust tests
* python:build      Build Python extension
* python:test       Run Python tests
...and more
```

## CI/CD Integration

The same Taskfile used locally is also used in continuous integration workflows. This ensures consistency between local development and CI pipelines.

### Build Profile in CI

CI pipelines use the `ci` build profile for enhanced diagnostics and consistency:

```bash
# In GitHub Actions and other CI systems:
BUILD_PROFILE=ci task build:all:ci
BUILD_PROFILE=ci task test:all:ci
```

### Running CI Checks Locally

To test your changes before pushing, run the same commands CI uses:

```bash
# Verify code quality (no modifications)
task check

# Run all tests with CI-level diagnostics
task test:all:ci

# Build all bindings in CI mode
BUILD_PROFILE=ci task build:all:ci
```

This helps catch issues before your PR reaches CI.

## Code Quality

### Formatting

The project enforces consistent code formatting across all languages:

```bash
# Auto-format all code
task format

# Check if code is formatted correctly (for CI)
task format:check
```

Supported formatters:
- **Rust**: `rustfmt`
- **Python**: `black` and `isort`
- **TypeScript**: `prettier` and `biome`
- **Ruby**: `ruby-linter`
- **Java**: `google-java-format`
- **Go**: `gofmt`
- **C#**: `dotnet format`
- **PHP**: `php-cs-fixer`
- **TOML**: `taplo`

### Linting

Lint checks ensure code quality and consistency:

```bash
# Run all linters with auto-fix
task lint:all

# Check only (no modifications)
task lint:check
```

Supported linters vary by language. Check `task -h lint:all` for details.

### Pre-commit Hooks

The project includes pre-commit hooks configuration (`.pre-commit-config.yaml`). To use it:

```bash
# Install pre-commit hooks (requires pre-commit to be installed)
pre-commit install

# Run hooks on all files
pre-commit run --all-files

# Run specific hooks
pre-commit run rust-fmt --all-files
```

## Testing

### Running Tests

```bash
# Run all language tests
task test

# Run tests for a specific language
task rust:test
task python:test
task node:test

# Run all tests in parallel (faster)
task test:all:parallel

# Run tests with CI-level diagnostics
task test:all:ci
```

### E2E Tests

End-to-end tests validate functionality across languages:

```bash
# Generate E2E test code from fixtures
task e2e:generate:all

# Run all E2E tests
task e2e:test:all

# Lint generated E2E test code
task e2e:lint:all
```

## Development Best Practices

1. **Use `task setup` first**: Ensure all dependencies are installed before starting development

2. **Run `task check` before committing**: Verify formatting and linting locally:
   ```bash
   task check
   ```

3. **Use language-specific tasks for focused work**:
   ```bash
   # Working on Rust? Focus on Rust tasks
   task rust:build:dev
   task rust:test
   task rust:format:check
   ```

4. **Leverage build profiles**: Use `dev` for rapid iteration, `release` when optimizing
   ```bash
   BUILD_PROFILE=dev task rust:build  # Fast iteration
   BUILD_PROFILE=release task rust:build  # Performance testing
   ```

5. **Test before pushing**: Run CI-level tests locally to catch issues early:
   ```bash
   BUILD_PROFILE=ci task test:all:ci
   ```

## Documentation and Help

- **Task documentation**: [https://taskfile.dev/](https://taskfile.dev/)
- **Kreuzberg documentation**: [https://kreuzberg.dev/](https://kreuzberg.dev/)
- **Get help**: Use `task -h <taskname>` to see options and descriptions
- **Contribute documentation**: Submit PRs to improve our docs

## Questions?

Have questions about contributing? Please:
- Check the [Kreuzberg documentation](https://kreuzberg.dev/)
- Open an issue on [GitHub](https://github.com/kreuzberg-dev/kreuzberg/issues)
- Join our [Discord community](https://discord.gg/pXxagNK2zN)

Thank you for contributing to Kreuzberg!
