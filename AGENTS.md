# Agent Instructions

## OpenRPC Maintenance

When creating, updating, renaming, or deleting a use case under `src/business/usecases`, update `openrpc.json` in the same change.

Use these rules when maintaining the OpenRPC document:

- The OpenRPC method name must match the JSON-RPC method exposed by `#[UseCase]`.
- If `#[UseCase]` has no explicit `method = "..."`, use the Rust use case struct name, for example `Register`.
- Request schemas must match the use case input type after JSON serialization/deserialization rules are applied.
- Response schemas must match the returned output type after JSON serialization rules are applied.
- Use externally visible JSON field names, not Rust-only field names. This project exposes camelCase JSON names through `jsonrpc-usecase`.
- Include validation constraints that are enforced during deserialization, such as string length bounds.
- Include all declared use case errors from `use_case_error!`, with their JSON-RPC error codes and messages.
- If a use case is removed, remove its method from `openrpc.json`.
- Run `cargo fmt` and `cargo test` after changing use case code or the OpenRPC document.

Do not leave a use case change without an OpenRPC update unless the use case is purely internal and not registered with `#[UseCase]`.
