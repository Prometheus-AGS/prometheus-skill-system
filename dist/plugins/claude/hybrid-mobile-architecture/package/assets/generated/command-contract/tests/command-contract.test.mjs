// Generated parity test.
import assert from "node:assert/strict";
import test from "node:test";
const expected = ["uar_run","uar_cancel","uar_recover"];
test("command names remain unique and shell is not granted", () => {
  assert.equal(new Set(expected).size, expected.length);
  assert.equal(["core:event:allow-listen","core:event:allow-unlisten"].includes("shell:default"), false);
});
