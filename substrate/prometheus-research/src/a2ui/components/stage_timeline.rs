use serde_json::Value;

pub fn render(props: Value) -> String {
    let stages = props["stages"].as_array().cloned().unwrap_or_default();

    if stages.is_empty() {
        return r#"<div id="stage_timeline" hx-swap-oob="true" class="a2ui-stage-timeline" style="font-family:sans-serif;color:#64748b;font-size:12px;padding:8px;">No stages</div>"#.to_string();
    }

    let items = stages
        .iter()
        .map(|s| {
            let name = s["name"].as_str().unwrap_or("Stage");
            let status = s["status"].as_str().unwrap_or("pending");
            let (bg, fg, marker) = match status {
                "completed" => ("#22c55e", "white", "✓"),
                "active" => ("#6366f1", "white", "●"),
                _ => ("#e2e8f0", "#64748b", "○"),
            };
            format!(
                r#"<div style="display:flex;flex-direction:column;align-items:center;gap:4px;flex:1;min-width:60px;">
    <div style="width:28px;height:28px;border-radius:50%;background:{bg};color:{fg};display:flex;align-items:center;justify-content:center;font-size:13px;font-weight:700;">{marker}</div>
    <div style="font-size:10px;text-align:center;color:#475569;line-height:1.3;">{}</div>
  </div>"#,
                &name[..name.len().min(16)]
            )
        })
        .collect::<Vec<_>>()
        .join(r#"<div style="flex:1;height:2px;background:#e2e8f0;margin-top:13px;min-width:8px;"></div>"#);

    format!(
        r#"<div id="stage_timeline" hx-swap-oob="true" class="a2ui-stage-timeline" style="font-family:sans-serif;padding:8px;">
  <div style="display:flex;align-items:flex-start;overflow-x:auto;gap:0;">
    {items}
  </div>
</div>"#
    )
}
