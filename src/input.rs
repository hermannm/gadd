use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};

use crate::{change_list::ChangeList, render};

pub(super) fn user_input_event_loop(
    stdout: &mut Stdout,
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
                    render(stdout, change_list)?;
                }
                KeyCode::Up => {
                    change_list.decrement_selected_change();
                    render(stdout, change_list)?;
                }
                KeyCode::Down => {
                    change_list.increment_selected_change();
                    render(stdout, change_list)?;
                }
                _ => continue,
            }
        }
    }

    Ok(())
}

const INPUT_CONTROLS: [[&str; 2]; 4] = [
    ["[enter]", "done"],
    ["[space]", "stage change"],
    ["[up]", "move up"],
    ["[down]", "move down"],
];

pub(super) fn render_input_controls(stdout: &mut Stdout) -> Result<()> {
    for (i, [button, description]) in INPUT_CONTROLS.into_iter().enumerate() {
        stdout.queue(SetForegroundColor(Color::Blue))?;
        stdout.write_all(button.as_bytes())?;
        stdout.queue(ResetColor)?;
        stdout.write_all(" ".as_bytes())?;
        stdout.write_all(description.as_bytes())?;

        if i < INPUT_CONTROLS.len() - 1 {
            stdout.write_all(" ".as_bytes())?;
        }
    }

    Ok(())
}
