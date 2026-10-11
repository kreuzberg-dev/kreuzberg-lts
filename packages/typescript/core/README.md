<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://cdn.jsdelivr.net/gh/xberg-io/assets@v1/banner/readme-banner-dark.svg">
    <img alt="Kreuzberg" width="420" src="https://cdn.jsdelivr.net/gh/xberg-io/assets@v1/banner/readme-banner-light.svg">
  </picture>
</p>

# @kreuzberg/core

Kreuzberg shared core types and utilities for use across all language bindings and packages.

## Features

- **Shared Types**: Common type definitions used across the Kreuzberg ecosystem
- **Utilities**: Reusable utility functions and helpers
- **Constants**: Common constants and configuration values

## Installation

```bash
pnpm add @kreuzberg/core
```

## Usage

### Import from root

```typescript
import { /* types */ } from "@kreuzberg/core";
```

### Import from submodules

```typescript
// Types only
import { /* types */ } from "@kreuzberg/core/types";

// Utilities only
import { /* utilities */ } from "@kreuzberg/core/utils";
```

## Development

### Build

```bash
pnpm run build
```

### Type checking

```bash
pnpm run typecheck
```

### Testing

```bash
pnpm test
```

### Linting

```bash
pnpm run lint
pnpm run lint:fix
```

## License

MIT License - see [LICENSE](../../../LICENSE) for details.
