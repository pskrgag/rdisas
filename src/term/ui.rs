use crate::app::{App, State};
use crate::term::frames::ScreenItem;
use crate::term::tui::Backend;
use tui::layout::Margin;
use tui::text::*;
use tui::widgets::{Block, Borders, Paragraph};
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
};

pub type Frame<'a> = tui::Frame<'a, Backend>;

pub fn render(app: &mut App, f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref());

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

    if app.help_requested {
        let help = app.help();

        let help = Paragraph::new(Text::from(
            help.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().bg(Color::DarkGray));

        f.render_widget(
            help,
            f.size().inner(&Margin {
                vertical: 30,
                horizontal: 40,
            }),
        );
    }

    if state == State::Insert {
        let block = Block::default()
            .title("Find")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        f.render_widget(cmd.block(block), chunks[idx]);
    } else {
        let block = Block::default().title("Find").borders(Borders::ALL);

        f.render_widget(cmd.block(block), chunks[idx]);
    }
}
