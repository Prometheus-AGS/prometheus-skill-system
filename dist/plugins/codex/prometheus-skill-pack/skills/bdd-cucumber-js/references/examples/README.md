# JavaScript BDD example source

These examples describe sign-in behavior through a real auth service. Choose
[HTTP steps](api-http-only/) for API behavior and
[browser steps](ui-playwright/) when rendering or interaction is part of the
specification. The step definitions call the service; replacing it with a mock
would not establish production acceptance.

The files are examples for a consuming project, not a self-contained installed
application. This directory does not ship the previously documented
`cucumber.yml`, `playwright.config.ts` or a fixture auth service. Supply the
project's dependencies, TypeScript loading, runner configuration and disposable
service before running the scenarios. Read the actual feature and step files
and preserve their business assertions.

Complete the coherent production implementation before authoring, modifying or
running tests. At the final local boundary, execute the project's real entry
point with its collaborating service/browser and retain results and applicable
captures. No timings, relative speed multiplier or previous passing result is
claimed for these sources. Hosted CI is not acceptance evidence.
