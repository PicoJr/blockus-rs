use std::collections::HashSet;
use std::io::{Result, stdout};
use std::thread;
use std::time::Duration;

use palette::convert::FromColorUnclamped;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        ExecutableCommand,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    style::Stylize
    ,
    Terminal,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;

use strategy::Player;

use crate::block::Block;
use crate::board::Board;
use crate::strategy::{GreedyStrategy, Strategy};

mod block;
mod board;
mod strategy;

/// A widget that displays the full range of RGB colors that can be displayed in the terminal.
///
/// This widget is animated and will change colors over time.
#[derive(Debug, Default)]
struct BoardWidget {
    /// The colors to render - should be double the height of the area as we render two rows of
    /// pixels for each row of the widget using the half block character. This is computed any time
    /// the size of the widget changes.
    colors: Vec<Vec<Color>>,

    /// the number of elapsed frames that have passed - used to animate the colors by shifting the
    /// x index by the frame number
    frame_count: usize,

    board: Board,
}

/// Widget impl for `ColorsWidget`
///
/// This is implemented on a mutable reference so that we can update the frame count and store a
/// cached version of the colors to render instead of recalculating them every frame.
impl Widget for &mut BoardWidget {
    /// Render the widget
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (xi, x) in (area.left()..area.right()).enumerate() {
            for (yi, y) in (area.top()..area.bottom()).enumerate() {
                if xi < self.board.ncols() && yi < self.board.nrows() {
                    let cell_type = self.board.at_row_col(yi, xi);
                    let color = match cell_type {
                        1 => Color::Rgb(255, 0, 0),
                        2 => Color::Rgb(0, 255, 0),
                        3 => Color::Rgb(0, 0, 255),
                        4 => Color::Rgb(255, 255, 0),
                        _ => Color::Rgb(0, 0, 0),
                    };
                    buf.get_mut(x, y).set_char('█').set_fg(color);
                }
            }
        }
        self.frame_count += 1;
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let n_players = 4;
    let mut players: Vec<Player> = (1u8..=n_players)
        .map(|player_id| Player {
            player_id,
            blocks: Block::default_block_set(),
        })
        .collect();
    let mut board = Board::new(20, 20);

    let mut turn_counter: usize = 0;
    let mut players_eliminated = HashSet::<u8>::new();

    let mut color_widget = BoardWidget::default();

    loop {
        color_widget.board = board.clone();
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(&mut color_widget, area)
        })?;

        for player_id in 1u8..=n_players {
            if players_eliminated.contains(&player_id) {
                continue;
            }
            if let Some(block_placement) =
                GreedyStrategy::place(&board, player_id, players.as_slice(), turn_counter == 0)
            {
                // remove block from player blocks
                for p in players.iter_mut() {
                    if p.player_id == player_id {
                        let block_index_to_remove =
                            p.blocks.iter().position(|b| *b == block_placement.block);
                        if let Some(index) = block_index_to_remove {
                            p.blocks.remove(index);
                        }
                    }
                }

                let (row, col, block) = block_placement.as_row_col_block();
                board.place(row, col, &block, player_id);
            } else {
                players_eliminated.insert(player_id);
            }
        }
        /*
        if players_eliminated.len() == (n_players as usize) {
            break;
        }
        */

        turn_counter += 1;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(30))
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    for player in players {
        println!(
            "player: {}. score: {}",
            player.player_id,
            player.blocks.iter().map(|b| b.cells()).sum::<usize>()
        );
    }
    Ok(())
}
