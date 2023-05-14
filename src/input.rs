use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::Block,
};

use crate::{change_list::ChangeList, render, Terminal};

pub(super) fn user_input_event_loop(
    terminal: &mut Terminal,
    change_list: &mut ChangeList,
) -> Result<()> {
    #[cfg(windows)]
    handle_initial_enter_press_windows()?;

    loop {
        let event = event::read()?;

        if let Event::Key(event) = event {
            use KeyCode::*;

            match (event.code, event.modifiers) {
                (Enter, _) | (Esc, _) | (Char('c'), KeyModifiers::CONTROL) => {
                    break;
                }
                (Char(' '), _) => {
                    change_list.stage_selected_change()?;
                    render(terminal, change_list)?;
                }
                (Char('r'), _) => {
                    change_list.unstage_selected_change()?;
                    render(terminal, change_list)?;
                }
                (Up, _) => {
                    change_list.increment_selected_change();
                    render(terminal, change_list)?;
                }
                (Down, _) => {
                    change_list.decrement_selected_change();
                    render(terminal, change_list)?;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn handle_initial_enter_press_windows() -> Result<()> {
    use crossterm::event::KeyEvent;

    loop {
        let event = event::read()?;

        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
        ) {
            return Ok(());
        }
    }
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
