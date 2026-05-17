use crate::core::Viewport;
use crate::rendering::RenderCommand;

/// Common trait for all engine-side overlays
pub trait ChartOverlay: Send + Sync {
    fn id(&self) -> &str;
    /// Generate render commands for this overlay given the current viewport
    fn render_commands(&self, viewport: &Viewport) -> Vec<RenderCommand>;
    /// Called when viewport changes (for cache invalidation)
    fn on_viewport_changed(&mut self, _viewport: &Viewport) {}
    /// Should this overlay be rendered this frame?
    fn is_visible(&self) -> bool {
        true
    }
    fn set_visible(&mut self, visible: bool);
}

/// Registry that holds and renders all active overlays
pub struct OverlayRegistry {
    overlays: Vec<Box<dyn ChartOverlay>>,
}

impl OverlayRegistry {
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
        }
    }

    pub fn register(&mut self, overlay: Box<dyn ChartOverlay>) {
        self.overlays.push(overlay);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.overlays.len();
        self.overlays.retain(|o| o.id() != id);
        self.overlays.len() < before
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Box<dyn ChartOverlay>> {
        self.overlays.iter_mut().find(|o| o.id() == id)
    }

    pub fn render_all(&self, viewport: &Viewport) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        for overlay in &self.overlays {
            if overlay.is_visible() {
                commands.extend(overlay.render_commands(viewport));
            }
        }
        commands
    }

    pub fn notify_viewport_changed(&mut self, viewport: &Viewport) {
        for overlay in &mut self.overlays {
            overlay.on_viewport_changed(viewport);
        }
    }

    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }
}

impl Default for OverlayRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{PriceRange, TimeRange, Viewport};
    use crate::primitives::Color;

    struct TestOverlay {
        id: String,
        visible: bool,
        command: RenderCommand,
    }

    impl TestOverlay {
        fn new(id: &str, command: RenderCommand) -> Self {
            Self {
                id: id.to_string(),
                visible: true,
                command,
            }
        }
    }

    impl ChartOverlay for TestOverlay {
        fn id(&self) -> &str {
            &self.id
        }

        fn render_commands(&self, _viewport: &Viewport) -> Vec<RenderCommand> {
            vec![self.command.clone()]
        }

        fn is_visible(&self) -> bool {
            self.visible
        }

        fn set_visible(&mut self, visible: bool) {
            self.visible = visible;
        }
    }

    fn make_viewport() -> Viewport {
        let mut vp = Viewport::new(800, 600);
        vp.time = TimeRange {
            start: 1000,
            end: 2000,
        };
        vp.price = PriceRange {
            min: 100.0,
            max: 200.0,
        };
        vp
    }

    fn test_command() -> RenderCommand {
        RenderCommand::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::rgb(255, 0, 0),
            width: 1.0,
        }
    }

    #[test]
    fn registry_starts_empty() {
        let reg = OverlayRegistry::new();
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }

    #[test]
    fn register_adds_and_len_correct() {
        let mut reg = OverlayRegistry::new();
        reg.register(Box::new(TestOverlay::new("a", test_command())));
        reg.register(Box::new(TestOverlay::new("b", test_command())));
        assert_eq!(reg.len(), 2);
        assert!(!reg.is_empty());
    }

    #[test]
    fn remove_by_id_works() {
        let mut reg = OverlayRegistry::new();
        reg.register(Box::new(TestOverlay::new("a", test_command())));
        reg.register(Box::new(TestOverlay::new("b", test_command())));

        let removed = reg.remove("a");
        assert!(removed);
        assert_eq!(reg.len(), 1);

        // Removing non-existent id returns false
        let not_found = reg.remove("a");
        assert!(!not_found);
    }

    #[test]
    fn render_all_collects_commands_from_visible_overlays() {
        let mut reg = OverlayRegistry::new();
        reg.register(Box::new(TestOverlay::new("a", test_command())));
        reg.register(Box::new(TestOverlay::new("b", test_command())));

        let vp = make_viewport();
        let cmds = reg.render_all(&vp);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn render_all_skips_invisible_overlays() {
        let mut reg = OverlayRegistry::new();
        let mut hidden = TestOverlay::new("hidden", test_command());
        hidden.set_visible(false);
        reg.register(Box::new(hidden));
        reg.register(Box::new(TestOverlay::new("visible", test_command())));

        let vp = make_viewport();
        let cmds = reg.render_all(&vp);
        assert_eq!(cmds.len(), 1);
    }
}
