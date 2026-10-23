//! The "Charts" tab: the time series chart, with sample data like Postino's load test results.
//!
//! The legend toggles and the selection live in the gallery, to show how an app hides a series
//! and keeps the range the chart reports. The hover is the chart's own state.

use std::sync::Arc;

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;
use telaio_ui::components::{
    AxisSide, ChartAxis, ChartSelection, ChartSeries, ReferenceLine, SeriesStyle, TimeSeriesChart,
};
use telaio_ui::metrics::RADIUS_LG;
use telaio_ui::palette::hex;

use crate::gallery::{Gallery, section};

/// How many seconds the sample run lasts, one point each.
const SECONDS: usize = 60;
/// The p99 goal of the reference line, in milliseconds.
const P99_GOAL: f64 = 50.0;
/// The top of the latency axis, in milliseconds. The spike goes past it and stays on the edge.
const LATENCY_MAX: f64 = 100.0;
/// The top of the req/s axis.
const RPS_MAX: f64 = 2400.0;
/// The virtual users at the end of the ramp.
const VUS_MAX: f64 = 50.0;
/// The virtual users area fills this much of the plot at most, like in the design.
const VUS_HEIGHT: f64 = 0.9;

/// One series the legend can hide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Rps,
    P50,
    P90,
    P99,
    Vus,
    Baseline,
}

impl Key {
    /// Every series, in the order the legend lists them.
    const ALL: [Key; 6] = [
        Key::Rps,
        Key::P50,
        Key::P90,
        Key::P99,
        Key::Vus,
        Key::Baseline,
    ];

    fn label(self) -> &'static str {
        match self {
            Key::Rps => "Req/s",
            Key::P50 => "p50",
            Key::P90 => "p90",
            Key::P99 => "p99",
            Key::Vus => "VUs",
            Key::Baseline => "Run #2 p99",
        }
    }

    /// The sample colors of the design. They are the gallery's own, not telaio tokens.
    fn color(self, palette: &telaio_ui::Palette) -> Hsla {
        match self {
            Key::Rps => hex("#a3a9ff"),
            Key::P50 => hex("#5fc9b8"),
            Key::P90 => hex("#e8b45c"),
            Key::P99 => hex("#f08060"),
            Key::Vus => palette.fg_subtle,
            Key::Baseline => hex("#f08060").opacity(0.55),
        }
    }

    fn dashed(self) -> bool {
        self == Key::Baseline
    }
}

/// The sample run: one value per second for every series.
pub struct LoadTestData {
    vus: Arc<[f64]>,
    rps: Arc<[f64]>,
    p50: Arc<[f64]>,
    p90: Arc<[f64]>,
    p99: Arc<[f64]>,
    baseline: Arc<[f64]>,
}

impl LoadTestData {
    /// A 60 s run: the VUs ramp from 0 to 50 over 10 s, req/s hover around 2,100 with a dip
    /// between seconds 36 and 43, and the latencies spike there. Deterministic.
    pub fn sample() -> Self {
        let mut seed: u32 = 11;
        let mut noise = move || {
            seed = (seed * 9301 + 49297) % 233_280;
            f64::from(seed) / 233_280.0
        };
        let (mut vus, mut rps, mut p50, mut p90, mut p99, mut baseline) = (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        for second in 0..SECONDS {
            let users = (5.0 * (second as f64 + 1.0)).min(VUS_MAX);
            let ramp = users / VUS_MAX;
            let spike = if (36..=43).contains(&second) {
                ((second as f64 - 35.5) / 8.0 * std::f64::consts::PI).sin()
            } else {
                0.0
            };
            let throughput = ramp * (2150.0 + noise() * 120.0 - 60.0) * (1.0 - 0.3 * spike);
            let median = 6.0 + 4.0 * ramp + noise() * 1.5 + 5.0 * spike;
            let ninety = median * 2.1 + noise() * 2.0 + 18.0 * spike;
            let ninety_nine = ninety * 1.5 + noise() * 4.0 + 40.0 * spike;
            vus.push(users);
            rps.push(throughput);
            p50.push(median);
            p90.push(ninety);
            p99.push(ninety_nine);
            baseline.push(median * 2.1 * 1.45 + noise() * 3.0);
        }
        Self {
            vus: vus.into(),
            rps: rps.into(),
            p50: p50.into(),
            p90: p90.into(),
            p99: p99.into(),
            baseline: baseline.into(),
        }
    }

    fn values(&self, key: Key) -> Arc<[f64]> {
        match key {
            Key::Rps => self.rps.clone(),
            Key::P50 => self.p50.clone(),
            Key::P90 => self.p90.clone(),
            Key::P99 => self.p99.clone(),
            Key::Vus => self.vus.clone(),
            Key::Baseline => self.baseline.clone(),
        }
    }
}

/// What the Charts tab keeps: the sample data, the series the legend hid and the range the chart
/// reported.
pub struct ChartsDemo {
    data: LoadTestData,
    hidden: [bool; Key::ALL.len()],
    selection: ChartSelection,
}

impl ChartsDemo {
    /// The tab's initial state: everything shown, no range.
    pub fn new() -> Self {
        Self {
            data: LoadTestData::sample(),
            hidden: [false; Key::ALL.len()],
            selection: None,
        }
    }

    fn is_hidden(&self, key: Key) -> bool {
        self.hidden[Key::ALL.iter().position(|k| *k == key).unwrap_or_default()]
    }

    fn toggle(&mut self, key: Key) {
        if let Some(index) = Key::ALL.iter().position(|k| *k == key) {
            self.hidden[index] = !self.hidden[index];
        }
    }
}

/// `value` rounded, with a comma between thousands: 2100 is "2,100".
fn thousands(value: f64) -> String {
    let digits = format!("{:.0}", value.abs());
    let mut grouped = String::new();
    for (position, digit) in digits.chars().enumerate() {
        if position > 0 && (digits.len() - position) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if value < 0.0 && grouped != "0" {
        grouped.insert(0, '-');
    }
    grouped
}

/// A latency in milliseconds: "4.2 ms" under ten, "42 ms", then "1.2 s" from a thousand.
fn latency(value: f64) -> String {
    if value >= 1000.0 {
        format!("{:.1} s", value / 1000.0)
    } else if value < 10.0 {
        format!("{value:.1} ms")
    } else {
        format!("{value:.0} ms")
    }
}

/// The tooltip's time line for second `index`: "00:36 - 00:37".
fn time_range(index: usize) -> String {
    format!("00:{index:02} - 00:{:02}", index + 1)
}

/// The label of the range the chart reported, seconds from the start: "34 to 46 s".
fn range_label(selection: (usize, usize)) -> String {
    format!("{} to {} s", selection.0, selection.1 + 1)
}

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_charts_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section(
                "Time series chart",
                self.render_load_test_chart(cx),
            ))
            .child(section("Gaps and one axis", self.render_gaps_chart(cx)))
    }

    /// The load test card: legend toggles, two axes, an area, a dashed baseline, a reference line,
    /// the hover tooltip and the range selection.
    fn render_load_test_chart(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.charts;

        let hint = match demo.selection {
            Some(selection) => format!("Range {}. Click to clear it", range_label(selection)),
            None => "Drag to select a range".to_string(),
        };
        let legend = Key::ALL.map(|key| {
            let color = key.color(&palette);
            let hidden = demo.is_hidden(key);
            // A filled bar, since a zero height box with a top border is drawn without its
            // color; the dashed key is two short dashes.
            let swatch = if key.dashed() {
                h_flex()
                    .w(px(12.0))
                    .gap(px(2.0))
                    .child(div().w(px(5.0)).h(px(2.0)).bg(color))
                    .child(div().w(px(5.0)).h(px(2.0)).bg(color))
            } else {
                h_flex().child(div().w(px(12.0)).h(px(2.0)).bg(color))
            };
            h_flex()
                .id(SharedString::from(format!(
                    "gallery-legend-{}",
                    key.label()
                )))
                .items_center()
                .gap(px(6.0))
                .h(px(22.0))
                .px(px(6.0))
                .rounded(px(5.0))
                .cursor_pointer()
                .text_color(palette.fg_muted)
                .when(hidden, |item| item.opacity(0.4))
                .hover(|style| style.bg(palette.hover))
                .on_click(Self::click(cx, move |this, _, cx| {
                    this.charts.toggle(key);
                    this.record(
                        format!(
                            "{} {}",
                            key.label(),
                            if this.charts.is_hidden(key) {
                                "hidden"
                            } else {
                                "shown"
                            }
                        ),
                        cx,
                    );
                }))
                .child(swatch)
                .child(key.label())
        });

        let mut chart = TimeSeriesChart::new("gallery-load-test-chart", SECONDS)
            .surface(palette.surface)
            .left_axis(
                ChartAxis::new(RPS_MAX, ["0", "600", "1,200", "1,800", "2,400/s"])
                    .label_color(Key::Rps.color(&palette)),
            )
            .right_axis(ChartAxis::new(
                LATENCY_MAX,
                ["0", "25", "50", "75", "100 ms"],
            ))
            .x_labels(["0s", "10s", "20s", "30s", "40s", "50s", "60s"])
            .time_label(|index| time_range(index).into())
            .selection(demo.selection)
            .on_select(Self::act_with(
                cx,
                |this, selection: ChartSelection, _, cx| {
                    this.charts.selection = selection;
                    let text = match selection {
                        Some(selection) => format!("Selected {}", range_label(selection)),
                        None => "Selection cleared".to_string(),
                    };
                    this.record(text, cx);
                },
            ));

        // Drawn in this order: the area under everything, the baseline, then the latencies and
        // the throughput on top.
        let order = [
            Key::Vus,
            Key::Baseline,
            Key::P99,
            Key::P90,
            Key::P50,
            Key::Rps,
        ];
        for key in order.into_iter().filter(|key| !demo.is_hidden(*key)) {
            let color = key.color(&palette);
            let values = demo.data.values(key);
            let series = ChartSeries::new(key.label(), color, values);
            let series = match key {
                Key::Rps => series
                    .style(SeriesStyle::line(2.25))
                    .format(|value| thousands(value).into()),
                Key::P50 | Key::P90 => series
                    .side(AxisSide::Right)
                    .style(SeriesStyle::line(1.5))
                    .format(|value| latency(value).into()),
                Key::P99 => series
                    .side(AxisSide::Right)
                    .style(SeriesStyle::line(1.75))
                    .format(|value| latency(value).into()),
                Key::Baseline => series
                    .side(AxisSide::Right)
                    .style(SeriesStyle::dashed(1.5, 4.0, 4.0))
                    .format(|value| latency(value).into()),
                Key::Vus => series
                    .style(SeriesStyle::area(
                        palette.fg_muted.opacity(0.08),
                        palette.fg_muted.opacity(0.35),
                    ))
                    .scale_max(VUS_MAX / VUS_HEIGHT)
                    .dot(false)
                    .format(|value| thousands(value).into()),
            };
            chart = chart.series(series);
        }
        chart = chart.reference_line(ReferenceLine::new(
            AxisSide::Right,
            P99_GOAL,
            "p99 goal 50 ms",
            palette.danger,
        ));

        v_flex()
            .gap_2()
            .px_4()
            .pt(px(12.0))
            .pb(px(10.0))
            .rounded(px(RADIUS_LG))
            .border_1()
            .border_color(palette.border)
            .bg(palette.surface)
            .child(
                h_flex()
                    .items_center()
                    .gap(px(14.0))
                    .text_size(px(12.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Performance over time"),
                    )
                    .child(div().flex_1().text_color(palette.fg_subtle).child(hint))
                    .children(legend),
            )
            .child(chart)
    }

    /// A smaller chart with one axis, a series that has gaps and one that starts late.
    fn render_gaps_chart(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let wave: Vec<f64> = (0..30)
            .map(|index| {
                if (11..=14).contains(&index) || index == 22 {
                    f64::NAN
                } else {
                    40.0 + 30.0 * (index as f64 / 4.0).sin()
                }
            })
            .collect();
        // Shorter than the chart: the missing points are gaps too.
        let late: Vec<f64> = (0..18).map(|index| 10.0 + index as f64 * 3.0).collect();
        v_flex()
            .px_4()
            .pt(px(12.0))
            .pb(px(10.0))
            .rounded(px(RADIUS_LG))
            .border_1()
            .border_color(palette.border)
            .bg(palette.raised)
            .child(
                TimeSeriesChart::new("gallery-gaps-chart", 30)
                    .height(160.0)
                    .gutters(40.0, 16.0)
                    .left_axis(ChartAxis::new(100.0, ["0", "50", "100 %"]))
                    .x_labels(["0", "15", "29"])
                    .series(
                        ChartSeries::new("CPU", hex("#5fc9b8"), wave)
                            .format(|value| format!("{value:.0} %").into()),
                    )
                    .series(
                        ChartSeries::new("Memory", hex("#e8b45c"), late)
                            .style(SeriesStyle::dashed(1.5, 4.0, 3.0))
                            .format(|value| format!("{value:.0} %").into()),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{LoadTestData, SECONDS, latency, range_label, thousands, time_range};

    #[test]
    fn the_sample_has_a_point_per_second_for_every_series() {
        let data = LoadTestData::sample();
        for values in [
            &data.vus,
            &data.rps,
            &data.p50,
            &data.p90,
            &data.p99,
            &data.baseline,
        ] {
            assert_eq!(values.len(), SECONDS);
        }
    }

    #[test]
    fn the_sample_ramps_up_and_dips_with_a_latency_spike() {
        let data = LoadTestData::sample();
        assert_eq!(data.vus[0], 5.0);
        assert_eq!(data.vus[9], 50.0);
        assert_eq!(data.vus[59], 50.0);
        assert!(data.rps[20] > 1900.0);
        // The worst second is inside the dip, with the highest p99.
        let worst = data
            .p99
            .iter()
            .copied()
            .enumerate()
            .fold((0, 0.0), |worst, (index, value)| {
                if value > worst.1 {
                    (index, value)
                } else {
                    worst
                }
            });
        assert!((36..=43).contains(&worst.0));
        assert!(data.rps[worst.0] < data.rps[20]);
    }

    #[test]
    fn numbers_are_grouped_and_latencies_scaled() {
        assert_eq!(thousands(2100.4), "2,100");
        assert_eq!(thousands(999.0), "999");
        assert_eq!(thousands(1_234_567.0), "1,234,567");
        assert_eq!(thousands(0.0), "0");
        assert_eq!(latency(4.24), "4.2 ms");
        assert_eq!(latency(42.4), "42 ms");
        assert_eq!(latency(1250.0), "1.2 s");
        assert_eq!(time_range(5), "00:05 - 00:06");
        assert_eq!(range_label((34, 45)), "34 to 46 s");
    }
}
