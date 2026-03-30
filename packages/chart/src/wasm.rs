//! WASM bridge for JavaScript chart integration.
//!
//! Exports drawings to a format consumable by the JS chart library.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::primitives::*;
use crate::context::DrawingBuffer;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Serialized drawing for JS consumption
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsDrawing {
    pub id: u64,
    pub drawing_type: String,
    pub layer: u32,
    pub plugin: String,
    pub data: JsDrawingData,
}

/// Drawing data for different types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(tag = "type")]
pub enum JsDrawingData {
    HorizontalLine {
        price: f64,
        start_time: Option<i64>,
        end_time: Option<i64>,
        color: String,
        width: f32,
        style: String,
        label: Option<String>,
    },
    VerticalLine {
        time: i64,
        color: String,
        width: f32,
        style: String,
        label: Option<String>,
    },
    Line {
        start_time: i64,
        start_price: f64,
        end_time: i64,
        end_price: f64,
        color: String,
        width: f32,
        style: String,
    },
    TrendLine {
        start_time: i64,
        start_price: f64,
        end_time: i64,
        end_price: f64,
        extend_left: bool,
        extend_right: bool,
        color: String,
        width: f32,
    },
    Label {
        text: String,
        time: i64,
        price: f64,
        color: String,
        background: Option<String>,
        font_size: f32,
        anchor: String,
        offset_x: f32,
        offset_y: f32,
    },
    PriceZone {
        top: f64,
        bottom: f64,
        start_time: Option<i64>,
        end_time: Option<i64>,
        fill_color: String,
        border_color: Option<String>,
        label: Option<String>,
    },
    Rectangle {
        time1: i64,
        price1: f64,
        time2: i64,
        price2: f64,
        stroke_color: String,
        fill_color: Option<String>,
        stroke_width: f32,
    },
    Triangle {
        p1_time: i64,
        p1_price: f64,
        p2_time: i64,
        p2_price: f64,
        p3_time: i64,
        p3_price: f64,
        stroke_color: String,
        fill_color: Option<String>,
    },
    Icon {
        time: i64,
        price: f64,
        icon_type: String,
        size: f32,
        color: String,
        tooltip: Option<String>,
    },
    Fibonacci {
        start_time: i64,
        start_price: f64,
        end_time: i64,
        end_price: f64,
        levels: Vec<(f64, String)>,
        show_labels: bool,
    },
    Channel {
        top_start_time: i64,
        top_start_price: f64,
        top_end_time: i64,
        top_end_price: f64,
        bottom_start_time: i64,
        bottom_start_price: f64,
        bottom_end_time: i64,
        bottom_end_price: f64,
        fill_color: Option<String>,
        stroke_color: String,
    },
    Arrow {
        start_time: i64,
        start_price: f64,
        end_time: i64,
        end_price: f64,
        color: String,
        has_start_head: bool,
        has_end_head: bool,
    },
    Path {
        commands: Vec<JsPathCommand>,
        stroke_color: String,
        fill_color: Option<String>,
        stroke_width: f32,
    },
}

/// Path command for JS
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsPathCommand {
    pub cmd: String,
    pub time: Option<i64>,
    pub price: Option<f64>,
    pub control1_time: Option<i64>,
    pub control1_price: Option<f64>,
    pub control2_time: Option<i64>,
    pub control2_price: Option<f64>,
}

/// Convert a DrawingBuffer to JS-friendly format
pub fn export_drawings(buffer: &DrawingBuffer) -> Vec<JsDrawing> {
    buffer.drawings.iter().map(|entry| {
        let data = convert_drawing(&entry.drawing);
        JsDrawing {
            id: entry.id.0,
            drawing_type: drawing_type_name(&entry.drawing),
            layer: entry.layer.0,
            plugin: entry.plugin_name.clone(),
            data,
        }
    }).collect()
}

fn drawing_type_name(drawing: &Drawing) -> String {
    String::from(match drawing {
        Drawing::Line(_) => "line",
        Drawing::HorizontalLine(_) => "horizontal_line",
        Drawing::VerticalLine(_) => "vertical_line",
        Drawing::TrendLine(_) => "trend_line",
        Drawing::Ray(_) => "ray",
        Drawing::ExtendedLine(_) => "extended_line",
        Drawing::Label(_) => "label",
        Drawing::PriceLabel(_) => "price_label",
        Drawing::TimeLabel(_) => "time_label",
        Drawing::Rectangle(_) => "rectangle",
        Drawing::Triangle(_) => "triangle",
        Drawing::Circle(_) => "circle",
        Drawing::Ellipse(_) => "ellipse",
        Drawing::Arc(_) => "arc",
        Drawing::Arrow(_) => "arrow",
        Drawing::Bracket(_) => "bracket",
        Drawing::PriceZone(_) => "price_zone",
        Drawing::TimeZone(_) => "time_zone",
        Drawing::FibonacciRetracement(_) => "fibonacci",
        Drawing::FibonacciExtension(_) => "fibonacci_extension",
        Drawing::FibonacciFan(_) => "fibonacci_fan",
        Drawing::Channel(_) => "channel",
        Drawing::PitchFork(_) => "pitchfork",
        Drawing::Icon(_) => "icon",
        Drawing::Text(_) => "text",
        Drawing::Path(_) => "path",
    })
}

fn line_style_to_string(style: &LineStyle) -> String {
    String::from(match style {
        LineStyle::Solid => "solid",
        LineStyle::Dashed => "dashed",
        LineStyle::Dotted => "dotted",
        LineStyle::DashDot => "dashdot",
    })
}

fn anchor_to_string(anchor: &Anchor) -> String {
    String::from(match anchor {
        Anchor::TopLeft => "top-left",
        Anchor::TopCenter => "top-center",
        Anchor::TopRight => "top-right",
        Anchor::MiddleLeft => "middle-left",
        Anchor::MiddleCenter => "middle-center",
        Anchor::MiddleRight => "middle-right",
        Anchor::BottomLeft => "bottom-left",
        Anchor::BottomCenter => "bottom-center",
        Anchor::BottomRight => "bottom-right",
    })
}

fn icon_type_to_string(icon: &IconType) -> String {
    String::from(match icon {
        IconType::ArrowUp => "arrow_up",
        IconType::ArrowDown => "arrow_down",
        IconType::ArrowLeft => "arrow_left",
        IconType::ArrowRight => "arrow_right",
        IconType::CircleFilled => "circle_filled",
        IconType::CircleOutline => "circle_outline",
        IconType::SquareFilled => "square_filled",
        IconType::SquareOutline => "square_outline",
        IconType::TriangleUp => "triangle_up",
        IconType::TriangleDown => "triangle_down",
        IconType::Diamond => "diamond",
        IconType::Star => "star",
        IconType::Flag => "flag",
        IconType::Cross => "cross",
        IconType::Check => "check",
        IconType::Warning => "warning",
        IconType::Info => "info",
        IconType::DoubleTop => "double_top",
        IconType::DoubleBottom => "double_bottom",
        IconType::HeadShoulders => "head_shoulders",
    })
}

fn convert_drawing(drawing: &Drawing) -> JsDrawingData {
    match drawing {
        Drawing::HorizontalLine(hl) => JsDrawingData::HorizontalLine {
            price: hl.price,
            start_time: hl.start_time,
            end_time: hl.end_time,
            color: hl.props.color.to_rgba_string(),
            width: hl.props.width,
            style: line_style_to_string(&hl.props.style),
            label: hl.label.clone(),
        },
        Drawing::VerticalLine(vl) => JsDrawingData::VerticalLine {
            time: vl.time,
            color: vl.props.color.to_rgba_string(),
            width: vl.props.width,
            style: line_style_to_string(&vl.props.style),
            label: vl.label.clone(),
        },
        Drawing::Line(l) => JsDrawingData::Line {
            start_time: l.start.time,
            start_price: l.start.price,
            end_time: l.end.time,
            end_price: l.end.price,
            color: l.props.color.to_rgba_string(),
            width: l.props.width,
            style: line_style_to_string(&l.props.style),
        },
        Drawing::TrendLine(tl) => JsDrawingData::TrendLine {
            start_time: tl.start.time,
            start_price: tl.start.price,
            end_time: tl.end.time,
            end_price: tl.end.price,
            extend_left: tl.extend_left,
            extend_right: tl.extend_right,
            color: tl.props.color.to_rgba_string(),
            width: tl.props.width,
        },
        Drawing::Label(l) => JsDrawingData::Label {
            text: l.text.clone(),
            time: l.position.time,
            price: l.position.price,
            color: l.style.color.to_rgba_string(),
            background: l.style.background.map(|c| c.to_rgba_string()),
            font_size: l.style.font_size,
            anchor: anchor_to_string(&l.style.anchor),
            offset_x: l.offset_x,
            offset_y: l.offset_y,
        },
        Drawing::PriceZone(pz) => JsDrawingData::PriceZone {
            top: pz.top,
            bottom: pz.bottom,
            start_time: pz.start_time,
            end_time: pz.end_time,
            fill_color: pz.fill_color.to_rgba_string(),
            border_color: pz.border_color.map(|c| c.to_rgba_string()),
            label: pz.label.clone(),
        },
        Drawing::Rectangle(r) => JsDrawingData::Rectangle {
            time1: r.top_left.time,
            price1: r.top_left.price,
            time2: r.bottom_right.time,
            price2: r.bottom_right.price,
            stroke_color: r.props.stroke_color.to_rgba_string(),
            fill_color: r.props.fill_color.map(|c| c.to_rgba_string()),
            stroke_width: r.props.stroke_width,
        },
        Drawing::Triangle(t) => JsDrawingData::Triangle {
            p1_time: t.p1.time,
            p1_price: t.p1.price,
            p2_time: t.p2.time,
            p2_price: t.p2.price,
            p3_time: t.p3.time,
            p3_price: t.p3.price,
            stroke_color: t.props.stroke_color.to_rgba_string(),
            fill_color: t.props.fill_color.map(|c| c.to_rgba_string()),
        },
        Drawing::Icon(i) => JsDrawingData::Icon {
            time: i.position.time,
            price: i.position.price,
            icon_type: icon_type_to_string(&i.icon_type),
            size: i.size,
            color: i.color.to_rgba_string(),
            tooltip: i.tooltip.clone(),
        },
        Drawing::FibonacciRetracement(f) => JsDrawingData::Fibonacci {
            start_time: f.start.time,
            start_price: f.start.price,
            end_time: f.end.time,
            end_price: f.end.price,
            levels: f.levels.iter().map(|(l, c)| (*l, c.to_rgba_string())).collect(),
            show_labels: f.show_labels,
        },
        Drawing::Channel(ch) => JsDrawingData::Channel {
            top_start_time: ch.top_start.time,
            top_start_price: ch.top_start.price,
            top_end_time: ch.top_end.time,
            top_end_price: ch.top_end.price,
            bottom_start_time: ch.bottom_start.time,
            bottom_start_price: ch.bottom_start.price,
            bottom_end_time: ch.bottom_end.time,
            bottom_end_price: ch.bottom_end.price,
            fill_color: ch.fill_color.map(|c| c.to_rgba_string()),
            stroke_color: ch.props.color.to_rgba_string(),
        },
        Drawing::Arrow(a) => JsDrawingData::Arrow {
            start_time: a.start.time,
            start_price: a.start.price,
            end_time: a.end.time,
            end_price: a.end.price,
            color: a.props.color.to_rgba_string(),
            has_start_head: a.start_head != ArrowHead::None,
            has_end_head: a.end_head != ArrowHead::None,
        },
        // Default fallback for other types
        _ => JsDrawingData::Label {
            text: String::from("Unsupported"),
            time: 0,
            price: 0.0,
            color: String::from("white"),
            background: None,
            font_size: 12.0,
            anchor: String::from("middle-center"),
            offset_x: 0.0,
            offset_y: 0.0,
        },
    }
}

/// WASM-exported functions
#[cfg(feature = "wasm")]
mod wasm_exports {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn export_drawings_json(buffer_ptr: *const DrawingBuffer) -> String {
        let buffer = unsafe { &*buffer_ptr };
        let drawings = export_drawings(buffer);
        serde_json::to_string(&drawings).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        let color = Color::rgba(255, 0, 0, 0.5);
        assert_eq!(color.to_rgba_string(), "rgba(255,0,0,0.5)");
    }
}
