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
use ratatui::layout::{Layout, Rect};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::style::Color;
use ratatui::text::Text;
use ratatui::widgets::Widget;

use strategy::Player;

use crate::block::Block;
use crate::board::Board;
use crate::strategy::{GreedyStrategy, Strategy};

mod block;
mod board;
mod strategy;

#[derive(Debug, Default)]
struct BoardWidget {
    board: Board,
}

#[derive(Debug, Default)]
struct PlayerWidget {
    player: Player
}

impl Widget for &mut PlayerWidget {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let mut dx = 0;
        let mut dy = 0;
        for block in self.player.blocks.iter() {
            let block_width_with_margin = (block.ncols() + 1) * 2;
            let block_height_with_margin = block.nrows() + 1;
            let enough_h_space = (area.left() + dx + (block_width_with_margin as u16)) < area.right();
            if !enough_h_space {
                // try next row
                dx = 0;
                dy += block_height_with_margin as u16;
            }
            let enough_h_space = (area.left() + dx + (block_width_with_margin as u16)) < area.right();
            let enough_v_space = (area.top() + dy + (block_height_with_margin as u16)) < area.bottom();
            if enough_h_space && enough_v_space {
                for (xi, x) in ((area.left() + dx)..(area.left() + dx + (block_width_with_margin as u16))).enumerate() {
                    for (yi, y) in ((area.top() + dy)..(area.top() + dy + (block_height_with_margin as u16))).enumerate() {
                        let row = yi;
                        let col = xi / 2;
                        if (row < block.nrows()) && (col < block.ncols()) {
                            if block.cell_at_row_col(row, col) {
                                let color = match self.player.player_id {
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
                }
            }
            dx += block_width_with_margin as u16;
        }
    }
}

impl Widget for &mut BoardWidget {
    /// Render the widget
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (xi, x) in (area.left()..area.right()).enumerate() {
            for (yi, y) in (area.top()..area.bottom()).enumerate() {
                let board_row = yi;
                let board_col = xi / 2;
                if board_col < self.board.ncols() && board_row < self.board.nrows() {
                    let cell_type = self.board.at_row_col(board_row, board_col);
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
    }
}

#[derive(Debug, Default)]
struct App {
    board_widget: BoardWidget,
    player_widget: PlayerWidget,
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] = Layout::vertical([Length(20), Min(0)]).areas(area);
        let [board, player] = Layout::horizontal([Length(40), Min(40)]).areas(top);
        self.board_widget.render(board, buf);
        self.player_widget.render(player, buf);
        // Text::from("Player").left_aligned().render(player, buf);
        Text::from("Console").left_aligned().render(bottom, buf);
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

    let mut app = App::default();

    loop {
        app.board_widget.board = board.clone();

        for player_id in 1u8..=n_players {
            if players_eliminated.contains(&player_id) {
                continue;
            }

            if let Some(player) = players.iter().find(|p| p.player_id == player_id) {
                app.player_widget.player = player.clone();
            }
            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(&mut app, area);
            })?;

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
