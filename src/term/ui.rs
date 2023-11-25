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
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());
    let upper_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

    let state = app.state();

    let chunks = layout.split(f.size());
    let chunks_bottom = bottom_layout.split(*chunks.last().unwrap());

    let fr = app.active_main_frame();
    let second_frame = fr.0.second_frame();
    let list = fr.0.draw();

    if state == State::Control {
        let block = Block::default()
            .title(fr.0.title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if let Some(s) = second_frame {
            let split = upper_layout.split(chunks[0]);

            f.render_stateful_widget(list.block(block), split[0], &mut fr.1);

            let block = Block::default()
                .title("test")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue));
            f.render_widget(s.block(block), split[1]);
        } else {
            f.render_stateful_widget(list.block(block), chunks[0], &mut fr.1);
        }
    } else {
        let block = Block::default().title(fr.0.title()).borders(Borders::ALL);

        if let Some(s) = second_frame {
            let split = upper_layout.split(chunks[0]);

            f.render_stateful_widget(list.block(block), split[0], &mut fr.1);
            f.render_widget(s, split[1]);
        } else {
            f.render_stateful_widget(list.block(block), chunks[0], &mut fr.1);
        }
    }

    {
        let debug = crate::dump_logger!();
        f.render_widget(debug, chunks_bottom[1]);
    }

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

        f.render_widget(cmd.block(block), chunks_bottom[0]);
    } else {
        let block = Block::default().title("Find").borders(Borders::ALL);

        f.render_widget(cmd.block(block), chunks_bottom[0]);
    }
}
