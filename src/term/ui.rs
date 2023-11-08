use crate::app::{App, State};
use crate::term::frames::ScreenItem;
use crate::term::tui::Backend;
use tui::widgets::{Block, Borders};
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
};

pub type Frame<'a> = tui::Frame<'a, Backend>;

pub fn render(app: &mut App, f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(95),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        );

    let state = app.state();
    let mut idx = 0;

    let chunks = layout.split(f.size());
    let fr = app.active_main_frame();
    let list = fr.0.draw();

    if state == State::Control {
        let block = Block::default()
            .title(fr.0.title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        f.render_stateful_widget(list.block(block), chunks[idx], &mut fr.1);
    } else {
        let block = Block::default().title(fr.0.title()).borders(Borders::ALL);

        f.render_stateful_widget(list.block(block), chunks[idx], &mut fr.1);
    }

    idx += 1;

    // {
    //     let debug = crate::dump_logger!();
    //     f.render_widget(debug, chunks[idx]);
    //     idx += 1;
    // }

    let cmd = app.cmd.dump();

    if state == State::Insert {
        let block = Block::default()
            .title("Cmd")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        f.render_widget(cmd.block(block), chunks[idx]);
    } else {
        let block = Block::default().title("Cmd").borders(Borders::ALL);

        f.render_widget(cmd.block(block), chunks[idx]);
    }
}
