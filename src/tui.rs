use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, poll, read};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::{border},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::sync::mpsc::Receiver;
use std::{thread, time::Duration};

pub enum Action {
    Quit,
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
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
            KeyCode::Char('q') => {
                self.exit();
            }
            KeyCode::Left => self.counter -= 1,
            KeyCode::Right => self.counter += 1,
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
