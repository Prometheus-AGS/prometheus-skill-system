use serde_json::Value;

pub fn render(props: Value) -> String {
    let claim_a = props["claim_a"].as_str().unwrap_or("Claim A not provided");
    let claim_b = props["claim_b"].as_str().unwrap_or("Claim B not provided");
    let resolution = props["resolution"].as_str().unwrap_or("Under analysis");
    let strategy = props["strategy"].as_str().unwrap_or("compare");

    let (verdict_color, verdict_label) = match strategy {
        "resolved" => ("#22c55e", "RESOLVED"),
        "unresolved" => ("#ef4444", "UNRESOLVED"),
        "partial" => ("#f59e0b", "PARTIAL"),
        _ => ("#6366f1", "COMPARING"),
    };

    format!(
        r#"<div id="contradiction_panel" hx-swap-oob="true" class="a2ui-contradiction" style="font-family:sans-serif;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden;">
  <div style="background:{verdict_color};color:white;padding:6px 12px;font-size:12px;font-weight:700;display:flex;align-items:center;gap:8px;">
    <span>{verdict_label}</span>
    <span style="opacity:0.85;font-weight:400;font-size:11px;">{resolution}</span>
  </div>
  <div style="display:grid;grid-template-columns:1fr 1fr;gap:0;">
    <div style="padding:12px;background:#fff7ed;border-right:1px solid #e2e8f0;">
      <div style="font-size:10px;font-weight:700;color:#c2410c;margin-bottom:4px;text-transform:uppercase;">Claim A</div>
      <div style="font-size:13px;color:#1e293b;line-height:1.5;">{claim_a}</div>
    </div>
    <div style="padding:12px;background:#f0f9ff;">
      <div style="font-size:10px;font-weight:700;color:#0369a1;margin-bottom:4px;text-transform:uppercase;">Claim B</div>
      <div style="font-size:13px;color:#1e293b;line-height:1.5;">{claim_b}</div>
    </div>
  </div>
</div>"#
    )
}
