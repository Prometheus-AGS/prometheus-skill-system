use serde_json::Value;

pub fn render(props: Value) -> String {
    let media_type = props["type"].as_str().unwrap_or("unknown");
    let title = props["title"].as_str().unwrap_or("Media");
    let url = props["url"].as_str().unwrap_or("#");
    let confidence = props["confidence"].as_f64().unwrap_or(0.0);
    let conf_pct = (confidence * 100.0) as u32;

    let (icon, type_color) = match media_type {
        "video" => ("▶", "#7c3aed"),
        "audio" => ("♪", "#0891b2"),
        "image" => ("◼", "#059669"),
        "pdf" => ("📄", "#dc2626"),
        _ => ("◈", "#64748b"),
    };

    let conf_color = if confidence >= 0.8 {
        "#22c55e"
    } else if confidence >= 0.5 {
        "#f59e0b"
    } else {
        "#ef4444"
    };

    format!(
        r#"<div id="media_card" hx-swap-oob="true" class="a2ui-media-card" style="font-family:sans-serif;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden;background:#fff;max-width:320px;">
  <div style="background:{type_color};padding:20px;display:flex;align-items:center;justify-content:center;">
    <span style="font-size:36px;color:white;">{icon}</span>
  </div>
  <div style="padding:10px;">
    <div style="font-size:11px;font-weight:700;color:{type_color};text-transform:uppercase;margin-bottom:4px;">{media_type}</div>
    <a href="{url}" target="_blank" style="font-size:13px;font-weight:600;color:#1e293b;text-decoration:none;display:block;">{title}</a>
    <div style="display:flex;align-items:center;gap:6px;margin-top:8px;">
      <div style="font-size:11px;color:#64748b;">Confidence</div>
      <div style="background:{conf_color};color:white;border-radius:10px;padding:1px 7px;font-size:11px;font-weight:700;">{conf_pct}%</div>
    </div>
  </div>
</div>"#
    )
}
