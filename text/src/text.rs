use crate::wrap::{self, Wrap};
use prototty_render::*;

pub struct TextView<W: Wrap> {
    pub style: Style,
    wrap: W,
}

impl<W: Wrap> TextView<W> {
    pub fn new(style: Style, wrap: W) -> Self {
        Self { style, wrap }
    }
}

impl<S, I, W> View<I> for TextView<W>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
    W: Wrap,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        parts: I,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.wrap.clear();
        for part in parts {
            let part = part.as_ref();
            for character in part.chars() {
                self.wrap
                    .process_character(character, self.style, context, grid);
            }
        }
        self.wrap.flush(context, grid);
    }
}

pub struct StringView<W: Wrap> {
    pub style: Style,
    wrap: W,
}

impl<W: Wrap> StringView<W> {
    pub fn new(style: Style, wrap: W) -> Self {
        Self { style, wrap }
    }
}

impl<'a, S, W> View<S> for StringView<W>
where
    S: AsRef<str>,
    W: Wrap,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        part: S,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.wrap.clear();
        let part = part.as_ref();
        for character in part.chars() {
            self.wrap
                .process_character(character, self.style, context, grid);
        }
        self.wrap.flush(context, grid);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct StringViewSingleLine {
    pub style: Style,
}

impl StringViewSingleLine {
    pub fn new(style: Style) -> Self {
        Self { style }
    }
}

impl<'a, S> View<S> for StringViewSingleLine
where
    S: AsRef<str>,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        part: S,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        StringView::new(self.style, wrap::None::new()).view(part, context, grid);
    }

    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        part: S,
        _context: ViewContext<R>,
    ) -> Size {
        let part = part.as_ref();
        let width = part.len() as u32;
        Size::new(width, 1)
    }
}
