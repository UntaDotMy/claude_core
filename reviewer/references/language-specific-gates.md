# Reviewer Reference — Language-Specific Gates

## Rust
- `unsafe` blocks must be justified with a safety comment
- `unwrap()` / `expect()` usage should be minimized or justified
- Prefer `match` or `if let` over `unwrap()`
- Use `thiserror` or `anyhow` for error types

## TypeScript / JavaScript
- Strict mode (`strict: true`) required in `tsconfig.json`
- Minimize `any` usage — prefer `unknown` when type is uncertain
- Use `const` over `let` where possible
- Prefer `async`/`await` over raw promises

## Python
- Type hints required for function signatures
- Use `pathlib` over `os.path`
- Prefer `Exception` subclasses over string-based errors

## Go
- Check error returns — never ignore them with `_`
- Use `context.Context` for cancellation and timeouts
- Prefer `go test -race` for concurrent code
