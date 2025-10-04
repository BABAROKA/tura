use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use std::sync::mpsc::Receiver;
use std::{thread, time::Duration};

pub enum Action {
    Quit,
    Downloading,
    Playing(String),
}

#[derive(Debug, Default)]
pub struct App {
    action: String,
    exit: bool,
}

pub fn init(rx: Receiver<Action>) {
    color_eyre::install().unwrap();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal, rx);
    ratatui::restore();
    app_result.unwrap();
}

impl App {
    fn exit(&mut self) {
        self.exit = true;
    }

    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: Receiver<Action>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            thread::sleep(Duration::from_millis(33));

            if let Ok(action) = rx.try_recv() {
                match action {
                    Action::Quit => self.exit(),
                    Action::Playing(title) => self.action = format!(" {} ", title),
                    Action::Downloading => self.action = format!("Downloading..."),
                }
            }

            self.handle_events();
        }
        Ok(())
    }

    fn handle_events(&mut self) {
        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    self.handle_key_event(event)
                }
                _ => {}
            }
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Char('q') => {
                self.exit();
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" {} ", self.action))
            .bold()
            .centered()
            .white();
        let _title_block = Block::bordered()
            .title(title)
            .border_set(border::THICK)
            .blue()
            .render(area, buf);
    }
}
