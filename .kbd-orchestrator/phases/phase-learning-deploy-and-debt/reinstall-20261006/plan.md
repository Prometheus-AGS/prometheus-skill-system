# Owner-authorized runtime reinstall — 2026-10-06

Scope: build and reinstall merged Surreal Memory into its existing launch agent, and refresh the merged full pack across supported installed AI tools and package-owned project caches. Preserve live data, credentials, unrelated local files and deploy-main/deploy/main. No tags/pin edits or broad testing. Single Cargo/rustc build at a time.

1. Freeze merged installation sources and preserve installed state.
2. Build and reinstall Surreal Memory with its existing service configuration.
3. Refresh full-pack tool installations and package-owned project caches.
4. Confirm live installed identities and record receipts.
