use serde_json::Value;

pub fn render(props: Value) -> String {
    let topics: Vec<String> = props["topics"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let claims: Vec<String> = props["claims"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let node_count = topics.len() + claims.len();

    let nodes_svg = topics
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let x = 80 + (i % 4) * 160;
            let y = 60 + (i / 4) * 100;
            let label = &t[..t.len().min(18)];
            format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"28\" fill=\"#6366f1\" opacity=\"0.9\"/>\
                <text x=\"{x}\" y=\"{}\" text-anchor=\"middle\" fill=\"white\" font-size=\"11\" font-family=\"sans-serif\">{label}</text>",
                y + 4
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let claims_svg = claims
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let x = 160 + (i % 3) * 180;
            let y = 200 + (i / 3) * 80;
            let label = &c[..c.len().min(22)];
            format!(
                "<rect x=\"{}\" y=\"{}\" width=\"140\" height=\"36\" rx=\"6\" fill=\"#0ea5e9\" opacity=\"0.85\"/>\
                <text x=\"{x}\" y=\"{}\" text-anchor=\"middle\" fill=\"white\" font-size=\"10\" font-family=\"sans-serif\">{label}</text>",
                x - 70,
                y - 18,
                y + 4
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<div id=\"graph_view\" hx-swap-oob=\"true\" class=\"a2ui-graph-view\" \
         style=\"border:1px solid #e2e8f0;border-radius:8px;background:#f8fafc;padding:8px;\">\
  <div style=\"font-size:11px;color:#64748b;margin-bottom:4px;font-family:sans-serif;\">Knowledge Graph \u{00B7} {node_count} nodes</div>\
  <svg viewBox=\"0 0 640 320\" width=\"100%\" style=\"overflow:hidden;border-radius:6px;background:#1e1b4b;\">\
    <defs>\
      <marker id=\"arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" orient=\"auto\">\
        <path d=\"M0,0 L0,6 L8,3 z\" fill=\"#94a3b8\"/>\
      </marker>\
    </defs>\
    {nodes_svg}\
    {claims_svg}\
    <text x=\"320\" y=\"310\" text-anchor=\"middle\" fill=\"#475569\" font-size=\"10\" font-family=\"sans-serif\">scroll/pinch to zoom</text>\
  </svg>\
</div>"
    )
}
