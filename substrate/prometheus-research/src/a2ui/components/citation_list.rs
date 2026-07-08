use serde_json::Value;

pub fn render(props: Value) -> String {
    let citations = props["citations"].as_array().cloned().unwrap_or_default();
    let style = props["style"].as_str().unwrap_or("apa");
    let count = citations.len();

    let items = citations
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let author = c["author"].as_str().unwrap_or("Unknown");
            let title = c["title"].as_str().unwrap_or("Untitled");
            let year = c["year"].as_str().or_else(|| c["year"].as_u64().map(|_| "")).unwrap_or("n.d.");
            let url = c["url"].as_str().unwrap_or("");
            let publisher = c["publisher"].as_str().unwrap_or("");

            let citation_text = match style {
                "mla" => format!(
                    r#"{author}. "<em>{title}</em>." {publisher}, {year}."#
                ),
                "chicago" => format!(
                    r#"{author}. "{title}." {publisher} ({year})."#
                ),
                "ieee" => format!(
                    r#"[{n}] {author}, "{title}," {publisher}, {year}."#,
                    n = i + 1
                ),
                _ => {
                    // APA default
                    format!(r#"{author} ({year}). <em>{title}</em>. {publisher}."#)
                }
            };

            let link = if !url.is_empty() {
                format!(r#" <a href="{url}" target="_blank" style="color:#6366f1;font-size:11px;">[link]</a>"#)
            } else {
                String::new()
            };

            format!(
                r#"<div style="display:flex;gap:8px;padding:8px 0;border-bottom:1px solid #f1f5f9;font-size:13px;line-height:1.5;color:#334155;">
    <span style="min-width:22px;font-weight:700;color:#6366f1;">[{n}]</span>
    <span>{citation_text}{link}</span>
  </div>"#,
                n = i + 1
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let style_label = style.to_uppercase();
    format!(
        r#"<div id="citation_list" hx-swap-oob="true" class="a2ui-citation-list" style="font-family:sans-serif;padding:10px;border:1px solid #e2e8f0;border-radius:8px;background:#fff;">
  <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
    <span style="font-size:12px;font-weight:700;color:#1e293b;">References</span>
    <span style="font-size:10px;background:#e2e8f0;color:#475569;border-radius:4px;padding:2px 6px;font-weight:600;">{style_label} · {count}</span>
  </div>
  {items}
</div>"#
    )
}
