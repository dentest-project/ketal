# Agent Instructions

## Stability and Error Handling

- Never introduce `.expect(...)`, `.unwrap(...)`, `panic!(...)`, `unreachable!(...)`, `todo!(...)`, or other intentional panic paths, including in tests.
- Return or propagate typed errors from application code instead of terminating or unwinding.
- Business errors must identify the specific rule that failed. For example, missing organization administration rights return `UserNotAllowedToAdministrateOrganizationError`.
- Use `UnexpectedError` when an invariant fails and no more specific error applies, so request handling remains stable.
- In tests, use assertions and explicit result matching instead of panic-based result extraction.
- When touching existing code that contains a panic path, replace that path with stable error handling when it is within the scope of the change.

## Use Case Structure

- The `execute` method must explicitly show the business steps in execution order. Do not delegate the entire workflow to one method that does everything.
- Extract individual steps into methods named after the business rule or operation they perform. Keep retrieval, authorization, creation, persistence, and presentation responsibilities clear.
- Use cases coordinate entity retrieval, missing-entity errors, authorization checks, persistence, and presentation. Delegate entity-specific rules to the relevant entity.
- Check the authenticated user's rights on the target resource before performing protected changes. Permissions on another resource do not authorize the operation.
- For example, `AddUserToOrganization::execute` shows these business steps before presenting the result:
  1. `retrieve_organization`
  2. `ensure_authenticated_user_is_allowed_to_administrate_organization`
  3. `retrieve_user_to_add`
  4. `add_user_to_organization`

## Entity Responsibilities

- Entities enforce their own business rules and permission checks through methods such as `ensure_can_administrate`, returning the appropriate typed error when a rule fails.
- For a found `OrganizationUser`, call `ensure_can_administrate()` and propagate its `UserNotAllowedToAdministrateOrganizationError`; do not duplicate its permission logic in the use case.
- Handle a missing lookup result directly with the appropriate error. If the authenticated user has no membership in the target organization, return `UserNotAllowedToAdministrateOrganizationError` from the use case.
- Never construct a replacement entity for a missing lookup result just to run a business-rule check. For example, do not fall back to `OrganizationUser::new(...)` when looking up the authenticated user's membership. Construct new entities only in an explicit creation step.
- Keep implementation helpers private unless callers need them. In particular, `has_permission` is private; callers use the entity's business methods.
- Creating a membership must not implicitly grant or copy permissions. Grant permissions only when the use case explicitly requires it; adding a user to an organization creates a membership without permissions.

## Presenters and Outputs

- Return presentation output types from use cases rather than exposing domain entities directly. Use a presenter to convert the entity into its output.
- Use cases that create an entity or relationship should return its detailed presentation unless another response is explicitly required. `AddUserToOrganization` returns `OrganizationUserDetailedOutput`, not `()`.
- Give an entity or relationship its own presenter, following the existing `*_presenter.rs` and `usecases/outputs` conventions.
- Compose nested outputs through the existing presenters instead of duplicating their field mapping. An organization-user detailed presenter combines the organization's detailed output, the user's detailed output, and the membership's permissions.

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

### OpenRPC Wording

- Keep documentation complete through accurate parameter schemas, result schemas, validation constraints, and declared errors. Do not repeat that information unnecessarily in prose.
- A summary is a short statement of the action, such as `Add a user to an organization.` Do not include details that are already implied by the schemas and errors.
- Use short sentences in the description for authorization requirements and relevant behavior that the schemas and errors do not express. State the required right directly, such as `The authenticated user must have the right to administrate the organization.`
- Describe permission changes in the description, not the summary. For this use case: `The user is added without permissions.`
- Do not describe the return type or enumerate its fields in the description; the result schema already documents them.
- Do not restate behavior already implied by declared errors, such as requiring an existing user or preserving existing memberships and permissions when a duplicate is rejected.
- The preferred wording for `AddUserToOrganization` is:
  - Summary: `Add a user to an organization.`
  - Description: `The authenticated user must have the right to administrate the organization. The user is added without permissions.`
