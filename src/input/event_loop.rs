use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{change_list::ChangeList, render, FullscreenTerminal};

use super::widget::InputControlsWidget;

pub(crate) fn user_input_event_loop(
    terminal: &mut FullscreenTerminal,
    change_list: &mut ChangeList,
    input_widget: &InputControlsWidget,
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
                    render(terminal, change_list, input_widget)?;
                }
                (Char('r'), _) => {
                    change_list.unstage_selected_change()?;
                    render(terminal, change_list, input_widget)?;
                }
                (Up, _) => {
                    change_list.increment_selected_change();
                    render(terminal, change_list, input_widget)?;
                }
                (Down, _) => {
                    change_list.decrement_selected_change();
                    render(terminal, change_list, input_widget)?;
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
