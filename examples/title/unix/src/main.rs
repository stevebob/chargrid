extern crate prototty_unix;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty_title::*;
use prototty_unix::prototty_render::Renderer;

fn main() {
    let mut context = prototty_unix::Context::new().unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    // render the title using the DemoTitleView
    context.render(&mut DemoTitleView, &title).unwrap();

    // exit after a key is pressed
    context.wait_input().unwrap();
}
