---
type: regex
pattern: "not passed|did not pass|does not pass|passed[^a-z\\n]{0,6}false|\\bfail(s|ed)?\\b"
flags: i
target: last_message
weight: 1
---
