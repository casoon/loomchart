//! Drawing Renderer
//! Phase 4: Task 7.3

use super::{Drawing, DrawingType, Point};

/// Viewport for coordinate transformation
pub struct Viewport {
    pub width: f64,
    pub height: f64,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub min_price: f64,
    pub max_price: f64,
}

impl Viewport {
    /// Convert timestamp to screen X coordinate
    pub fn timestamp_to_x(&self, timestamp: i64) -> f64 {
        if self.max_timestamp == self.min_timestamp {
            return self.width / 2.0;
        }

        let ratio = (timestamp - self.min_timestamp) as f64
            / (self.max_timestamp - self.min_timestamp) as f64;
        ratio * self.width
    }

    /// Convert price to screen Y coordinate (inverted - lower price = higher Y)
    pub fn price_to_y(&self, price: f64) -> f64 {
        if (self.max_price - self.min_price).abs() < 0.0001 {
            return self.height / 2.0;
        }

        let ratio = (price - self.min_price) / (self.max_price - self.min_price);
        self.height - (ratio * self.height) // Invert Y axis
    }

    /// Convert screen X to timestamp
    pub fn x_to_timestamp(&self, x: f64) -> i64 {
        let ratio = x / self.width;
        self.min_timestamp + ((self.max_timestamp - self.min_timestamp) as f64 * ratio) as i64
    }

    /// Convert screen Y to price
    pub fn y_to_price(&self, y: f64) -> f64 {
        let ratio = (self.height - y) / self.height; // Invert Y axis
        self.min_price + (self.max_price - self.min_price) * ratio
    }
}

/// Simple render command for drawings
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: (u8, u8, u8, u8),
        width: f64,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        stroke: Option<(u8, u8, u8, u8)>,
        fill: Option<(u8, u8, u8, u8)>,
        stroke_width: f64,
    },
    Text {
        text: String,
        x: f64,
        y: f64,
        font_size: f64,
        color: (u8, u8, u8, u8),
    },
}

/// Drawing renderer
pub struct DrawingRenderer;

impl DrawingRenderer {
    /// Render a drawing to draw commands
    pub fn render(drawing: &Drawing, viewport: &Viewport, is_selected: bool) -> Vec<DrawCommand> {
        if !drawing.visible {
            return vec![];
        }

        match drawing.drawing_type {
            DrawingType::TrendLine => Self::render_trend_line(drawing, viewport, is_selected),
            DrawingType::HorizontalLine => {
                Self::render_horizontal_line(drawing, viewport, is_selected)
            }
            DrawingType::VerticalLine => Self::render_vertical_line(drawing, viewport, is_selected),
            DrawingType::Rectangle => Self::render_rectangle(drawing, viewport, is_selected),
            DrawingType::FibonacciRetracement => {
                Self::render_fibonacci(drawing, viewport, is_selected)
            }
        }
    }

    fn render_trend_line(
        drawing: &Drawing,
        viewport: &Viewport,
        is_selected: bool,
    ) -> Vec<DrawCommand> {
        if drawing.points.len() < 2 {
            return vec![];
        }

        let mut commands = vec![];

        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];

        let mut x1 = viewport.timestamp_to_x(p1.timestamp);
        let mut y1 = viewport.price_to_y(p1.price);
        let mut x2 = viewport.timestamp_to_x(p2.timestamp);
        let mut y2 = viewport.price_to_y(p2.price);

        // Extend line if requested
        if drawing.style.extend_left || drawing.style.extend_right {
            if (x2 - x1).abs() > 0.001 {
                let slope = (y2 - y1) / (x2 - x1);

                if drawing.style.extend_left {
                    y1 = y2 - slope * (x2 - 0.0);
                    x1 = 0.0;
                }

                if drawing.style.extend_right {
                    y2 = y1 + slope * (viewport.width - x1);
                    x2 = viewport.width;
                }
            }
        }

        let color = if is_selected {
            lighten_color(drawing.style.color)
        } else {
            drawing.style.color
        };

        commands.push(DrawCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            width: drawing.style.width,
        });

        // Draw control points if selected
        if is_selected {
            commands.extend(Self::draw_control_point(p1, viewport));
            commands.extend(Self::draw_control_point(p2, viewport));
        }

        commands
    }

    fn render_horizontal_line(
        drawing: &Drawing,
        viewport: &Viewport,
        is_selected: bool,
    ) -> Vec<DrawCommand> {
        if drawing.points.is_empty() {
            return vec![];
        }

        let mut commands = vec![];

        let price = drawing.points[0].price;
        let y = viewport.price_to_y(price);

        let color = if is_selected {
            lighten_color(drawing.style.color)
        } else {
            drawing.style.color
        };

        commands.push(DrawCommand::Line {
            x1: 0.0,
            y1: y,
            x2: viewport.width,
            y2: y,
            color,
            width: drawing.style.width,
        });

        // Draw price label
        commands.push(DrawCommand::Text {
            text: format!("{:.2}", price),
            x: viewport.width - 60.0,
            y: y - 5.0,
            font_size: 12.0,
            color: drawing.style.color,
        });

        commands
    }

    fn render_vertical_line(
        drawing: &Drawing,
        viewport: &Viewport,
        is_selected: bool,
    ) -> Vec<DrawCommand> {
        if drawing.points.is_empty() {
            return vec![];
        }

        let timestamp = drawing.points[0].timestamp;
        let x = viewport.timestamp_to_x(timestamp);

        let color = if is_selected {
            lighten_color(drawing.style.color)
        } else {
            drawing.style.color
        };

        vec![DrawCommand::Line {
            x1: x,
            y1: 0.0,
            x2: x,
            y2: viewport.height,
            color,
            width: drawing.style.width,
        }]
    }

    fn render_rectangle(
        drawing: &Drawing,
        viewport: &Viewport,
        is_selected: bool,
    ) -> Vec<DrawCommand> {
        if drawing.points.len() < 2 {
            return vec![];
        }

        let mut commands = vec![];

        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];

        let x1 = viewport.timestamp_to_x(p1.timestamp);
        let y1 = viewport.price_to_y(p1.price);
        let x2 = viewport.timestamp_to_x(p2.timestamp);
        let y2 = viewport.price_to_y(p2.price);

        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();

        let stroke = if is_selected {
            Some(lighten_color(drawing.style.color))
        } else {
            Some(drawing.style.color)
        };

        commands.push(DrawCommand::Rect {
            x,
            y,
            width,
            height,
            fill: drawing.style.fill_color,
            stroke,
            stroke_width: drawing.style.width,
        });

        if is_selected {
            commands.extend(Self::draw_control_point(p1, viewport));
            commands.extend(Self::draw_control_point(p2, viewport));
        }

        commands
    }

    fn render_fibonacci(
        drawing: &Drawing,
        viewport: &Viewport,
        is_selected: bool,
    ) -> Vec<DrawCommand> {
        if drawing.points.len() < 2 {
            return vec![];
        }

        let mut commands = vec![];

        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];

        let x1 = viewport.timestamp_to_x(p1.timestamp);
        let x2 = viewport.timestamp_to_x(p2.timestamp);
        let y1 = viewport.price_to_y(p1.price);
        let y2 = viewport.price_to_y(p2.price);

        let levels = [
            (0.0, "0%"),
            (0.236, "23.6%"),
            (0.382, "38.2%"),
            (0.5, "50%"),
            (0.618, "61.8%"),
            (0.786, "78.6%"),
            (1.0, "100%"),
        ];

        for (ratio, label) in levels {
            let y = y1 + (y2 - y1) * ratio;
            let price = p1.price + (p2.price - p1.price) * ratio;

            let (r, g, b, _) = drawing.style.color;
            let color = (r, g, b, 150); // Semi-transparent

            commands.push(DrawCommand::Line {
                x1: x1.min(x2),
                y1: y,
                x2: x1.max(x2),
                y2: y,
                color,
                width: 1.0,
            });

            commands.push(DrawCommand::Text {
                text: format!("{} ({:.2})", label, price),
                x: x1.max(x2) + 5.0,
                y: y - 5.0,
                font_size: 11.0,
                color: drawing.style.color,
            });
        }

        if is_selected {
            commands.extend(Self::draw_control_point(p1, viewport));
            commands.extend(Self::draw_control_point(p2, viewport));
        }

        commands
    }

    fn draw_control_point(point: &Point, viewport: &Viewport) -> Vec<DrawCommand> {
        let x = viewport.timestamp_to_x(point.timestamp);
        let y = viewport.price_to_y(point.price);

        vec![DrawCommand::Rect {
            x: x - 4.0,
            y: y - 4.0,
            width: 8.0,
            height: 8.0,
            fill: Some((255, 255, 255, 255)),
            stroke: Some((33, 150, 243, 255)),
            stroke_width: 2.0,
        }]
    }
}

/// Lighten a color by 30% for selection highlight
fn lighten_color((r, g, b, a): (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let lighten = |c: u8| -> u8 { ((c as f64) * 1.3).min(255.0) as u8 };
    (lighten(r), lighten(g), lighten(b), a)
}
