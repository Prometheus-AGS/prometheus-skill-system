import json
import os
from pathlib import Path


input_path = Path(os.environ["PROMETHEUS_INPUT_DIR"], "records")
output_path = Path(os.environ["PROMETHEUS_OUTPUT_DIR"], "summary.json")
records = json.loads(input_path.read_text(encoding="utf-8"))["records"]
summary = {
    "count": len(records),
    "ids": sorted(record["id"] for record in records),
    "maxRisk": max(record["risk"] for record in records),
    "totalRisk": sum(record["risk"] for record in records),
}
encoded = json.dumps(summary, separators=(",", ":"), sort_keys=True)
output_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(encoded)
