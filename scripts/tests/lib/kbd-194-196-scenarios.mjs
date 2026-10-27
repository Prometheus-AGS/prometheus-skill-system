// Scenario registry for scripts/tests/kbd-194-196.integration.mjs.
// Each scenario returns { attempted, assertions[], commands[], artifacts?, limitations?, status? }.
// `status` is set only for BLOCKED / NOT_EXERCISED; PASS and FAIL derive from assertions.
import { idrScenarios } from './kbd-194-196-idr.mjs';
import { packageScenarios } from './kbd-194-196-package.mjs';
import { sessionScenarios } from './kbd-194-196-sessions.mjs';

export const scenarioModules = [...idrScenarios, ...sessionScenarios, ...packageScenarios];
