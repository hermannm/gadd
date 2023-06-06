use anyhow::{Context, Result};
use change_list::ChangeList;
use git2::Repository;
use input_handling::user_input_event_loop;
use rendering::{fullscreen::FullscreenRenderer, inline::render_inline};
use utils::get_raw_stdout;

mod change_list;
mod change_ordering;
mod input_handling;
mod rendering;
mod statuses;
mod utils;

type Stdout = std::fs::File;

fn main() -> Result<()> {
    let repository =
        Repository::discover(".").context("Failed to find Git repository at current location")?;

    let mut change_list = ChangeList::new(&repository)?;

    if change_list.changes.is_empty() {
        println!("No changes!");
        return Ok(());
    }

    let mut stdout = get_raw_stdout();

    {
        let mut renderer = FullscreenRenderer::new(&mut stdout)?;
        renderer.render(&change_list)?;
        user_input_event_loop(&mut change_list, &mut renderer)?;
    } // Drops renderer, exiting fullscreen

    change_list
        .refresh_changes()
        .context("Failed to refresh changes on exit")?;

    render_inline(&mut stdout, &change_list).context("Failed to render changes on exit")?;

    Ok(())
}
