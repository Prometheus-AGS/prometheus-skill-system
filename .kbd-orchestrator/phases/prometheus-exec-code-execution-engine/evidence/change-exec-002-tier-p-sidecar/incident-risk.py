import json, os
input_path = os.path.join(os.environ["PROMETHEUS_INPUT_DIR"], "incident_batch.json")
with open(input_path, "r", encoding="utf-8") as handle:
    payload = json.load(handle)
scores = [item["severity"] * item["exposure"] for item in payload["incidents"]]
summary = {"incident_count": len(scores), "max_risk": max(scores), "total_risk": sum(scores)}
output_path = os.path.join(os.environ["PROMETHEUS_OUTPUT_DIR"], "risk-summary.json")
with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, sort_keys=True, separators=(",", ":"))
print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
