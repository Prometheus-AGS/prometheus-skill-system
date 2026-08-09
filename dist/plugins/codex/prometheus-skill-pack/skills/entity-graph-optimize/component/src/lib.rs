//! entity-graph-optimize as a `prometheus:component/skill` guest.
//!
//! Ports `scripts/detect-orchestrators.sh`, which probes for orchestrator
//! marker paths and emits JSON. The shell version calls `[ -e ... ]` directly;
//! a component cannot, so the probe goes through the granted `kv-store`
//! capability instead. That is the whole point of the capability model: the
//! guest states what it needs and the host decides whether to grant it.
//!
//! REFERENCE COMPONENT, NOT A DROP-IN REPLACEMENT. The shell script stays. This
//! proves the world is implementable by a real skill; it does not retire
//! anything, and nothing executes it until UAR's host stub is fixed.

wit_bindgen::generate!({
    world: "skill",
    path: "wit",
});

// `Error` and the Guest trait arrive in scope from generate!(); only these two
// need naming explicitly.
use crate::prometheus::component::kv_store;
use crate::prometheus::component::types::ErrorKind;

struct Component;

/// Marker paths the shell version probes, in its order.
const MARKERS: [(&str, &str); 4] = [
    ("kbd", ".kbd-orchestrator/project.json"),
    ("evolver", ".evolver/"),
    ("refiner", ".refiner/"),
    ("openspec", "openspec/"),
];

fn json_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            c => vec![c],
        })
        .collect()
}

impl Guest for Component {
    fn run(input: String) -> Result<String, Error> {
        // The shell version ignores stdin; accept anything, including empty.
        // Rejecting input the original tolerated would be a behaviour change
        // smuggled in under "port".
        let _ = input;

        let mut out = String::from("{");
        for (i, (name, path)) in MARKERS.iter().enumerate() {
            // `get` returning Ok(None) means absent — distinct from an error.
            // An error here is a real capability failure and must surface, not
            // be flattened into "false"; the shell version could not tell the
            // difference and would have reported a denied probe as absence.
            let present = match kv_store::get(path) {
                Ok(v) => v.is_some(),
                Err(e) => {
                    return Err(Error {
                        kind: ErrorKind::CapabilityDenied,
                        message: format!("probing {path}: {}", e.message),
                    })
                }
            };
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "\"{}\":{{\"available\":{}}}",
                json_escape(name),
                present
            ));
        }
        out.push('}');
        Ok(out)
    }

    fn describe() -> String {
        String::from(
            "{\"skill\":\"entity-graph-optimize\",\
             \"exports\":[\"run\",\"describe\"],\
             \"capabilities\":[\"kv-store\"]}",
        )
    }
}

export!(Component);
