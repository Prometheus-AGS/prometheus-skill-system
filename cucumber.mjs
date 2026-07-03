// Cucumber.js configuration.
//
// Loader: TypeScript step definitions are loaded via the `tsx` ESM loader,
// registered through NODE_OPTIONS="--import tsx" in the `cucumber` npm script
// (cucumber@11 is ESM; `ts-node/register` is not installed and does not apply).
//
// Paths: only the implemented feature suites run in CI. Draft features live in
// tests/features/drafts/ per the CLAUDE.md BDD rule (the sanctioned home for
// not-yet-implemented features) and MUST NOT gate CI — they have no step
// definitions yet, so including them would report undefined steps and fail the
// build. They are intentionally excluded here, not deleted.
export default {
  paths: ['tests/features/forge-validate.feature', 'tests/features/forge-enrich.feature'],
  import: ['tests/steps/**/*.ts'],
};
