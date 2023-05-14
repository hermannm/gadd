use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::Block,
};

use crate::Frame;

const INPUT_CONTROLS: [[&str; 2]; 5] = [
    ["[enter]", "done"],
    ["[space]", "stage change"],
    ["[r]", "unstage change"],
    ["[up]", "move up"],
    ["[down]", "move down"],
];

pub(crate) struct InputControlsWidget {
    widget: Block<'static>,
}

impl InputControlsWidget {
    pub fn new() -> InputControlsWidget {
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

        let widget = Block::default().title(Spans::from(line));

        InputControlsWidget { widget }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.widget.clone(), area);
    }
}
