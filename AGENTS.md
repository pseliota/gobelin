# Working on Gobelin

## Collaboration

- Answer concisely.
- Do not modify this file or any under `docs/`.

## Development

### Documentation

- No comments or docstrings.

### Design

- Prefer high cohesion: group code around clear responsibilities.
- Prefer low coupling and avoid leaky abstractions.
- Prefer components whose behavior can be tested independently.

### Style

- Use `mod.rs` for modules with child files. Keep leaf modules in named `.rs` files.

### Commits

- Follow [Conventional Commits](https://www.conventionalcommits.org).

### Testing

#### Unit Tests

- Keep unit tests focused on a single behavior.
- Organize the test body as: arrange -> act -> assert.
- All test names should follow `{method_name}_{behavior}_{condition}`.
- Keep unit tests deterministic and isolated.
