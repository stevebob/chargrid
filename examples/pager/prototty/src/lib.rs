extern crate prototty;
use prototty::decorator::*;
use prototty::input::*;
use prototty::input::{keys, Input, KeyboardInput};
use prototty::render::*;
use prototty::text::*;

pub enum ControlFlow {
    Exit,
}

pub struct AppState {
    text: String,
    border_style: BorderStyle,
    bound: Size,
    background: Rgb24,
    alignment: Alignment,
    vertical_scroll_state: VerticalScrollState,
    vertical_scroll_bar_style: VerticalScrollBarStyle,
}

pub struct AppView {
    vertical_scroll_limits: VerticalScrollLimits,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            vertical_scroll_limits: VerticalScrollLimits::new(),
        }
    }
}

impl AppState {
    pub fn new(text: String) -> Self {
        Self {
            text,
            border_style: BorderStyle {
                title_style: Style {
                    bold: Some(true),
                    foreground: Some(Rgb24::new(0, 255, 0)),
                    background: Some(Rgb24::new(0, 64, 0)),
                    ..Default::default()
                },
                padding: BorderPadding {
                    right: 0,
                    left: 2,
                    top: 1,
                    bottom: 1,
                },
                ..BorderStyle::default_with_title("Pager")
            },
            bound: Size::new(40, 30),
            background: Rgb24::new(80, 80, 0),
            alignment: Alignment::centre(),
            vertical_scroll_state: VerticalScrollState::new(),
            vertical_scroll_bar_style: VerticalScrollBarStyle::default(),
        }
    }
    pub fn tick<I>(&mut self, inputs: I, view: &AppView) -> Option<ControlFlow>
    where
        I: IntoIterator<Item = Input>,
    {
        for input in inputs {
            match input {
                Input::Keyboard(keys::ETX)
                | Input::Keyboard(keys::ESCAPE)
                | Input::Keyboard(KeyboardInput::Char('q')) => {
                    return Some(ControlFlow::Exit);
                }
                Input::Mouse(MouseInput::MouseScroll { direction, .. }) => match direction {
                    ScrollDirection::Up => self.vertical_scroll_state.scroll_up_line(&view.vertical_scroll_limits),
                    ScrollDirection::Down => self
                        .vertical_scroll_state
                        .scroll_down_line(&view.vertical_scroll_limits),
                    _ => (),
                },
                Input::Keyboard(KeyboardInput::Up) => {
                    self.vertical_scroll_state.scroll_up_line(&view.vertical_scroll_limits)
                }
                Input::Keyboard(KeyboardInput::Down) => self
                    .vertical_scroll_state
                    .scroll_down_line(&view.vertical_scroll_limits),
                Input::Keyboard(KeyboardInput::PageUp) => {
                    self.vertical_scroll_state.scroll_up_page(&view.vertical_scroll_limits)
                }
                Input::Keyboard(KeyboardInput::PageDown) => self
                    .vertical_scroll_state
                    .scroll_down_page(&view.vertical_scroll_limits),
                Input::Keyboard(KeyboardInput::Home) | Input::Keyboard(KeyboardInput::Char('g')) => {
                    self.vertical_scroll_state.scroll_to_top(&view.vertical_scroll_limits)
                }
                Input::Keyboard(KeyboardInput::End) | Input::Keyboard(KeyboardInput::Char('G')) => self
                    .vertical_scroll_state
                    .scroll_to_bottom(&view.vertical_scroll_limits),
                _ => (),
            }
        }
        None
    }
}

impl<'a> View<&'a AppState> for AppView {
    fn view<F: Frame, C: ColModify>(&mut self, app_state: &'a AppState, context: ViewContext<C>, frame: &mut F) {
        let rich_text = &[
            ("Hello, World!\nblah\nblah blah ", Style::default()),
            (
                "blue\n",
                Style {
                    foreground: Some(Rgb24::new(0, 0, 255)),
                    bold: Some(true),
                    ..Default::default()
                },
            ),
            ("User string:\n", Default::default()),
            (
                app_state.text.as_ref(),
                Style {
                    background: Some(Rgb24::new(187, 0, 0)),
                    underline: Some(true),
                    ..Default::default()
                },
            ),
        ];
        AlignView {
            alignment: app_state.alignment,
            view: &mut FillBackgroundView {
                rgb24: app_state.background,
                view: &mut BorderView {
                    style: &app_state.border_style,
                    view: &mut BoundView {
                        size: app_state.bound,
                        view: &mut VerticalScrollView {
                            limits: &mut self.vertical_scroll_limits,
                            state: app_state.vertical_scroll_state,
                            scroll_bar_style: &app_state.vertical_scroll_bar_style,
                            view: &mut RichTextView::new(wrap::Word::new()),
                        },
                    },
                },
            },
        }
        .view(rich_text, context, frame);
    }
}
