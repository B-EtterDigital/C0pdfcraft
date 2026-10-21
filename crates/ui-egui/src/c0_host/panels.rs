//! Translate split dividers when the host hides a pane from the render projection.
use crate::docking::Panel;
use craft_ui::docking::{Action, Layout, Node};

pub(crate) fn split_path(saved: &Layout<Panel>, rendered: &Layout<Panel>, path: &[bool]) -> Option<Vec<bool>> {
    let mut target = rendered.root.as_ref()?;
    for second in path {
        let Node::Split { first, second: other, .. } = target else { return None };
        target = if *second { other } else { first };
    }
    let missing: Vec<_> = Panel::ALL.iter().copied().filter(|panel| !rendered.contains(panel)).collect();
    let mut pending = vec![(saved.root.as_ref()?, Vec::new())];
    while let Some((node, original)) = pending.pop() {
        if let Node::Split { first, second, .. } = node {
            let mut projection = Layout { root: Some(node.clone()), floating: Vec::new() };
            for panel in &missing {
                if projection.contains(panel) {
                    let _ = projection.apply(Action::Close { panel: *panel });
                }
            }
            let available = |child: &Node<Panel>| {
                Layout { root: Some(child.clone()), floating: Vec::new() }.panels().iter().any(|panel| !missing.contains(panel))
            };
            if available(first) && available(second) && projection.root.as_ref() == Some(target) {
                return Some(original);
            }
            let mut right = original.clone();
            right.push(true);
            pending.push((second, right));
            let mut left = original;
            left.push(false);
            pending.push((first, left));
        }
    }
    None
}
