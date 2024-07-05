use std::collections::HashSet;
use std::io::{stdout, Result};
use std::thread;
use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Borders, List, ListItem, ListState, StatefulWidget, Widget};
use ratatui::{backend::CrosstermBackend, crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
}, symbols, Terminal};
use ratatui::style::palette::tailwind::SLATE;

use strategy::Player;

use crate::block::Block;
use crate::board::Board;
use crate::strategy::{BlockPlacement, GreedyStrategy, Strategy};

mod block;
mod board;
mod strategy;

const COMPUTER_COLOR_BG: Color = SLATE.c500;
const HUMAN_COLOR_BG: Color = SLATE.c100;
const NOT_SELECTED_COLOR_BG: Color = SLATE.c800;

#[derive(Debug, Default)]
struct BoardWidget {
    board: Board,
}

#[derive(Debug, Default)]
struct PlayerWidget {
    player: Player,
}

#[derive(Debug, Default)]
struct BlockPlacementWidget {
    block_placement: Option<BlockPlacement>,
    player_id: u8,
}

fn color_from_player_id(player_id: u8) -> Color {
    match player_id {
        1 => Color::Rgb(255, 0, 0),
        2 => Color::Rgb(0, 255, 0),
        3 => Color::Rgb(0, 0, 255),
        4 => Color::Rgb(255, 255, 0),
        _ => Color::Rgb(0, 0, 0),
    }
}

impl Widget for &mut BlockPlacementWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
    {
        if let Some(block_placement) = &self.block_placement {
            let (board_row, board_col, block) = block_placement.as_row_col_block();
            let top_left_x = area.left() + (board_col * 2) as u16;
            let top_left_y = area.top() + board_row as u16;
            for (xi, x) in (top_left_x..(top_left_x + (block.ncols() * 2) as u16)).enumerate() {
                for (yi, y) in (top_left_y..(top_left_y + block.nrows() as u16)).enumerate() {
                    let block_col = xi / 2;
                    let block_row = yi;
                    if block.cell_at_row_col(block_row, block_col) {
                        let color = color_from_player_id(self.player_id);
                        buf.get_mut(x, y).set_char('█').set_fg(color);
                    }
                }
            }
        }
    }
}

impl Widget for &mut PlayerWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
    {
        let mut dx = 0;
        let mut dy = 0;
        for block in self.player.blocks.iter() {
            let block_width_with_margin = (block.ncols() + 1) * 2;
            let block_height_with_margin = block.nrows() + 1;
            let enough_h_space =
                (area.left() + dx + (block_width_with_margin as u16)) < area.right();
            if !enough_h_space {
                // try next row
                dx = 0;
                dy += block_height_with_margin as u16;
            }
            let enough_h_space =
                (area.left() + dx + (block_width_with_margin as u16)) < area.right();
            let enough_v_space =
                (area.top() + dy + (block_height_with_margin as u16)) < area.bottom();
            if enough_h_space && enough_v_space {
                for (xi, x) in ((area.left() + dx)
                    ..(area.left() + dx + (block_width_with_margin as u16)))
                    .enumerate()
                {
                    for (yi, y) in ((area.top() + dy)
                        ..(area.top() + dy + (block_height_with_margin as u16)))
                        .enumerate()
                    {
                        let row = yi;
                        let col = xi / 2;
                        if (row < block.nrows()) && (col < block.ncols()) {
                            if block.cell_at_row_col(row, col) {
                                let color = color_from_player_id(self.player.player_id);
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
                    let color = color_from_player_id(cell_type);
                    buf.get_mut(x, y).set_char('█').set_fg(color);
                }
            }
        }
    }
}

#[derive(Default, Debug)]
enum PlayerSelectionStatus {
    #[default]
    Computer,
    Human,
    NotSelected,
}

#[derive(Default, Debug)]
struct PlayerSelectionItem {
    status: PlayerSelectionStatus,
    info: String,
}

impl From<&PlayerSelectionItem> for ListItem<'_> {
    fn from(value: &PlayerSelectionItem) -> Self {
        let line = match value.status {
            PlayerSelectionStatus::Computer => Line::styled(format!(" Computer     {}", value.info), COMPUTER_COLOR_BG),
            PlayerSelectionStatus::Human => Line::styled(format!(" Human        {}", value.info), HUMAN_COLOR_BG),
            PlayerSelectionStatus::NotSelected => Line::styled(format!(" Not selected {}", value.info), NOT_SELECTED_COLOR_BG),
        };
        ListItem::new(line)
    }
}

#[derive(Debug)]
struct PlayerSelectionList {
    items: Vec<PlayerSelectionItem>,
    state: ListState,
}

impl Default for PlayerSelectionList {
    fn default() -> Self {
        let items = vec![
            PlayerSelectionItem { status: PlayerSelectionStatus::Computer, info: "Player 1".to_string() },
            PlayerSelectionItem { status: PlayerSelectionStatus::Computer, info: "Player 2".to_string() },
            PlayerSelectionItem { status: PlayerSelectionStatus::Computer, info: "Player 3".to_string() },
            PlayerSelectionItem { status: PlayerSelectionStatus::Computer, info: "Player 4".to_string() },
        ];
        PlayerSelectionList {
            items,
            state: ListState::default(),
        }
    }
}

#[derive(Default, Debug)]
enum GameState {
    #[default]
    MainMenu,
    Game,
}

#[derive(Debug, Default)]
struct App {
    board_widget: BoardWidget,
    player_widget: PlayerWidget,
    block_placement_widget: BlockPlacementWidget,
    game_state: GameState,
    player_selection_list: PlayerSelectionList,
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.game_state {
            GameState::MainMenu => {
                let block = ratatui::widgets::Block::new()
                    .title(Line::raw("Player Selection").centered())
                    .borders(Borders::TOP)
                    .border_set(symbols::border::EMPTY);

                let items: Vec<ListItem> = self.player_selection_list.items.iter().map(|item| ListItem::from(item)).collect();
                let list = List::new(items).block(block).highlight_symbol(">");

                StatefulWidget::render(list, area, buf, &mut self.player_selection_list.state)
            }
            GameState::Game => {
                let [top, bottom] = Layout::vertical([Length(20), Min(0)]).areas(area);
                let [board, player] = Layout::horizontal([Length(40), Min(40)]).areas(top);
                self.board_widget.render(board, buf);
                self.block_placement_widget.render(board, buf);
                self.player_widget.render(player, buf);
                let text = if let Some(block) = &self.block_placement_widget.block_placement {
                    format!("row: {}, col: {}, q(uit) j/k (previous/next) r(otate) t(ranspose)", block.row, block.col)
                } else {
                    String::from("q(uit)")
                };
                Text::from(text).left_aligned().render(bottom, buf);
            }
        }
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = App::default();

    // main menu
    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(&mut app, area);
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Up {
                    app.player_selection_list.state.select_previous();
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Down {
                    app.player_selection_list.state.select_next();
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Left {
                    if let Some(i) = app.player_selection_list.state.selected() {
                        app.player_selection_list.items[i].status = match app.player_selection_list.items[i].status {
                            PlayerSelectionStatus::Computer => PlayerSelectionStatus::Human,
                            PlayerSelectionStatus::Human => PlayerSelectionStatus::NotSelected,
                            PlayerSelectionStatus::NotSelected => PlayerSelectionStatus::Computer,
                        }
                    }
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Right {
                    if let Some(i) = app.player_selection_list.state.selected() {
                        app.player_selection_list.items[i].status = match app.player_selection_list.items[i].status {
                            PlayerSelectionStatus::Computer => PlayerSelectionStatus::NotSelected,
                            PlayerSelectionStatus::Human => PlayerSelectionStatus::Computer,
                            PlayerSelectionStatus::NotSelected => PlayerSelectionStatus::Human,
                        }
                    }
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    app.game_state = GameState::Game;

    let mut players: Vec<Player> = app.player_selection_list.items.iter().enumerate().map(
        |(player_id, player_selection)|
            match player_selection.status {
                PlayerSelectionStatus::Computer => Some(Player { player_id: (player_id + 1) as u8, human: false, blocks: Block::default_block_set() }),
                PlayerSelectionStatus::Human => Some(Player { player_id: (player_id + 1) as u8, human: true, blocks: Block::default_block_set() }),
                PlayerSelectionStatus::NotSelected => None,
            }
    ).flatten().collect();
    let players_id: Vec<u8> = players.iter().map(|p| p.player_id).collect();

    let mut board = Board::new(20, 20);

    let mut turn_counter: usize = 0;
    let mut players_eliminated = HashSet::<u8>::new();

    loop {
        app.board_widget.board = board.clone();

        for &player_id in players_id.iter() {
            if let Some(position) = players.iter().position(|p| p.player_id == player_id) {
                let player: &Player = players.get(position).unwrap();
                if players_eliminated.contains(&player_id) {
                    continue;
                }
                app.player_widget.player = player.clone();
                app.block_placement_widget.player_id = player.player_id;
                let block_placement: Option<BlockPlacement> = if player.human {
                    if let Some(first_block) = player.blocks.first() {
                        let mut block_selection: usize = 0;
                        let mut player_block_placement = Some(BlockPlacement {
                            block: first_block.clone(),
                            row: 0,
                            col: 0,
                            rotation: 0,
                            transposition: 0,
                        });
                        loop {
                            app.block_placement_widget.block_placement =
                                player_block_placement.clone();
                            if event::poll(Duration::from_millis(16))? {
                                if let event::Event::Key(key) = event::read()? {
                                    if key.kind == KeyEventKind::Press
                                        && key.code == KeyCode::Char('q')
                                    {
                                        player_block_placement = None;
                                        break;
                                    }
                                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Left
                                    {
                                        if let Some(block) = &mut player_block_placement {
                                            block.col = (block.col as i32 - 1).max(0) as usize;
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Right
                                    {
                                        if let Some(block) = &mut player_block_placement {
                                            block.col = (block.col + 1)
                                                .min(app.board_widget.board.ncols() - 1);
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Up {
                                        if let Some(block) = &mut player_block_placement {
                                            block.row = (block.row as i32 - 1).max(0) as usize;
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Down
                                    {
                                        if let Some(block) = &mut player_block_placement {
                                            block.row = (block.row + 1)
                                                .min(app.board_widget.board.nrows() - 1);
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press
                                        && key.code == KeyCode::Char('j')
                                    {
                                        if let Some(block_placement) = &mut player_block_placement {
                                            block_selection =
                                                (block_selection + player.blocks.len() - 1)
                                                    % player.blocks.len();
                                            if let Some(block) = player.blocks.get(block_selection)
                                            {
                                                block_placement.block = block.clone();
                                            }
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press
                                        && key.code == KeyCode::Char('k')
                                    {
                                        if let Some(block_placement) = &mut player_block_placement {
                                            block_selection =
                                                (block_selection + 1) % player.blocks.len();
                                            if let Some(block) = player.blocks.get(block_selection)
                                            {
                                                block_placement.block = block.clone();
                                            }
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press
                                        && key.code == KeyCode::Char('t')
                                    {
                                        if let Some(block_placement) = &mut player_block_placement {
                                            block_placement.transposition =
                                                (block_placement.transposition + 1) % 2;
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press
                                        && key.code == KeyCode::Char('r')
                                    {
                                        if let Some(block_placement) = &mut player_block_placement {
                                            block_placement.rotation =
                                                (block_placement.rotation + 1) % 4;
                                        }
                                    }
                                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter
                                    {
                                        if let Some(block_placement) = &mut player_block_placement {
                                            let (row, col, block) =
                                                block_placement.as_row_col_block();
                                            let placement_rule = app.board_widget.board.can_place(
                                                row,
                                                col,
                                                &block,
                                                player.player_id,
                                                turn_counter == 0,
                                            );
                                            if placement_rule.placement_ok() {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            terminal.draw(|frame| {
                                let area = frame.size();
                                frame.render_widget(&mut app, area);
                            })?;
                        }
                        player_block_placement
                    } else {
                        None
                    }
                } else {
                    GreedyStrategy::place(&board, player.player_id, players.as_slice(), turn_counter == 0)
                };

                let player: &mut Player = players.get_mut(position).unwrap();
                if let Some(block_placement) = block_placement {
                    // remove block from player blocks
                    let block_index_to_remove =
                        player.blocks.iter().position(|b| *b == block_placement.block);
                    if let Some(index) = block_index_to_remove {
                        player.blocks.remove(index);
                    }

                    let (row, col, block) = block_placement.as_row_col_block();
                    board.place(row, col, &block, player_id);
                } else {
                    players_eliminated.insert(player_id);
                }
            }

            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(&mut app, area);
            })?;
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
