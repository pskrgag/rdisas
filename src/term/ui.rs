use crate::app::{App, State};
use crate::term::frames::ScreenItem;
use crate::term::tui::Backend;
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
                Constraint::Percentage(80),
                Constraint::Percentage(15),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        );

    let chunks = layout.split(f.size());
    let fr = app.active_main_frame();
    let list = fr.0.draw();

    f.render_stateful_widget(list, chunks[0], &mut fr.1);

    let debug = crate::dump_logger!();
    f.render_widget(debug, chunks[1]);

    let cmd = app.cmd.dump();

    if app.state() == State::Control {
        f.render_widget(cmd.style(Style::default().fg(Color::Blue)), chunks[2]);
    } else {
        f.render_widget(cmd, chunks[2]);
    }
}
