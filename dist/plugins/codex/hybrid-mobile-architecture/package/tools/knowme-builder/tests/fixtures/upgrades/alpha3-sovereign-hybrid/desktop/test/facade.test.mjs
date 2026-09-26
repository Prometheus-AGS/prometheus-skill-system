import assert from "node:assert/strict";
import test from "node:test";
import { ServiceUarRuntimeFacade } from "../src/uar-facade.mjs";

test("delegates the run to UAR", async () => {
  const runtime = new ServiceUarRuntimeFacade({
    endpoint: "http://uar.invalid",
    fetchImpl: async (_url, init) => ({
      ok: true,
      json: async () => [{ type: "completed", request: JSON.parse(init.body) }],
    }),
  });
  const events = await runtime.run({ runId: "run-1", message: "hello" });
  assert.equal(events[0].type, "completed");
});
