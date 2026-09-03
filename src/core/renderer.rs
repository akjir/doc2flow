//! HTML AST renderer and element formatting engine.

use std::cell::Cell;
use std::fmt::{self, Write as _};

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;

/// Maximum capacity of active feature modules tracked concurrently.
pub const MAX_ACTIVE_FEATURES: usize = 32;

/// Trait for document element renderers declaring supported AST element types.
pub trait DocumentElementRenderer: Send + Sync + fmt::Debug {
    /// Returns a slice of supported AST element IDs handled by this renderer.
    fn supported(&self) -> &[DocumentElementId] {
        &[]
    }

    /// Renders a document element directly into an output buffer.
    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
        renderer: &HtmlRenderer,
    );
}

/// Document AST HTML renderer managing active features and element interception.
#[derive(Debug)]
pub struct HtmlRenderer<'a> {
    active_mask: Cell<u32>,
    h1_counter: Cell<u32>,
    h2_counter: Cell<u32>,
    modules: &'a [&'static dyn FeatureModule],
    slots: [Option<(usize, &'static dyn FeatureModule)>; DocumentElementId::COUNT],
}

impl<'a> HtmlRenderer<'a> {
    /// Creates a new HTML renderer configured with the specified slice of active features.
    ///
    /// # Panics
    ///
    /// Panics if the number of feature modules exceeds 32 (the bit width of the tracking bitmask).
    #[must_use]
    pub fn new(modules: &'a [&'static dyn FeatureModule]) -> Self {
        assert!(
            modules.len() <= 32,
            "feature modules count ({}) exceeds bitmask capacity of 32",
            modules.len()
        );
        let mut slots = [None; DocumentElementId::COUNT];
        for (i, &module) in modules.iter().enumerate() {
            for &id in module.supported() {
                slots[id as usize] = Some((i, module));
            }
        }
        Self {
            active_mask: Cell::new(0),
            h1_counter: Cell::new(0),
            h2_counter: Cell::new(0),
            modules,
            slots,
        }
    }

    /// Creates an HTML renderer with all default features enabled.
    #[must_use]
    pub fn default_renderer() -> HtmlRenderer<'static> {
        HtmlRenderer::new(&crate::features::ALL_FEATURE_MODULES)
    }
}

impl Default for HtmlRenderer<'static> {
    fn default() -> Self {
        Self::default_renderer()
    }
}

impl<'a> HtmlRenderer<'a> {

    /// Populates a fixed-size buffer with active feature module references and returns the slice.
    pub fn active_features<'b>(
        &self,
        buffer: &'b mut [&'static dyn FeatureModule; MAX_ACTIVE_FEATURES],
    ) -> &'b [&'static dyn FeatureModule] {
        let mask = self.active_mask.get();
        let mut count = 0;
        for (i, &module) in self.modules.iter().enumerate() {
            if (mask & (1 << i)) != 0 || module.name() == "core" {
                debug_assert!(
                    count < buffer.len(),
                    "active features count exceeds MAX_ACTIVE_FEATURES buffer capacity"
                );
                if count < buffer.len() {
                    buffer[count] = module;
                    count += 1;
                }
            }
        }
        &buffer[..count]
    }

    /// Marks the feature module at the specified index as active in O(1) time.
    #[inline]
    fn mark_module_active(&self, index: usize) {
        debug_assert!(index < 32, "module index exceeds bitmask width");
        self.active_mask.set(self.active_mask.get() | (1 << index));
    }

    /// Marks the feature module with the given name as active.
    pub fn mark_feature_active(&self, name: &str) {
        for (i, &module) in self.modules.iter().enumerate() {
            if module.name() == name {
                self.mark_module_active(i);
                break;
            }
        }
    }

    /// Renders a single document element and its children into the output buffer.
    pub fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
    ) {
        let id = element.element_id();
        if let Some((index, module)) = self.slots[id as usize] {
            self.mark_module_active(index);
            module.render_element(element, indent, depth, parameters, out, self);
        } else {
            panic!("No feature module registered for element {}", id as usize);
        }
    }

    /// Renders a slice of document elements sequentially into the output buffer.
    pub fn render_children(
        &self,
        children: &[DocumentElement],
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
    ) {
        for child in children {
            self.render_element(child, indent, depth, parameters, out);
        }
    }

    /// Advances section counters and writes section number prefix into output buffer.
    pub fn write_section_prefix(&self, level: usize, out: &mut String) {
        if level == 1 {
            let next_h1 = self.h1_counter.get() + 1;
            self.h1_counter.set(next_h1);
            self.h2_counter.set(0);
            let _ = write!(out, "{next_h1}. ");
        } else if level == 2 {
            let h1 = self.h1_counter.get();
            let next_h2 = self.h2_counter.get() + 1;
            self.h2_counter.set(next_h2);
            let _ = write!(out, "{h1}.{next_h2} ");
        }
    }
}

/// Renders a document element and its children directly into an output buffer.
///
/// Recursively processes nested child elements and delegates to registered feature renderers.
pub fn render_element_into(
    element: &DocumentElement,
    indent: usize,
    parameters: &DocumentParameters,
    out: &mut String,
) {
    crate::core::language::init(&parameters.language);
    let renderer = HtmlRenderer::default_renderer();
    renderer.render_element(element, indent, 0, parameters, out);
}

/// Renders a document element and its children into an HTML string representation.
///
/// Recursively processes nested child elements and delegates to registered feature renderers.
///
/// # Examples
///
/// ```
/// use doc2flow::core::renderer::render_element;
/// use doc2flow::core::document::{DocumentElement, DocumentParameters};
///
/// let element = DocumentElement::text("Hello world");
/// let params = DocumentParameters::default();
/// let html = render_element(&element, 1, &params);
/// assert!(html.contains("Hello world"));
/// assert!(html.contains("item-comment-icon"));
/// ```
pub fn render_element(
    element: &DocumentElement,
    indent: usize,
    parameters: &DocumentParameters,
) -> String {
    let mut out = String::with_capacity(512);
    render_element_into(element, indent, parameters, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::comment::COMMENT_ICON_SVG;

    #[test]
    fn test_render_element_text() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let text = DocumentElement::text("Sample paragraph text");
        assert_eq!(
            render_element(&text, 1, &DocumentParameters::default()),
            format!(
                "  <div class=\"item text-item item-selectable\">\n    <span class=\"text-content\">\n      Sample paragraph text\n    </span>\n    {COMMENT_ICON_SVG}\n  </div>\n"
            )
        );
    }

    #[test]
    fn test_render_element_unknown() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let unknown = DocumentElement::unknown("Unrecognized markdown");
        assert_eq!(
            render_element(&unknown, 1, &DocumentParameters::default()),
            "  <p class=\"unknown-default\">\n    Unrecognized markdown\n  </p>\n"
        );
    }

    #[test]
    fn test_render_element_section_with_children() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let section = DocumentElement::section(
            2,
            "Details",
            vec![
                DocumentElement::text("First paragraph"),
                DocumentElement::text("Second paragraph"),
            ],
        );
        let expected = format!(
            concat!(
                "  <section class=\"section\" data-level=\"2\">\n",
                "    <h2 class=\"section-header\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">\n",
                "      <span class=\"section-title\">Details</span>\n",
                "      <span class=\"section-toggler\">&#9660;</span>\n",
                "    </h2>\n",
                "    <div class=\"section-body\">\n",
                "      <div class=\"item text-item item-selectable\">\n",
                "        <span class=\"text-content\">\n",
                "          First paragraph\n",
                "        </span>\n",
                "        {COMMENT_ICON_SVG}\n",
                "      </div>\n",
                "      <div class=\"item text-item item-selectable\">\n",
                "        <span class=\"text-content\">\n",
                "          Second paragraph\n",
                "        </span>\n",
                "        {COMMENT_ICON_SVG}\n",
                "      </div>\n",
                "    </div>\n",
                "  </section>\n"
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
        );
        assert_eq!(
            render_element(&section, 1, &DocumentParameters::default()),
            expected
        );
    }

    #[test]
    fn test_render_element_nested_sections() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let inner_section =
            DocumentElement::section(3, "Inner", vec![DocumentElement::text("Inner content")]);
        let outer_section = DocumentElement::section(1, "Outer", vec![inner_section]);
        let expected = format!(
            concat!(
                "  <section class=\"section\" data-level=\"1\">\n",
                "    <h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">\n",
                "      <span class=\"section-title\">Outer</span>\n",
                "      <span class=\"section-toggler\">&#9660;</span>\n",
                "    </h1>\n",
                "    <div class=\"section-body\">\n",
                "      <section class=\"section\" data-level=\"3\">\n",
                "        <h3 class=\"section-subheading\">Inner</h3>\n",
                "        <div class=\"section-body\">\n",
                "          <div class=\"item text-item item-selectable\">\n",
                "            <span class=\"text-content\">\n",
                "              Inner content\n",
                "            </span>\n",
                "            {COMMENT_ICON_SVG}\n",
                "          </div>\n",
                "        </div>\n",
                "      </section>\n",
                "    </div>\n",
                "  </section>\n"
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
        );
        assert_eq!(
            render_element(&outer_section, 1, &DocumentParameters::default()),
            expected
        );
    }

    #[test]
    fn test_render_element_code_block() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
        let code_block = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(
            render_element(&code_block, 1, &DocumentParameters::default()),
            "  <pre class=\"code-default\" data-label-copy=\"Copy code\" data-label-copied=\"Copied!\"><code>fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_render_element_horizontal_rule() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let hr = DocumentElement::horizontal_rule();
        assert_eq!(
            render_element(&hr, 1, &DocumentParameters::default()),
            "  <hr />\n"
        );
        assert_eq!(
            render_element(&hr, 2, &DocumentParameters::default()),
            "    <hr />\n"
        );
    }

    #[test]
    fn test_render_element_bullet_list_item() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut bullet = DocumentElement::bullet_list_item("Parent bullet");
        let child = DocumentElement::bullet_list_item("Child bullet");
        bullet.push_child(child).unwrap();

        let html = render_element(&bullet, 1, &DocumentParameters::default());
        assert!(html.contains(
            "<div class=\"item bullet-item item-selectable\">\n    <span class=\"bullet-marker\">&bull;</span>"
        ));
        assert!(html.contains("<div class=\"item bullet-item item-selectable\" style=\"--indent: 1;\">"));
        assert!(html.contains("Parent bullet"));
        assert!(html.contains("Child bullet"));
    }

    #[test]
    fn test_render_element_check_box_item() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut check = DocumentElement::check_box_item(true, "Done task");
        let child = DocumentElement::check_box_item(false, "Sub task");
        check.push_child(child).unwrap();

        let html = render_element(&check, 1, &DocumentParameters::default());
        assert!(html.contains("<div class=\"item check-item item-selectable checked\">"));
        assert!(html.contains("<div class=\"item check-item item-selectable\" style=\"--indent: 1;\">"));
        assert!(html.contains("<input type=\"checkbox\" class=\"check-box\" checked />"));
        assert!(html.contains("<input type=\"checkbox\" class=\"check-box\" />"));
        assert!(html.contains("Done task"));
        assert!(html.contains("Sub task"));
    }

    #[test]
    fn test_render_element_ordered_list_item() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut order = DocumentElement::ordered_list_item(1, "First step");
        let child = DocumentElement::ordered_list_item(1, "Sub step");
        order.push_child(child).unwrap();

        let html = render_element(&order, 1, &DocumentParameters::default());
        assert!(html.contains(
            "<div class=\"item order-item item-selectable\">\n    <span class=\"order-marker\">1.</span>"
        ));
        assert!(html.contains(
            "<div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n    <span class=\"order-marker\">a.</span>"
        ));
        assert!(html.contains("First step"));
        assert!(html.contains("Sub step"));
    }

    #[test]
    fn test_render_element_image() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let image = DocumentElement::image("alt text", "test.png");
        assert_eq!(
            render_element(&image, 1, &DocumentParameters::default()),
            "  <div class=\"image-item\">\n    <img src=\"test.png\" alt=\"alt text\" />\n  </div>\n"
        );
    }

    #[test]
    fn test_render_element_input() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let input = DocumentElement::input("test_value");
        assert_eq!(
            render_element(&input, 1, &DocumentParameters::default()),
            "  <div class=\"input-wrap\">\n    <input type=\"text\" class=\"input-field\" value=\"test_value\" />\n  </div>\n"
        );
    }

    #[test]
    fn test_render_element_shoutout() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let shoutout = DocumentElement::shoutout(
            crate::core::document::ShoutoutElementKind::Note,
            "Unregistered shoutout",
        );
        assert_eq!(
            render_element(&shoutout, 1, &DocumentParameters::default()),
            "  <div class=\"shoutout shoutout-note\" data-label=\"Note\">\n    Unregistered shoutout\n  </div>\n"
        );
    }

    #[test]
    fn test_active_features_tracking() {
        let renderer = HtmlRenderer::default_renderer();
        let mut buffer =
            [&crate::features::CORE_FEATURE as &'static dyn FeatureModule; MAX_ACTIVE_FEATURES];
        let initial_active = renderer.active_features(&mut buffer);
        assert_eq!(initial_active.len(), 1);
        assert_eq!(initial_active[0].name(), "core");

        let mut out = String::new();
        let bullet = DocumentElement::bullet_list_item("item");
        renderer.render_element(&bullet, 0, 0, &DocumentParameters::default(), &mut out);

        let mut active_buffer =
            [&crate::features::CORE_FEATURE as &'static dyn FeatureModule; MAX_ACTIVE_FEATURES];
        let active = renderer.active_features(&mut active_buffer);
        assert_eq!(active.len(), 2);
        let names: Vec<&str> = active.iter().map(|m| m.name()).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"bullet"));
    }

    #[test]
    #[should_panic(expected = "No feature module registered for element")]
    fn test_render_element_panics_on_unregistered_element() {
        let mut out = String::new();
        let renderer = HtmlRenderer::new(&[]);
        let dummy_params = DocumentParameters::default();

        let shoutout = DocumentElement::shoutout(
            crate::core::document::ShoutoutElementKind::Caution,
            "caution message",
        );
        renderer.render_element(&shoutout, 0, 0, &dummy_params, &mut out);
    }

    #[test]
    #[should_panic(expected = "exceeds bitmask capacity of 32")]
    fn test_renderer_new_panics_on_too_many_modules() {
        let modules = [&crate::features::CORE_FEATURE as &'static dyn FeatureModule; 33];
        let _ = HtmlRenderer::new(&modules);
    }
}
