use crate::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    ObjectFit, Pixels, Style, StyleRefinement, Styled, Window,
};
// Rounded surface corners are only painted by the macOS surface pipeline; the
// other backends have no surface primitive to round.
#[cfg(target_os = "macos")]
use crate::Corners;
#[cfg(target_os = "macos")]
use core_video::pixel_buffer::CVPixelBuffer;
use refineable::Refineable;

/// A source of a surface's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceSource {
    /// A macOS image buffer from CoreVideo
    #[cfg(target_os = "macos")]
    Surface(CVPixelBuffer),
}

#[cfg(target_os = "macos")]
impl From<CVPixelBuffer> for SurfaceSource {
    fn from(value: CVPixelBuffer) -> Self {
        SurfaceSource::Surface(value)
    }
}

/// A surface element.
pub struct Surface {
    source: SurfaceSource,
    object_fit: ObjectFit,
    style: StyleRefinement,
    crop: Option<Bounds<f32>>,
}

#[cfg(target_os = "macos")]
fn style_corner_radii(style: &StyleRefinement, window: &Window) -> Corners<Pixels> {
    use refineable::Refineable;
    let mut s = Style::default();
    s.refine(style);
    s.corner_radii.to_pixels(window.rem_size())
}

/// Create a new surface element.
#[cfg(target_os = "macos")]
pub fn surface(source: impl Into<SurfaceSource>) -> Surface {
    Surface {
        source: source.into(),
        object_fit: ObjectFit::Contain,
        style: Default::default(),
        crop: None,
    }
}

impl Surface {
    /// Set the object fit for the image.
    pub fn object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.object_fit = object_fit;
        self
    }

    /// Show only a normalized sub-rect of the surface (0..1 in both axes).
    /// Sampled on the GPU; object-fit sizes against the cropped region.
    pub fn crop(mut self, crop: Bounds<f32>) -> Self {
        self.crop = Some(crop);
        self
    }
}

impl Element for Surface {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] window: &mut Window,
        _: &mut App,
    ) {
        match &self.source {
            #[cfg(target_os = "macos")]
            SurfaceSource::Surface(surface) => {
                let mut size = crate::size(surface.get_width().into(), surface.get_height().into());
                // Fit math scales the cropped region, not the full frame.
                let crop = super::img::sanitize_crop(self.crop);
                if let Some(c) = crop {
                    size.width = crate::DevicePixels(
                        ((i32::from(size.width) as f32) * c[2]).round().max(1.0) as i32,
                    );
                    size.height = crate::DevicePixels(
                        ((i32::from(size.height) as f32) * c[3]).round().max(1.0) as i32,
                    );
                }
                let (paint_bounds, crop4) = self.object_fit.get_bounds_and_crop(
                    bounds,
                    size,
                    crop.unwrap_or([0.0, 0.0, 1.0, 1.0]),
                );
                let corner_radii = style_corner_radii(&self.style, window);
                window.paint_surface(paint_bounds, corner_radii, surface.clone(), crop4);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
}

impl IntoElement for Surface {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for Surface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
