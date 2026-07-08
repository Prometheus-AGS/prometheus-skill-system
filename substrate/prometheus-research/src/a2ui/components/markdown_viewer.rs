use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;

pub fn render(props: Value) -> String {
    let content = props["content"].as_str().unwrap_or("");

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    format!(
        r#"<div id="markdown_viewer" hx-swap-oob="true" class="a2ui-markdown-viewer" style="font-family:sans-serif;padding:12px;border:1px solid #e2e8f0;border-radius:8px;background:#fff;line-height:1.6;">
  <style>
    .a2ui-markdown-viewer h1,.a2ui-markdown-viewer h2,.a2ui-markdown-viewer h3{{margin:0.8em 0 0.4em;line-height:1.3;}}
    .a2ui-markdown-viewer h1{{font-size:1.5em;color:#1e293b;border-bottom:2px solid #e2e8f0;padding-bottom:0.3em;}}
    .a2ui-markdown-viewer h2{{font-size:1.2em;color:#1e293b;}}
    .a2ui-markdown-viewer h3{{font-size:1.05em;color:#334155;}}
    .a2ui-markdown-viewer p{{margin:0.6em 0;font-size:14px;color:#334155;}}
    .a2ui-markdown-viewer code{{background:#f1f5f9;border-radius:3px;padding:1px 5px;font-size:12px;font-family:monospace;color:#6366f1;}}
    .a2ui-markdown-viewer pre{{background:#1e293b;border-radius:6px;padding:12px;overflow-x:auto;}}
    .a2ui-markdown-viewer pre code{{background:transparent;color:#e2e8f0;}}
    .a2ui-markdown-viewer blockquote{{border-left:3px solid #6366f1;margin:0.6em 0;padding:0.4em 0.8em;background:#f8f9ff;color:#475569;}}
    .a2ui-markdown-viewer table{{border-collapse:collapse;width:100%;font-size:13px;margin:0.6em 0;}}
    .a2ui-markdown-viewer th{{background:#f1f5f9;padding:6px 10px;text-align:left;font-weight:600;border:1px solid #e2e8f0;}}
    .a2ui-markdown-viewer td{{padding:6px 10px;border:1px solid #e2e8f0;}}
    .a2ui-markdown-viewer a{{color:#6366f1;text-decoration:none;}}
    .a2ui-markdown-viewer ul,.a2ui-markdown-viewer ol{{padding-left:1.4em;font-size:14px;color:#334155;}}
  </style>
  {html_output}
</div>"#
    )
}
