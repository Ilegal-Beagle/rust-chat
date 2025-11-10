use crate::ui::start;
use crate::ui::chat;

#[derive(Debug)]
pub enum State {
    Start,
    Chat,
}

impl State {
    pub fn next(&self) -> State {
        match self {
            State::Start => State::Chat,
            State::Chat => todo!(),
        }
    }
}

trait StateBehavior {
    fn handle(&self);
}

impl StateBehavior for State {
    fn handle(&self) {
        match self {
            State::Start => run_start(),
            State::Chat => run_chat(),
        };
    }

}

pub struct FSM {
    current_state: State,
}

impl FSM {
    pub fn new() -> Self {
        FSM {
            current_state: State::Start,
        }
    }

    pub fn handle(&mut self) {
        self.current_state.handle();
    }

    pub fn next(&mut self) {
        self.current_state = self.current_state.next();
    }
}

fn run_start() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<start::Start>::default())
        }),
    )
}

fn run_chat() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<chat::Chat>::default())
        }),
    )
}