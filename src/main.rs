use std::io::{stdout, Result};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style::Stylize,
    widgets::Paragraph,
    Terminal,
};

use std::collections::HashSet;
use strategy::Player;
use crate::block::Block;
use crate::board::Board;
use crate::strategy::{GreedyStrategy, Strategy};

mod block;
mod board;
mod strategy;

fn main() -> Result<()> {

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let n_players = 2;
    let mut players: Vec<Player> = (1u8..=n_players).map(|player_id| Player{
        player_id,
        blocks: Block::default_block_set(),
    }).collect();
    let mut board = Board::new(20, 20);

    let mut turn_counter: usize = 0;
    let mut players_eliminated = HashSet::<u8>::new();




    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
                area,
            );
        })?;

        for player_id in 1u8..=n_players {
            if players_eliminated.contains(&player_id) {
                continue;
            }
            if let Some(block_placement) = GreedyStrategy::place(&board, player_id, players.as_slice(), turn_counter == 0) {
                // remove block from player blocks
                for p in players.iter_mut() {
                    if p.player_id == player_id {
                        let block_index_to_remove = p.blocks.iter().position(|b| *b == block_placement.block);
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
        if players_eliminated.len() == (n_players as usize) {
            break;
        }
        turn_counter += 1;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
