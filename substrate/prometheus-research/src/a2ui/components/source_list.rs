use serde_json::Value;

pub fn render(props: Value) -> String {
    let sources = props["sources"].as_array().cloned().unwrap_or_default();
    let count = sources.len();

    let cards = sources
        .iter()
        .map(|s| {
            let title = s["title"].as_str().unwrap_or("Untitled");
            let url = s["url"].as_str().unwrap_or("#");
            let domain = s["domain"].as_str().unwrap_or("unknown");
            let date = s["date"].as_str().unwrap_or("");
            let score = s["credibility_score"].as_f64().unwrap_or(0.0);
            let score_pct = (score * 100.0) as u32;
            let score_color = if score >= 0.8 {
                "#22c55e"
            } else if score >= 0.5 {
                "#f59e0b"
            } else {
                "#ef4444"
            };
            format!(
                r#"<div style="display:flex;align-items:flex-start;gap:10px;padding:10px;border:1px solid #e2e8f0;border-radius:6px;margin-bottom:6px;background:#fff;">
    <div style="flex:1;min-width:0;">
      <a href="{url}" target="_blank" style="font-size:13px;font-weight:600;color:#1e40af;text-decoration:none;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{title}</a>
      <div style="font-size:11px;color:#64748b;margin-top:2px;">{domain} {date}</div>
    </div>
    <div style="flex-shrink:0;background:{score_color};color:white;border-radius:12px;padding:2px 8px;font-size:11px;font-weight:700;">{score_pct}%</div>
  </div>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div id="source_list" hx-swap-oob="true" class="a2ui-source-list" style="font-family:sans-serif;">
  <div style="font-size:12px;color:#64748b;margin-bottom:8px;font-weight:600;">{count} Sources</div>
  {cards}
</div>"#
    )
}
