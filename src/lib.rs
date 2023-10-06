//! Terminal plotting library for using in CLI applications.
//! Should work well in any unicode terminal with monospaced font.
//!
//! It is inspired by [TextPlots.jl](https://github.com/sunetos/TextPlots.jl) which is inspired by [Drawille](https://github.com/asciimoo/drawille).
//!
//! Currently it features only drawing line plots on Braille canvas, but could be extended
//! to support other canvas and chart types just like [UnicodePlots.jl](https://github.com/Evizero/UnicodePlots.jl)
//! or any other cool terminal plotting library.
//!
//! Contributions are very much welcome!
//!
//! # Usage
//! ```toml
//! [dependencies]
//! textplots = "0.8"
//! ```
//!
//! ```rust
//! use textplots::{Chart, Plot, Shape};
//!
//! println!("y = sin(x) / x");
//!
//! Chart::default()
//!     .lineplot(&Shape::Continuous(Box::new(|x| x.sin() / x)))
//!     .display();
//! ```
//!
//! It will display something like this:
//!
//! <img src="https://github.com/loony-bean/textplots-rs/blob/master/doc/demo.png?raw=true"/>
//!
//! Default viewport size is 120 x 60 points, with X values ranging from -10 to 10.
//! You can override the defaults calling `new`.
//!
//! ```rust
//! use textplots::{Chart, Plot, Shape};
//!
//! println!("y = cos(x), y = sin(x) / 2");
//!
//! Chart::new(180, 60, -5.0, 5.0)
//!     .lineplot(&Shape::Continuous(Box::new(|x| x.cos())))
//!     .lineplot(&Shape::Continuous(Box::new(|x| x.sin() / 2.0)))
//!     .display();
//! ```
//!
//! <img src="https://github.com/loony-bean/textplots-rs/blob/master/doc/demo2.png?raw=true"/>
//!
//! You could also plot series of points. See [Shape](enum.Shape.html) and [examples](https://github.com/loony-bean/textplots-rs/tree/master/examples) for more details.
//!
//! <img src="https://github.com/loony-bean/textplots-rs/blob/master/doc/demo3.png?raw=true"/>

// If std feature isn't enabled, don't load standard library
#![cfg_attr(not(feature = "std"), no_std)]

pub mod scale;
pub mod utils;

// These imports are mutual for std and no_std
extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::default::Default;
use core::f32;
use core::fmt::{Display, Formatter};
#[allow(unused_imports, clippy::single_component_path_imports)]
use drawille_nostd;
#[allow(unused_imports)]
use num_traits::real::Real;
use scale::Scale;

// These imports are for std-version only
#[cfg(feature = "std")]
use drawille::PixelColor;
#[cfg(feature = "std")]
use rgb::RGB8;

/// How the chart will do the ranging on axes
#[derive(PartialEq)]
enum ChartRangeMethod {
    /// Automatically ranges based on input data
    AutoRange,
    /// Has a fixed range between the given min & max
    FixedRange,
}

#[cfg(not(feature = "std"))]
type BrailleCanvas = drawille_nostd::Canvas;

#[cfg(not(feature = "std"))]
type Shapes<'a> = Vec<&'a Shape<'a>>;

#[cfg(feature = "std")]
type BrailleCanvas = drawille::Canvas;

#[cfg(feature = "std")]
type Shapes<'a> = Vec<(&'a Shape<'a>, Option<RGB8>)>;

/// Controls the drawing.
pub struct Chart<'a> {
    /// Canvas width in points.
    width: u32,
    /// Canvas height in points.
    height: u32,
    /// X-axis start value.
    xmin: f32,
    /// X-axis end value.
    xmax: f32,
    /// Y-axis start value (potentially calculated automatically).
    ymin: f32,
    /// Y-axis end value (potentially calculated automatically).
    ymax: f32,
    /// The type of y axis ranging we'll do
    y_ranging: ChartRangeMethod,
    /// Collection of shapes to be presented on the canvas.
    shapes: Shapes<'a>,
    /// Underlying canvas object.
    canvas: BrailleCanvas,
    /// X-axis style.
    x_style: LineStyle,
    /// Y-axis style.
    y_style: LineStyle,
    /// X-axis label format.
    x_label_format: LabelFormat,
    /// Y-axis label format.
    y_label_format: LabelFormat,
}

/// Specifies different kinds of plotted data.
pub enum Shape<'a> {
    /// Real value function.
    Continuous(Box<dyn Fn(f32) -> f32 + 'a>),
    /// Points of a scatter plot.
    Points(&'a [(f32, f32)]),
    /// Points connected with lines.
    Lines(&'a [(f32, f32)]),
    /// Points connected in step fashion.
    Steps(&'a [(f32, f32)]),
    /// Points represented with bars.
    Bars(&'a [(f32, f32)]),
}

/// Provides an interface for drawing plots.
pub trait Plot<'a> {
    /// Draws a [line chart](https://en.wikipedia.org/wiki/Line_chart) of points connected by straight line segments.
    fn lineplot(&'a mut self, shape: &'a Shape) -> &'a mut Chart;
}

/// Provides an interface for drawing colored plots.
/// Available with `std` feature.
#[cfg(feature = "std")]
pub trait ColorPlot<'a> {
    /// Draws a [line chart](https://en.wikipedia.org/wiki/Line_chart) of points connected by straight line segments using the specified color
    fn linecolorplot(&'a mut self, shape: &'a Shape, color: RGB8) -> &'a mut Chart;
}

/// Provides a builder interface for styling axis.
pub trait AxisBuilder<'a> {
    /// Specifies the style of x-axis.
    fn x_axis_style(&'a mut self, style: LineStyle) -> &'a mut Chart<'a>;

    /// Specifies the style of y-axis.
    fn y_axis_style(&'a mut self, style: LineStyle) -> &'a mut Chart<'a>;
}

pub trait LabelBuilder<'a> {
    /// Specifies the label format of x-axis.
    fn x_label_format(&'a mut self, format: LabelFormat) -> &'a mut Chart<'a>;

    /// Specifies the label format of y-axis.
    fn y_label_format(&'a mut self, format: LabelFormat) -> &'a mut Chart<'a>;
}

impl<'a> Default for Chart<'a> {
    fn default() -> Self {
        Self::new(120, 60, -10.0, 10.0)
    }
}

/// Specifies line style.
/// Default value is `LineStyle::Dotted`.
#[derive(Clone, Copy)]
pub enum LineStyle {
    /// Line is not displayed.
    None,
    /// Line is solid  (⠤⠤⠤).
    Solid,
    /// Line is dotted (⠄⠠⠀).
    Dotted,
    /// Line is dashed (⠤⠀⠤).
    Dashed,
}

/// Specifies label format.
/// Default value is `LabelFormat::Value`.
pub enum LabelFormat {
    /// Label is not displayed.
    None,
    /// Label is shown as a value.
    Value,
    /// Label is shown as a custom string.
    Custom(Box<dyn Fn(f32) -> String>),
}

impl<'a> Display for Chart<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        // get frame and replace space with U+2800 (BRAILLE PATTERN BLANK)
        let mut frame = self.canvas.frame().replace(' ', "\u{2800}");

        if let Some(idx) = frame.find('\n') {
            let xmin = self.format_x_axis_tick(self.xmin);
            let xmax = self.format_x_axis_tick(self.xmax);

            frame.insert_str(idx, &format!(" {0}", self.format_y_axis_tick(self.ymax)));

            frame.push_str(&format!(
                " {0}\n{1: <width$}{2}\n",
                self.format_y_axis_tick(self.ymin),
                xmin,
                xmax,
                width = (self.width as usize) / 2 - xmax.len()
            ));
        }
        write!(f, "{}", frame)
    }
}

impl<'a> Chart<'a> {
    /// Creates a new `Chart` object.
    ///
    /// # Panics
    ///
    /// Panics if `width` is less than 32 or `height` is less than 3.
    pub fn new(width: u32, height: u32, xmin: f32, xmax: f32) -> Self {
        if width < 32 {
            panic!("width should be at least 32");
        }

        if height < 3 {
            panic!("height should be at least 3");
        }

        Self {
            xmin,
            xmax,
            ymin: f32::INFINITY,
            ymax: f32::NEG_INFINITY,
            y_ranging: ChartRangeMethod::AutoRange,
            width,
            height,
            shapes: Vec::new(),
            canvas: BrailleCanvas::new(width, height),
            x_style: LineStyle::Dotted,
            y_style: LineStyle::Dotted,
            x_label_format: LabelFormat::Value,
            y_label_format: LabelFormat::Value,
        }
    }

    /// Creates a new `Chart` object with fixed y axis range.
    ///
    /// # Panics
    ///
    /// Panics if `width` is less than 32 or `height` is less than 3.
    pub fn new_with_y_range(
        width: u32,
        height: u32,
        xmin: f32,
        xmax: f32,
        ymin: f32,
        ymax: f32,
    ) -> Self {
        if width < 32 {
            panic!("width should be at least 32");
        }

        if height < 3 {
            panic!("height should be at least 3");
        }

        Self {
            xmin,
            xmax,
            ymin,
            ymax,
            y_ranging: ChartRangeMethod::FixedRange,
            width,
            height,
            shapes: Vec::new(),
            canvas: BrailleCanvas::new(width, height),
            x_style: LineStyle::Dotted,
            y_style: LineStyle::Dotted,
            x_label_format: LabelFormat::Value,
            y_label_format: LabelFormat::Value,
        }
    }

    /// Displays bounding rect.
    pub fn borders(&mut self) {
        let w = self.width;
        let h = self.height;

        self.vline(0, LineStyle::Dotted);
        self.vline(w, LineStyle::Dotted);
        self.hline(0, LineStyle::Dotted);
        self.hline(h, LineStyle::Dotted);
    }

    /// Draws vertical line of the specified style.
    fn vline(&mut self, i: u32, mode: LineStyle) {
        match mode {
            LineStyle::None => {}
            LineStyle::Solid => {
                if i <= self.width {
                    for j in 0..=self.height {
                        self.canvas.set(i, j);
                    }
                }
            }
            LineStyle::Dotted => {
                if i <= self.width {
                    for j in 0..=self.height {
                        if j % 3 == 0 {
                            self.canvas.set(i, j);
                        }
                    }
                }
            }
            LineStyle::Dashed => {
                if i <= self.width {
                    for j in 0..=self.height {
                        if j % 4 == 0 {
                            self.canvas.set(i, j);
                            self.canvas.set(i, j + 1);
                        }
                    }
                }
            }
        }
    }

    /// Draws horizontal line of the specified style.
    fn hline(&mut self, j: u32, mode: LineStyle) {
        match mode {
            LineStyle::None => {}
            LineStyle::Solid => {
                if j <= self.height {
                    for i in 0..=self.width {
                        self.canvas.set(i, self.height - j);
                    }
                }
            }
            LineStyle::Dotted => {
                if j <= self.height {
                    for i in 0..=self.width {
                        if i % 3 == 0 {
                            self.canvas.set(i, self.height - j);
                        }
                    }
                }
            }
            LineStyle::Dashed => {
                if j <= self.height {
                    for i in 0..=self.width {
                        if i % 4 == 0 {
                            self.canvas.set(i, self.height - j);
                            self.canvas.set(i + 1, self.height - j);
                        }
                    }
                }
            }
        }
    }

    /// Prints canvas content.
    /// Available with `std` feature.
    #[cfg(feature = "std")]
    pub fn display(&mut self) {
        self.axis();
        self.figures();

        println!("{}", self);
    }

    /// Prints canvas content with some additional visual elements (like borders).
    /// Available with `std` feature.
    #[cfg(feature = "std")]
    pub fn nice(&mut self) {
        self.borders();
        self.display();
    }

    /// Shows axis.
    pub fn axis(&mut self) {
        self.x_axis();
        self.y_axis();
    }

    /// Shows x-axis.
    pub fn x_axis(&mut self) {
        let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

        if self.ymin <= 0.0 && self.ymax >= 0.0 {
            self.hline(y_scale.linear(0.0) as u32, self.x_style);
        }
    }

    /// Shows y-axis.
    pub fn y_axis(&mut self) {
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);

        if self.xmin <= 0.0 && self.xmax >= 0.0 {
            self.vline(x_scale.linear(0.0) as u32, self.y_style);
        }
    }

    /// Performs formatting of the x axis.
    fn format_x_axis_tick(&self, value: f32) -> String {
        match &self.x_label_format {
            LabelFormat::None => "".to_owned(),
            LabelFormat::Value => format!("{:.1}", value),
            LabelFormat::Custom(f) => f(value),
        }
    }

    /// Performs formatting of the y axis.
    fn format_y_axis_tick(&self, value: f32) -> String {
        match &self.y_label_format {
            LabelFormat::None => "".to_owned(),
            LabelFormat::Value => format!("{:.1}", value),
            LabelFormat::Custom(f) => f(value),
        }
    }

    /// Translates (x, y) points into screen coordinates
    fn translate(&self, shape: &Shape, x_scale: &Scale, y_scale: &Scale) -> Vec<(u32, u32)> {
        let points: Vec<_> = match shape {
            Shape::Continuous(f) => (0..self.width)
                .filter_map(|i| {
                    let x = x_scale.inv_linear(i as f32);
                    let y = f(x);
                    if y.is_normal() {
                        let j = y_scale.linear(y).round();
                        Some((i, self.height - j as u32))
                    } else {
                        None
                    }
                })
                .collect(),
            Shape::Points(dt) | Shape::Lines(dt) | Shape::Steps(dt) | Shape::Bars(dt) => dt
                .iter()
                .filter_map(|(x, y)| {
                    let i = x_scale.linear(*x).round() as u32;
                    let j = y_scale.linear(*y).round() as u32;
                    if i <= self.width && j <= self.height {
                        Some((i, self.height - j))
                    } else {
                        None
                    }
                })
                .collect(),
        };
        points
    }

    // Shows colored figures.
    // Available with `std` feature.
    #[cfg(feature = "std")]
    pub fn figures(&mut self) {
        for (shape, color) in &self.shapes {
            let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);
            let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

            // translate (x, y) points into screen coordinates
            let points = self.translate(shape, &x_scale, &y_scale);

            // display segments
            match shape {
                Shape::Continuous(_) | Shape::Lines(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];
                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y1, x2, y2, color);
                        } else {
                            self.canvas.line(x1, y1, x2, y2);
                        }
                    }
                }
                Shape::Points(_) => {
                    for (x, y) in points {
                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.set_colored(x, y, color);
                        } else {
                            self.canvas.set(x, y);
                        }
                    }
                }
                Shape::Steps(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];

                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y2, x2, y2, color);
                            self.canvas.line_colored(x1, y1, x1, y2, color);
                        } else {
                            self.canvas.line(x1, y2, x2, y2);
                            self.canvas.line(x1, y1, x1, y2);
                        }
                    }
                }
                Shape::Bars(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];

                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y2, x2, y2, color);
                            self.canvas.line_colored(x1, y1, x1, y2, color);
                            self.canvas.line_colored(x1, self.height, x1, y1, color);
                            self.canvas.line_colored(x2, self.height, x2, y2, color);
                        } else {
                            self.canvas.line(x1, y2, x2, y2);
                            self.canvas.line(x1, y1, x1, y2);
                            self.canvas.line(x1, self.height, x1, y1);
                            self.canvas.line(x2, self.height, x2, y2);
                        }
                    }
                }
            }
        }
    }

    // Shows figures.
    #[cfg(not(feature = "std"))]
    pub fn figures(&mut self) {
        for shape in &self.shapes {
            let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);
            let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

            // translate (x, y) points into screen coordinates
            let points = self.translate(shape, &x_scale, &y_scale);

            // display segments
            match shape {
                Shape::Continuous(_) | Shape::Lines(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];
                        self.canvas.line(x1, y1, x2, y2);
                    }
                }
                Shape::Points(_) => {
                    for (x, y) in points {
                        self.canvas.set(x, y);
                    }
                }
                Shape::Steps(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];
                        self.canvas.line(x1, y2, x2, y2);
                        self.canvas.line(x1, y1, x1, y2);
                    }
                }
                Shape::Bars(_) => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];
                        self.canvas.line(x1, y2, x2, y2);
                        self.canvas.line(x1, y1, x1, y2);
                        self.canvas.line(x1, self.height, x1, y1);
                        self.canvas.line(x2, self.height, x2, y2);
                    }
                }
            }
        }
    }

    /// Returns the frame.
    pub fn frame(&self) -> String {
        self.canvas.frame()
    }

    fn rescale(&mut self, shape: &Shape) {
        // rescale ymin and ymax
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);

        let ys: Vec<_> = match shape {
            Shape::Continuous(f) => (0..self.width)
                .filter_map(|i| {
                    let x = x_scale.inv_linear(i as f32);
                    let y = f(x);
                    if y.is_normal() {
                        Some(y)
                    } else {
                        None
                    }
                })
                .collect(),
            Shape::Points(dt) | Shape::Lines(dt) | Shape::Steps(dt) | Shape::Bars(dt) => dt
                .iter()
                .filter_map(|(x, y)| {
                    if *x >= self.xmin && *x <= self.xmax {
                        Some(*y)
                    } else {
                        None
                    }
                })
                .collect(),
        };

        let ymax = *ys
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or(&0.0);
        let ymin = *ys
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or(&0.0);

        self.ymin = f32::min(self.ymin, ymin);
        self.ymax = f32::max(self.ymax, ymax);
    }
}

#[cfg(feature = "std")]
impl<'a> ColorPlot<'a> for Chart<'a> {
    fn linecolorplot(&'a mut self, shape: &'a Shape, color: RGB8) -> &'a mut Chart {
        self.shapes.push((shape, Some(color)));
        if self.y_ranging == ChartRangeMethod::AutoRange {
            self.rescale(shape);
        }
        self
    }
}

impl<'a> Plot<'a> for Chart<'a> {
    #[cfg(feature = "std")]
    fn lineplot(&'a mut self, shape: &'a Shape) -> &'a mut Chart {
        self.shapes.push((shape, None));
        if self.y_ranging == ChartRangeMethod::AutoRange {
            self.rescale(shape);
        }
        self
    }

    #[cfg(not(feature = "std"))]
    fn lineplot(&'a mut self, shape: &'a Shape) -> &'a mut Chart {
        self.shapes.push(shape);
        if self.y_ranging == ChartRangeMethod::AutoRange {
            self.rescale(shape);
        }
        self
    }
}

#[cfg(feature = "std")]
fn rgb_to_pixelcolor(rgb: &RGB8) -> PixelColor {
    PixelColor::TrueColor {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

impl<'a> AxisBuilder<'a> for Chart<'a> {
    fn x_axis_style(&'a mut self, style: LineStyle) -> &'a mut Chart {
        self.x_style = style;
        self
    }

    fn y_axis_style(&'a mut self, style: LineStyle) -> &'a mut Chart {
        self.y_style = style;
        self
    }
}

impl<'a> LabelBuilder<'a> for Chart<'a> {
    /// Specifies a formater for the x-axis label.
    fn x_label_format(&mut self, format: LabelFormat) -> &mut Self {
        self.x_label_format = format;
        self
    }

    /// Specifies a formater for the y-axis label.
    fn y_label_format(&mut self, format: LabelFormat) -> &mut Self {
        self.y_label_format = format;
        self
    }
}
