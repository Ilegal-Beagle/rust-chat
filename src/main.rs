mod ui;
mod network;
mod state;

fn main() -> eframe::Result<()> {
    let mut sm = state::FSM::new();
    let username = sm.handle();
    sm.next();
    // sm.handle();
    Ok(())
}