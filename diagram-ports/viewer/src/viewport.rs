#[derive(Clone, Copy, Debug)]
pub enum Zoom {
    Fit,
    Percent(u16),
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
