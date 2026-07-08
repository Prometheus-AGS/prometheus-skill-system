use std::collections::HashMap;

use super::components::{
    citation_list, contradiction_panel, graph_view, markdown_viewer, media_card, progress_ring,
    source_list, stage_timeline,
};

type RenderFn = fn(serde_json::Value) -> String;

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    components: HashMap<&'static str, RenderFn>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        let mut components: HashMap<&'static str, RenderFn> = HashMap::new();
        components.insert("graph_view", graph_view::render);
        components.insert("source_list", source_list::render);
        components.insert("contradiction_panel", contradiction_panel::render);
        components.insert("progress_ring", progress_ring::render);
        components.insert("media_card", media_card::render);
        components.insert("stage_timeline", stage_timeline::render);
        components.insert("markdown_viewer", markdown_viewer::render);
        components.insert("citation_list", citation_list::render);
        Self { components }
    }

    pub fn render(&self, name: &str, props: serde_json::Value) -> String {
        match self.components.get(name) {
            Some(render_fn) => render_fn(props),
            None => format!(
                r#"<div id="{name}" hx-swap-oob="true" style="font-family:sans-serif;padding:12px;border:1px dashed #e2e8f0;border-radius:6px;color:#94a3b8;font-size:12px;">
  Unknown component: <code>{name}</code>. Available: graph_view, source_list, contradiction_panel, progress_ring, media_card, stage_timeline, markdown_viewer, citation_list
</div>"#
            ),
        }
    }

    pub fn component_names(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.components.keys().copied().collect();
        names.sort_unstable();
        names
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
