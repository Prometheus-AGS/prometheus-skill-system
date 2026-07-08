use serde_json::Value;

pub fn render(props: Value) -> String {
    let stage = props["stage"].as_u64().unwrap_or(0);
    let total_stages = props["total_stages"].as_u64().unwrap_or(10).max(1);
    let pct = props["pct"].as_f64().unwrap_or(0.0).clamp(0.0, 100.0);
    let stage_name = props["stage_name"].as_str().unwrap_or("Processing");

    let radius = 44.0_f64;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let dash_offset = circumference * (1.0 - pct / 100.0);
    let pct_int = pct as u32;

    let stage_color = if pct >= 100.0 {
        "#22c55e"
    } else if pct >= 50.0 {
        "#6366f1"
    } else {
        "#0ea5e9"
    };

    format!(
        "<div id=\"progress_ring\" hx-swap-oob=\"true\" class=\"a2ui-progress-ring\" \
         style=\"font-family:sans-serif;display:flex;flex-direction:column;align-items:center;gap:6px;padding:12px;\">\
  <svg width=\"110\" height=\"110\" viewBox=\"0 0 110 110\">\
    <circle cx=\"55\" cy=\"55\" r=\"{radius}\" fill=\"none\" stroke=\"#e2e8f0\" stroke-width=\"8\"/>\
    <circle cx=\"55\" cy=\"55\" r=\"{radius}\" fill=\"none\" stroke=\"{stage_color}\" stroke-width=\"8\"\
      stroke-dasharray=\"{circumference:.2}\" stroke-dashoffset=\"{dash_offset:.2}\"\
      stroke-linecap=\"round\" transform=\"rotate(-90 55 55)\"\
      style=\"transition:stroke-dashoffset 0.6s ease;\"/>\
    <text x=\"55\" y=\"50\" text-anchor=\"middle\" font-size=\"18\" font-weight=\"700\" \
      fill=\"{stage_color}\" font-family=\"sans-serif\">{pct_int}%</text>\
    <text x=\"55\" y=\"66\" text-anchor=\"middle\" font-size=\"10\" fill=\"#64748b\" \
      font-family=\"sans-serif\">stage {stage}/{total_stages}</text>\
  </svg>\
  <div style=\"font-size:12px;font-weight:600;color:#1e293b;\">{stage_name}</div>\
</div>"
    )
}
