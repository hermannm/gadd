use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::{changes::ChangeList, rendering::FullscreenRenderer};

pub(crate) fn user_input_event_loop(
    change_list: &mut ChangeList,
    renderer: &mut FullscreenRenderer,
) -> Result<()> {
    #[cfg(windows)]
    handle_initial_enter_press_windows()
        .context("Failed to read initial ENTER press (Windows-specific)")?;

    loop {
        let event = event::read().context("Failed to read user input")?;

        let Event::Key(event) = event else {
            continue;
        };

        if event.kind != KeyEventKind::Press {
            continue;
        }

        use KeyCode::*;
        match (event.code, event.modifiers) {
            (Enter, _) | (Esc, _) | (Char('c'), KeyModifiers::CONTROL) => {
                break;
            }
            (Up, _) => {
                change_list.select_previous_change();
                renderer.render(change_list)?;
            }
            (Down, _) => {
                change_list.select_next_change();
                renderer.render(change_list)?;
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
            (Char('a'), _) => {
                change_list
                    .stage_all_changes()
                    .context("Failed to stage all changes")?;

                renderer.render(change_list)?;
            }
            (Char('u'), _) => {
                change_list
                    .unstage_all_changes()
                    .context("Failed to stage all changes")?;

                renderer.render(change_list)?;
            }
            (Char('c'), _) => {
                change_list
                    .copy_path_of_selected_change()
                    .context("Failed to copy path of selected change")?;
            }
            _ => {
                continue;
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
