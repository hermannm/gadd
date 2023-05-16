use anyhow::{Context, Result};
use git2::Repository;
use input_handling::user_input_event_loop;
use rendering::{fullscreen::FullscreenRenderer, inline::render_inline};

use crate::change_list::ChangeList;

mod change_list;
mod change_ordering;
mod input_handling;
mod rendering;
mod statuses;
mod utils;

fn main() -> Result<()> {
    let repository = Repository::discover(".").context("Failed to open repository")?;
    let mut change_list = ChangeList::new(&repository)?;

    if change_list.changes.is_empty() {
        println!("No changes!");
        return Ok(());
    }

    {
        let mut renderer = FullscreenRenderer::new()?;
        renderer.render(&mut change_list)?;
        user_input_event_loop(&mut change_list, &mut renderer)?;
    } // drops renderer, exiting fullscreen

    change_list.refresh_changes()?;
    render_inline(&change_list)?;

    Ok(())
}
