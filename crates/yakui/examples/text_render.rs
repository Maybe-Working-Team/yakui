use std::{cell::Cell, sync::Arc};

use bootstrap::OPENMOJI;
use yakui::cosmic_text::fontdb;
use yakui::{column, font::Fonts, text, util::widget, widget::Widget, Vec2};

#[derive(Debug)]
struct LoadFontsWidget {
    loaded: Cell<bool>,
}

impl Widget for LoadFontsWidget {
    type Props<'a> = ();

    type Response = ();

    fn new() -> Self {
        Self {
            loaded: Cell::default(),
        }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}

    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        _constraints: yakui::Constraints,
    ) -> yakui::Vec2 {
        if !self.loaded.get() {
            let fonts = ctx.dom.get_global_or_init(Fonts::default);

            fonts.load_font_source(fontdb::Source::Binary(Arc::from(&OPENMOJI)));

            self.loaded.set(true);
        }

        Vec2::ZERO
    }
}

pub fn run() {
    widget::<LoadFontsWidget>(());

    yakui::center(|| {
        text(96.0, "  Sphinx of Black Quartz, hear my vow");
    });
}

fn main() {
    bootstrap::start(run as fn());
}
