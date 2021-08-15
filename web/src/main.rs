mod bindings;

use intcode::{Error, Warning};
use yew::prelude::*;

use crate::bindings::Editor;

pub enum Msg {
    Run,
}

#[derive(Debug)]
struct App {
    link: ComponentLink<Self>,
    editor: Editor,
    output: Html,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            editor: Editor::default(),
            output: String::from("Write your program and hit \"Build\"...").into(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Run => {
                let input = self.editor.text().unwrap();
                let p = intcode::Pretty::new(&input);
                let mut result = String::new();
                intcode::assemble::to_intcode(&input)
                    .map(|(_, warnings)| {
                        result.push_str("Successfully compiled program.\n");
                        for Warning { msg, span } in warnings {
                            result.push_str(&p.warn(msg, span));
                            result.push('\n');
                        }
                    })
                    .map_err(|(errors, warnings)| {
                        result.push_str("Failed to compile program.\n");
                        for Warning { msg, span } in warnings {
                            result.push_str(&p.warn(msg, span));
                            result.push('\n');
                        }
                        for Error { msg, span } in errors {
                            result.push_str(&p.error(msg, span));
                            result.push('\n');
                        }
                    })
                    .ok();
                self.output = html! { <pre>{result}</pre> };
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <div id="header">
                    <button onclick={self.link.callback(|_| Msg::Run)}>{"Build"}</button>
                </div>

                <div id="main" class="auto">
                    <div id="left" class="auto">
                        <div id="editor" class="auto">
                            {include_str!("../../examples/hello-world.s")}
                        </div>
                    </div>
                    <div id="right" class="auto">
                        {self.output.clone()}
                    </div>
                </div>

            </div>
        }
    }
    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.editor.init();
        }
    }
}

fn main() {
    yansi::Paint::disable();
    yew::start_app::<App>();
}
