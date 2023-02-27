use std::ops::Range;
use std::path::PathBuf;

#[cfg(not(feature = "system-font"))]
use anyhow::bail;

use anyhow::Result;
use hdr10plus::metadata::{PeakBrightnessSource, VariablePeakBrightness};
use hdr10plus::metadata_json::{Hdr10PlusJsonMetadata, MetadataJsonRoot};
use plotters::coord::ranged1d::{KeyPointHint, NoDefaultFormatting, Ranged, ValueFormatter};
use plotters::coord::types::RangedCoordusize;
use plotters::prelude::{
    AreaSeries, BitMapBackend, Cartesian2d, ChartBuilder, ChartContext, IntoDrawingArea,
    PathElement, SeriesLabelPosition, WHITE,
};
use plotters::style::{Color, IntoTextStyle, RGBColor, ShapeStyle, BLACK};

use crate::utils::{nits_to_pq, pq_to_nits};

use super::{input_from_either, PlotArgs};

#[cfg(not(feature = "system-font"))]
const NOTO_SANS_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/NotoSans-Regular.ttf"
));

const MAXSCL_COLOR: RGBColor = RGBColor(65, 105, 225);
const AVERAGE_COLOR: RGBColor = RGBColor(75, 0, 130);

pub struct Plotter {
    input: PathBuf,
    peak_brightness_source: PeakBrightnessSource,
}

impl Plotter {
    pub fn plot(args: PlotArgs) -> Result<()> {
        #[cfg(not(feature = "system-font"))]
        {
            let res = plotters::style::register_font(
                "sans-serif",
                plotters::style::FontStyle::Normal,
                NOTO_SANS_REGULAR,
            );

            if res.is_err() {
                bail!("Failed registering font!");
            }
        }

        let PlotArgs {
            input,
            input_pos,
            output,
            title,
            peak_source,
        } = args;

        let output = output.unwrap_or(PathBuf::from("hdr10plus_plot.png"));
        let title = title.unwrap_or(String::from("HDR10+ plot"));

        let input = input_from_either("info", input, input_pos)?;
        let plotter = Plotter {
            input,
            peak_brightness_source: PeakBrightnessSource::from(peak_source),
        };

        println!("Parsing JSON file...");
        let metadata_root = MetadataJsonRoot::from_file(&plotter.input)?;
        let frames = &metadata_root.scene_info;

        let x_spec = 0..frames.len();

        let root = BitMapBackend::new(&output, (3000, 1200)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root
            .margin(30, 30, 60, 60)
            .titled(&title, ("sans-serif", 40))?;

        println!("Plotting...");

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .margin_top(90)
            .build_cartesian_2d(x_spec, PqCoord {})?;

        chart
            .configure_mesh()
            .bold_line_style(BLACK.mix(0.10))
            .light_line_style(BLACK.mix(0.01))
            .label_style(("sans-serif", 22))
            .axis_desc_style(("sans-serif", 24))
            .x_desc("frames")
            .x_max_light_lines(1)
            .x_labels(24)
            .y_desc("nits (cd/m²)")
            .draw()?;

        plotter.draw_brightness_series(&mut chart, frames)?;
        chart
            .configure_series_labels()
            .border_style(BLACK)
            .position(SeriesLabelPosition::LowerLeft)
            .label_font(("sans-serif", 24))
            .background_style(WHITE)
            .draw()?;

        let caption_style = ("sans-serif", 24).into_text_style(&root);
        let chart_caption = format!(
            "Frames: {}. Profile {}. Scenes: {}.",
            frames.len(),
            metadata_root.info.profile,
            metadata_root.scene_info_summary.scene_frame_numbers.len()
        );
        root.draw_text(&chart_caption, &caption_style, (60, 35))?;

        let chart_caption = format!("Peak brightness source: {}", plotter.peak_brightness_source);
        root.draw_text(&chart_caption, &caption_style, (60, 60))?;

        root.present()?;

        println!("Done.");

        Ok(())
    }

    fn draw_brightness_series(
        &self,
        chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordusize, PqCoord>>,
        frames: &[Hdr10PlusJsonMetadata],
    ) -> Result<()> {
        let data: Vec<_> = frames
            .iter()
            .map(|f| {
                let lum_params = &f.luminance_parameters;
                let avg_nits = lum_params.average_rgb as f64 / 10.0;
                let peak_nits = f.peak_brightness_nits(self.peak_brightness_source).unwrap();

                (nits_to_pq(avg_nits), nits_to_pq(peak_nits))
            })
            .collect();

        let maxfall = data
            .iter()
            .map(|e| e.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let maxfall_avg = data.iter().map(|e| e.0).sum::<f64>() / data.len() as f64;
        let maxcll = data
            .iter()
            .map(|e| e.1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let maxcll_avg = data.iter().map(|e| e.1).sum::<f64>() / data.len() as f64;

        let avg_series_label = format!(
            "Average (MaxFALL: {:.2} nits, avg: {:.2} nits)",
            pq_to_nits(maxfall),
            pq_to_nits(maxfall_avg),
        );
        let max_series_label = format!(
            "Maximum (MaxCLL: {:.2} nits, avg: {:.2} nits)",
            pq_to_nits(maxcll),
            pq_to_nits(maxcll_avg),
        );

        let avg_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.0)),
            0.0,
            AVERAGE_COLOR.mix(0.50),
        )
        .border_style(AVERAGE_COLOR);
        let maxscl_series = AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, y.1)),
            0.0,
            MAXSCL_COLOR.mix(0.25),
        )
        .border_style(MAXSCL_COLOR);

        chart
            .draw_series(maxscl_series)?
            .label(max_series_label)
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: MAXSCL_COLOR.to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    },
                )
            });
        chart
            .draw_series(avg_series)?
            .label(avg_series_label)
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: AVERAGE_COLOR.to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    },
                )
            });

        Ok(())
    }
}

pub struct PqCoord {}

impl Ranged for PqCoord {
    type FormatOption = NoDefaultFormatting;
    type ValueType = f64;

    fn map(&self, value: &f64, limit: (i32, i32)) -> i32 {
        let size = limit.1 - limit.0;
        (*value * size as f64) as i32 + limit.0
    }

    fn key_points<Hint: KeyPointHint>(&self, _hint: Hint) -> Vec<f64> {
        vec![
            nits_to_pq(0.01),
            nits_to_pq(0.1),
            nits_to_pq(0.5),
            nits_to_pq(1.0),
            nits_to_pq(2.5),
            nits_to_pq(5.0),
            nits_to_pq(10.0),
            nits_to_pq(25.0),
            nits_to_pq(50.0),
            nits_to_pq(100.0),
            nits_to_pq(200.0),
            nits_to_pq(400.0),
            nits_to_pq(600.0),
            nits_to_pq(1000.0),
            nits_to_pq(2000.0),
            nits_to_pq(4000.0),
            nits_to_pq(10000.0),
        ]
    }

    fn range(&self) -> Range<f64> {
        0_f64..1.0_f64
    }
}

impl ValueFormatter<f64> for PqCoord {
    fn format_ext(&self, value: &f64) -> String {
        let nits = (pq_to_nits(*value) * 1000.0).round() / 1000.0;
        format!("{nits}")
    }
}
