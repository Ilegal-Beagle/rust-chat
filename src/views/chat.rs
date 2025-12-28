use crate::App;

impl App {
    // rendering the chat state along with its UI components
    pub fn render_chat(&mut self, ctx: &egui::Context) {
        
        // recieve message network side
        if let Some(net) = &mut self.network.client {
            match net.recv() {
                Some(msg) => self.io.messages.push(msg),
                None => {},
            }
        }

        // render ui
        self.message_panel(ctx);

        self.side_panel(ctx);

        self.chat_panel(ctx);
    }
}