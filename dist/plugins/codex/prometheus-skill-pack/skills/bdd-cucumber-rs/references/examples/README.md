# Rust BDD example source

Both examples describe sign-in behavior against a real auth service. Choose
[HTTP steps](api-http-only/) for API behavior and
[browser steps](ui-thirtyfour/) when rendering or interaction belongs to the
specification. The browser crate declares its optional `ui` feature; the HTTP
crate declares the `features` integration target.

Each crate's Cargo manifest records its own dependency versions and compiler
requirement. Neither crate supplies the auth service or its registered user.
Provide a disposable service matching the features, and a compatible local
WebDriver for the browser example. A mocked service does not establish the real
production behavior.

After the complete production implementation, run from the relevant example
crate only when it is the consuming project's applicable local integration gate:

```bash
cargo test --test features -- --tags @api
```

For the browser crate, enable its declared `ui` feature and run its `features`
target with the `@ui` tag after starting the local WebDriver. Do not start a second
Cargo/rustc build while another is active on the machine. Capture UI evidence
when required by the specification; source files alone do not provide it.

These are teaching examples, not measured performance or release receipts.
Validation runs locally at the final production boundary. Hosted CI and unit-only
results are not acceptance evidence.
