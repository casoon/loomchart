use crate::core::overlay::ChartOverlay;
use crate::core::Viewport;
use crate::primitives::Color;
use crate::rendering::{LineStyle, RenderCommand, TextAlign, TextBaseline};

pub struct NewsEvent {
    pub time: i64,
    pub title: String,
    pub impact: NewsImpact,
}

pub enum NewsImpact {
    Low,
    Medium,
    High,
}

pub struct NewsOverlay {
    id: String,
    visible: bool,
    events: Vec<NewsEvent>,
}

impl NewsOverlay {
    pub fn new() -> Self {
        Self {
            id: "news".to_string(),
            visible: true,
            events: Vec::new(),
        }
    }

    pub fn set_events(&mut self, events: Vec<NewsEvent>) {
        self.events = events;
    }
}

impl Default for NewsOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartOverlay for NewsOverlay {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_visible(&mut self, v: bool) {
        self.visible = v;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn render_commands(&self, viewport: &Viewport) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let height = viewport.dimensions.height as f64;

        for event in &self.events {
            // Only render events within the visible time range
            if event.time < viewport.time.start || event.time > viewport.time.end {
                continue;
            }

            let x = viewport.time_to_x(event.time);

            let line_color = match event.impact {
                NewsImpact::Low => Color::rgb(100, 200, 100),
                NewsImpact::Medium => Color::rgb(255, 165, 0),
                NewsImpact::High => Color::rgb(220, 50, 50),
            };

            // Vertical dashed line across the full chart height
            commands.push(RenderCommand::IndicatorLine {
                points: vec![(x, 0.0), (x, height)],
                color: line_color,
                width: 1.0,
                style: LineStyle::Dashed {
                    dash_length: 4,
                    gap_length: 4,
                },
            });

            // Label at the top
            commands.push(RenderCommand::Text {
                text: event.title.clone(),
                x,
                y: 4.0,
                color: line_color,
                size: 11.0,
                align: TextAlign::Left,
                baseline: TextBaseline::Top,
            });
        }

        commands
    }
}
