use anyhow::{Context, Error, Result};
use crossbeam_channel::{Receiver, Sender};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use git2::Repository;
use std::{process::Command, thread};

use crate::{
    changes::{
        branches::{LocalBranch, UpstreamBranch},
        change_list::{ChangeList, FetchStatus, UpstreamCommitsDiff},
    },
    rendering::fullscreen::FullscreenRenderer,
};

enum Event {
    UserInput(KeyEvent),
    UserInputError(Error),
    FetchComplete(UpstreamCommitsDiff),
    FetchError(Error),
}

enum Signal {
    Continue,
    Stop,
}

pub(crate) fn user_input_event_loop(
    change_list: &mut ChangeList,
    renderer: &mut FullscreenRenderer,
) -> Result<()> {
    #[cfg(windows)]
    handle_initial_enter_press_windows()
        .context("Failed to read initial ENTER press (Windows-specific)")?;

    let (event_sender, event_receiver) = crossbeam_channel::unbounded::<Event>();
    let (input_signal_sender, input_signal_receiver) = crossbeam_channel::unbounded::<Signal>();
    let (fetch_signal_sender, fetch_signal_receiver) = crossbeam_channel::unbounded::<Signal>();

    thread::scope(|scope| -> Result<()> {
        scope.spawn(|| run_input_worker(event_sender.clone(), input_signal_receiver));

        let current_branch = change_list.current_branch.clone();
        let upstream = change_list.upstream.clone();
        scope.spawn(|| {
            run_fetch_worker(
                current_branch,
                upstream,
                event_sender.clone(),
                fetch_signal_receiver,
            )
        });

        let stop_input_worker = || {
            let _ = input_signal_sender.send(Signal::Stop);
        };
        let stop_fetch_worker = || {
            let _ = fetch_signal_sender.send(Signal::Stop);
        };

        loop {
            let event = event_receiver.recv()?;
            match event {
                Event::UserInput(event) => {
                    match handle_user_input(
                        event,
                        change_list,
                        renderer,
                        fetch_signal_sender.clone(),
                    ) {
                        Ok(Signal::Continue) => {
                            input_signal_sender.must_send(Signal::Continue);
                            continue;
                        }
                        Ok(Signal::Stop) => {
                            stop_input_worker();
                            stop_fetch_worker();
                            return Ok(());
                        }
                        Err(err) => {
                            stop_input_worker();
                            stop_fetch_worker();
                            return Err(err);
                        }
                    }
                }
                Event::UserInputError(err) => {
                    stop_fetch_worker(); // input worker will already have stopped on error
                    return Err(err);
                }
                Event::FetchComplete(upstream_diff) => {
                    change_list.fetch_status = FetchStatus::Fetched(upstream_diff);
                    renderer.render(change_list)?;
                }
                Event::FetchError(err) => {
                    change_list.fetch_status = FetchStatus::FetchFailed(err);
                }
            }
        }
    })
}

fn handle_user_input(
    event: KeyEvent,
    change_list: &mut ChangeList,
    renderer: &mut FullscreenRenderer,
    fetch_signal_sender: Sender<Signal>,
) -> Result<Signal> {
    use KeyCode::*;
    match (event.code, event.modifiers) {
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
                .context("Failed to unstage all changes")?;

            renderer.render(change_list)?;
        }
        (Char('f'), _) => {
            fetch_signal_sender
                .send(Signal::Continue)
                .context("Failed to reach worker thread to refetch upstream changes")?;
        }
        (Enter, _) => {
            renderer
                .exit_fullscreen()
                .context("Failed to reset terminal state before entering commit")?;
            Command::new("git")
                .arg("commit")
                .status()
                .context("Failed to run 'git commit'")?;
            return Ok(Signal::Stop);
        }
        (Esc, _) | (Char('c'), KeyModifiers::CONTROL) => {
            return Ok(Signal::Stop);
        }
        _ => {}
    }
    Ok(Signal::Continue)
}

fn run_input_worker(event_sender: Sender<Event>, signal_receiver: Receiver<Signal>) {
    let send_err = |err: Error| event_sender.must_send(Event::UserInputError(err));

    loop {
        let Ok(user_input) = event::read()
            .context("Failed to read user input")
            .map_err(send_err)
        else {
            return;
        };

        let event::Event::Key(user_input) = user_input else {
            continue;
        };

        if user_input.kind != KeyEventKind::Press {
            continue;
        }

        event_sender.must_send(Event::UserInput(user_input));

        match signal_receiver.must_recv() {
            Signal::Continue => continue,
            Signal::Stop => break,
        }
    }
}

fn run_fetch_worker(
    current_branch: LocalBranch,
    upstream: Option<UpstreamBranch>,
    event_sender: Sender<Event>,
    signal_receiver: Receiver<Signal>,
) {
    let send_err = |err: Error| event_sender.must_send(Event::FetchError(err));

    let Ok(repo) = Repository::discover(".")
        .context("Failed to find Git repository at current location")
        .map_err(send_err)
    else {
        return;
    };

    let mut upstream_with_remote = if let Some(upstream) = upstream {
        let Ok(remote) = repo
            .find_remote(&upstream.remote_name)
            .with_context(|| format!("Failed to find remote with name '{}'", upstream.remote_name))
            .map_err(send_err)
        else {
            return;
        };
        Some((upstream, remote))
    } else {
        None
    };

    loop {
        if let Some((upstream, remote)) = &mut upstream_with_remote {
            if let Ok(()) = remote
                .fetch(&[&upstream.name], None, None)
                .with_context(|| format!("Failed to fetch upstream '{}'", upstream.full_name))
                .map_err(send_err)
            {
                if let Ok((ahead, behind)) = repo
                    .graph_ahead_behind(current_branch.object_id, upstream.object_id)
                    .context("Failed to get commits ahead/behind upstream")
                    .map_err(send_err)
                {
                    event_sender
                        .must_send(Event::FetchComplete(UpstreamCommitsDiff { ahead, behind }))
                }
            };
        }

        match signal_receiver.must_recv() {
            Signal::Continue => continue,
            Signal::Stop => break,
        }
    }
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

trait MustSend<T> {
    fn must_send(&self, msg: T);
}

impl<T> MustSend<T> for Sender<T> {
    fn must_send(&self, msg: T) {
        self.send(msg)
            .expect("Thread communication failure: Channel disconnected");
    }
}

trait MustReceive<T> {
    fn must_recv(&self) -> T;
}

impl<T> MustReceive<T> for Receiver<T> {
    fn must_recv(&self) -> T {
        self.recv()
            .expect("Thread communication failure: Channel disconnected")
    }
}
