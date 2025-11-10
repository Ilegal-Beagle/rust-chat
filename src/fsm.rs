#[derive(Debug)]
enum State {
    Start,
    Chat,
}

impl State {
    fn next(&self) -> State {
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
