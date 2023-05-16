use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{change_list::ChangeList, rendering::fullscreen::FullscreenRenderer};

pub(crate) fn user_input_event_loop(
    change_list: &mut ChangeList,
    renderer: &mut FullscreenRenderer,
) -> Result<()> {
    #[cfg(windows)]
    handle_initial_enter_press_windows()
        .context("Failed to read initial ENTER press (Windows-specific)")?;

    loop {
        let event = event::read().context("Failed to read user input")?;

        if let Event::Key(event) = event {
            use KeyCode::*;

            match (event.code, event.modifiers) {
                (Enter, _) | (Esc, _) | (Char('c'), KeyModifiers::CONTROL) => {
                    break;
                }
                (Char(' '), _) => {
                    change_list
                        .stage_selected_change()
                        .context("Failed to stage selected change")?;

                    renderer.render(change_list)?;
                }
                (Char('r'), _) => {
                    change_list
                        .unstage_selected_change()
                        .context("Failed to unstage selected change")?;

                    renderer.render(change_list)?;
                }
                (Up, _) => {
                    change_list.increment_selected_change();
                    renderer.render(change_list)?;
                }
                (Down, _) => {
                    change_list.decrement_selected_change();
                    renderer.render(change_list)?;
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
        let event = event::read().context("Failed to read user input")?;

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
