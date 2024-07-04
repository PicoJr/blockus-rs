use crate::block::Block;
use crate::board::Board;

pub(crate) struct BlockPlacement {
    pub(crate) block: Block,
    row: usize,
    col: usize,
    rotation: u8,
    transposition: u8,
}

impl BlockPlacement {
    pub fn as_row_col_block(&self) -> (usize, usize, Block) {
        let block = if self.transposition == 0 {
            self.block.clone()
        } else {
            self.block.transpose()
        };
        let block = match self.rotation {
            1 => block.rotate_90(),
            2 => block.rotate_90().rotate_90(),
            3 => block.rotate_90().rotate_90().rotate_90(),
            _ => block,
        };
        (self.row, self.col, block)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Player {
    pub player_id: u8,
    pub blocks: Vec<Block>,
}

pub(crate) trait Strategy {
    fn place(
        board: &Board,
        player_id: u8,
        players: &[Player],
        first_block: bool,
    ) -> Option<BlockPlacement>;
}

pub(crate) struct GreedyStrategy {}

impl Strategy for GreedyStrategy {
    fn place(
        board: &Board,
        player_id: u8,
        players: &[Player],
        first_block: bool,
    ) -> Option<BlockPlacement> {
        let players_with_player_id: Vec<&Player> = players
            .iter()
            .filter(|p| p.player_id == player_id)
            .collect();
        if let Some(player) = players_with_player_id.first() {
            let mut player_blocks = player.blocks.clone();
            player_blocks.sort_unstable_by(|b1, b2| b1.cells().cmp(&b2.cells()).reverse());
            for block in player_blocks {
                let bruteforce_search =
                    board.bruteforce_search_place(&block, player_id, first_block);
                for possible_block_position in bruteforce_search {
                    return Some(BlockPlacement {
                        block,
                        row: possible_block_position.row,
                        col: possible_block_position.col,
                        rotation: possible_block_position.rotation,
                        transposition: possible_block_position.transposition,
                    });
                }
                // else block cannot be placed on the board
            }
            None
        } else {
            None
        }
    }
}
