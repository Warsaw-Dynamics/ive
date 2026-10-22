#[derive(Clone, Copy, Debug)]
pub enum Zoom {
    Fit,
    Percent(u16),
}

pub struct Viewport {
    pub size: (f32, f32),
    pub device_scale: f32,
}
pub struct Layout {
    pub logical_size: (f32, f32),
    pub raster_size: (u32, u32),
}

pub fn layout(diagram: (f32, f32), viewport: Viewport, zoom: Zoom) -> Result<Layout, String> {
    if [
        diagram.0,
        diagram.1,
        viewport.size.0,
        viewport.size.1,
        viewport.device_scale,
    ]
    .iter()
    .any(|value| !value.is_finite() || *value <= 0.)
        || viewport.device_scale > 8.
    {
        return Err(format!(
            "invalid logical diagram/viewport/device scale {diagram:?}/{:?}/{}",
            viewport.size, viewport.device_scale
        ));
    }
    let (width, height) = (f64::from(diagram.0), f64::from(diagram.1));
    let factor = match zoom {
        Zoom::Fit => (f64::from(viewport.size.0) / width).min(f64::from(viewport.size.1) / height),
        Zoom::Percent(percent) if (25..=400).contains(&percent) => f64::from(percent) / 100.,
        Zoom::Percent(percent) => {
            return Err(format!("zoom {percent}% must be between 25% and 400%"));
        }
    };
    let logical_size = ((width * factor) as f32, (height * factor) as f32);
    if [logical_size.0, logical_size.1]
        .iter()
        .any(|value| !value.is_finite() || *value <= 0. || *value > 1_000_000.)
    {
        return Err(format!(
            "requested logical diagram size {logical_size:?} must be finite, positive and at most 1000000 pixels per axis"
        ));
    }
    let physical = (
        logical_size.0 * viewport.device_scale,
        logical_size.1 * viewport.device_scale,
    );
    let raster_size = raster_size(physical, (1., 1.), Zoom::Percent(100))?;
    Ok(Layout {
        logical_size,
        raster_size,
    })
}

pub fn raster_size(
    diagram: (f32, f32),
    viewport: (f32, f32),
    zoom: Zoom,
) -> Result<(u32, u32), String> {
    let (width, height) = (f64::from(diagram.0), f64::from(diagram.1));
    if [width, height, f64::from(viewport.0), f64::from(viewport.1)]
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.)
    {
        return Err(format!(
            "invalid diagram/viewport dimensions {diagram:?}/{viewport:?}"
        ));
    }
    let scale = match zoom {
        Zoom::Fit => (f64::from(viewport.0) / width).min(f64::from(viewport.1) / height),
        Zoom::Percent(percent) if (25..=400).contains(&percent) => f64::from(percent) / 100.,
        Zoom::Percent(percent) => {
            return Err(format!("zoom {percent}% must be between 25% and 400%"));
        }
    };
    let cap = (4096. / width)
        .min(4096. / height)
        .min((4_194_304. / (width * height)).sqrt());
    let scale = scale.min(cap);
    Ok((
        (width * scale).floor().max(1.) as u32,
        (height * scale).floor().max(1.) as u32,
    ))
}

#[derive(Default)]
pub struct Generation(u64);
impl Generation {
    pub fn advance(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }
    pub fn accepts(&self, request: u64) -> bool {
        self.0 == request
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_enlarge_tall_diagrams_without_enlarging_the_bounded_bitmap() {
        // given
        let cases = [
            (100, (1000., 10000.)),
            (200, (2000., 20000.)),
            (400, (4000., 40000.)),
        ];
        // when
        for (percent, expected) in cases {
            let actual = layout(
                (1000., 10000.),
                Viewport {
                    size: (800., 600.),
                    device_scale: 1.,
                },
                Zoom::Percent(percent),
            )
            .unwrap();
            // then
            assert_eq!(actual.logical_size, expected);
            assert!(actual.raster_size.0 <= 4096 && actual.raster_size.1 <= 4096);
            assert!(u64::from(actual.raster_size.0) * u64::from(actual.raster_size.1) <= 4_194_304);
        }
    }
    #[test]
    fn should_keep_logical_zoom_independent_of_device_scale() {
        // given
        let viewport = Viewport {
            size: (800., 600.),
            device_scale: 2.,
        };
        // when
        let actual = layout((1000., 10000.), viewport, Zoom::Percent(200)).unwrap();
        // then
        assert_eq!(actual.logical_size, (2000., 20000.));
        assert_eq!(actual.raster_size, (409, 4096));
    }
    #[test]
    fn should_fit_in_logical_space_at_device_scale_two() {
        // given
        let viewport = Viewport {
            size: (800., 600.),
            device_scale: 2.,
        };
        // when
        let actual = layout((1000., 10000.), viewport, Zoom::Fit).unwrap();
        // then
        assert_eq!(actual.logical_size, (60., 600.));
        assert_eq!(actual.raster_size, (120, 1200));
    }
    #[test]
    fn should_reject_unscrollable_or_nonfinite_logical_sizes() {
        // given
        let cases = [
            ((1e20, 1e20), 1.),
            ((f32::NAN, 10.), 1.),
            ((10., 10.), 0.),
            ((10., 10.), f32::INFINITY),
            ((10., 10.), 9.),
        ];
        // when
        let actual: Vec<_> = cases
            .into_iter()
            .map(|(diagram, device_scale)| {
                layout(
                    diagram,
                    Viewport {
                        size: (800., 600.),
                        device_scale,
                    },
                    Zoom::Percent(400),
                )
            })
            .collect();
        // then
        assert!(actual.iter().all(Result::is_err));
    }
    #[test]
    fn should_fit_a_diagram_without_distorting_it() {
        // given
        let diagram = (1200., 600.);
        // when
        let size = raster_size(diagram, (600., 400.), Zoom::Fit).unwrap();
        // then
        assert_eq!(size, (600, 300));
    }
    #[test]
    fn should_bound_raster_area_even_for_extreme_svg_dimensions() {
        // given
        let diagram = (1e20, 1e20);
        // when
        let (width, height) = raster_size(diagram, (800., 600.), Zoom::Percent(400)).unwrap();
        // then
        assert!(width <= 4096 && height <= 4096);
        assert!(u64::from(width) * u64::from(height) <= 4_194_304);
        assert_eq!(width, height);
    }
    #[test]
    fn should_reject_invalid_dimensions_and_zoom() {
        // given
        let cases = [
            ((0., 10.), Zoom::Fit),
            ((f32::INFINITY, 10.), Zoom::Fit),
            ((10., 10.), Zoom::Percent(0)),
        ];
        // when
        let actual: Vec<_> = cases
            .into_iter()
            .map(|(diagram, zoom)| raster_size(diagram, (800., 600.), zoom))
            .collect();
        // then
        assert!(actual.iter().all(Result::is_err));
    }
    #[test]
    fn should_drop_a_result_after_the_request_changes() {
        // given
        let mut generation = Generation::default();
        let old = generation.advance();
        // when
        let current = generation.advance();
        // then
        assert!(!generation.accepts(old));
        assert!(generation.accepts(current));
    }
}
