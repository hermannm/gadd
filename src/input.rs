use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::Block,
};

use crate::{change_list::ChangeList, render, Terminal};

pub(super) fn user_input_event_loop(
    terminal: &mut Terminal,
    change_list: &mut ChangeList,
) -> Result<()> {
    loop {
        let event = event::read()?;

        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char(' ') => {
                    change_list.stage_selected_change()?;
                    render(terminal, change_list)?;
                }
                KeyCode::Char('r') => {
                    change_list.unstage_selected_change()?;
                    render(terminal, change_list)?;
                }
                KeyCode::Up => {
                    change_list.increment_selected_change();
                    render(terminal, change_list)?;
                }
                KeyCode::Down => {
                    change_list.decrement_selected_change();
                    render(terminal, change_list)?;
                }
                _ => continue,
            }
        }
    }

    Ok(())
}

const INPUT_CONTROLS: [[&str; 2]; 5] = [
    ["[enter]", "done"],
    ["[space]", "stage change"],
    ["[r]", "unstage change"],
    ["[up]", "move up"],
    ["[down]", "move down"],
];

pub(super) fn render_input_controls() -> Block<'static> {
    let blue_text = Style::default().fg(Color::Blue);

    let mut line = Vec::<Span>::with_capacity(3 * INPUT_CONTROLS.len() + 1);

    for (i, [button, description]) in INPUT_CONTROLS.into_iter().enumerate() {
        line.push(Span::styled(button, blue_text));
        line.push(Span::raw(" "));
        line.push(Span::raw(description));

        if i < INPUT_CONTROLS.len() - 1 {
            line.push(Span::raw(" "));
        }
    }

    Block::default().title(Spans::from(line))
}
