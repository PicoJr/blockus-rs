use crate::block::Block;
use nalgebra::DMatrix;

type CellType = u8;
const FREE_CELL: CellType = 0;

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Board {
    data: DMatrix<CellType>,
}

#[derive(Debug)]
pub(crate) struct PlacementRule {
    overlapping: Option<bool>,
    own_block_touching_sides: Option<bool>,
    no_corner: Option<bool>,
}

impl PlacementRule {
    pub fn placement_ok(&self) -> bool {
        matches!(
            (
                &self.overlapping,
                &self.own_block_touching_sides,
                &self.no_corner,
            ),
            (Some(false), Some(false), Some(false))
        )
    }
}

pub(crate) struct BruteForceSearchPlace {
    block: Block,
    block_type: CellType,
    first_block: bool,
    board: Board,
    start: usize,
}

#[derive(Debug)]
pub(crate) struct BlockPosition {
    pub row: usize,
    pub col: usize,
    pub rotation: u8,
    pub transposition: u8,
}

impl Iterator for BruteForceSearchPlace {
    type Item = BlockPosition;

    fn next(&mut self) -> Option<Self::Item> {
        let iterations = 2usize * 4usize * self.board.nrows() * self.board.ncols();
        for i in self.start..iterations {
            let col: usize = i % self.board.ncols();
            let row: usize = (i / self.board.ncols()) % self.board.nrows();
            let rotation: u8 = ((i / (self.board.ncols() * self.board.nrows())) % 4) as u8;
            let transposition: u8 = ((i / (self.board.ncols() * self.board.nrows() * 4)) % 2) as u8;

            let block = if transposition == 0 {
                &self.block
            } else {
                &self.block.transpose()
            };
            let block = match rotation {
                1 => &block.rotate_90(),
                2 => &block.rotate_90().rotate_90(),
                3 => &block.rotate_90().rotate_90().rotate_90(),
                _ => block,
            };
            let placement_rule =
                self.board
                    .can_place(row, col, block, self.block_type, self.first_block);

            if placement_rule.placement_ok() {
                self.start = i + 1;
                return Some(BlockPosition {
                    row,
                    col,
                    rotation,
                    transposition,
                });
            }
        }
        None
    }
}

impl Board {
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Board {
            data: DMatrix::from_element(nrows, ncols, FREE_CELL),
        }
    }

    pub fn nrows(&self) -> usize {
        self.data.nrows()
    }

    pub fn ncols(&self) -> usize {
        self.data.ncols()
    }
    pub fn free_at_row_col(&self, row: usize, col: usize) -> bool {
        if row < self.data.nrows() && col < self.data.ncols() {
            self.data[(row, col)] == FREE_CELL
        } else {
            false
        }
    }
    pub fn at_row_col(&self, row: usize, col: usize) -> CellType {
        if row < self.data.nrows() && col < self.data.ncols() {
            self.data[(row, col)]
        } else {
            FREE_CELL
        }
    }

    pub fn can_place(
        &self,
        row: usize,
        col: usize,
        block: &Block,
        block_type: CellType,
        first_block: bool,
    ) -> PlacementRule {
        let mut placement_rule = PlacementRule {
            overlapping: None,
            own_block_touching_sides: None,
            no_corner: None,
        };
        // check block does not overlap existing non-empty board cells
        for block_row in 0..block.nrows() {
            for block_col in 0..block.ncols() {
                let board_row = row + block_row;
                let board_col = col + block_col;
                let block_cell = block.cell_at_row_col(block_row, block_col);
                let board_cell_free = self.free_at_row_col(board_row, board_col);
                if block_cell && !board_cell_free {
                    placement_rule.overlapping = Some(true);
                    break;
                }
            }
        }
        if placement_rule.overlapping.is_some() {
            return placement_rule;
        }
        placement_rule.overlapping = Some(false);

        // check block neighbor cells are not the same cell type as the block
        for block_row in 0..block.nrows() {
            for block_col in 0..block.ncols() {
                let block_cell = block.cell_at_row_col(block_row, block_col);
                if block_cell {
                    for (drow, dcol) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let board_row = row + block_row;
                        let board_col = col + block_col;
                        if (board_row as i32 + drow >= 0)
                            && (board_col as i32 + dcol >= 0)
                            && (board_row as i32 + drow < self.data.nrows() as i32)
                            && (board_col as i32 + dcol < self.data.ncols() as i32)
                        {
                            let board_row = ((board_row as i32) + drow) as usize;
                            let board_col = ((board_col as i32) + dcol) as usize;
                            let board_cell = self.data[(board_row, board_col)];
                            if board_cell == block_type {
                                placement_rule.own_block_touching_sides = Some(true);
                                break;
                            }
                        }
                    }
                }
            }
        }
        if placement_rule.own_block_touching_sides.is_some() {
            return placement_rule;
        }
        placement_rule.own_block_touching_sides = Some(false);

        if first_block {
            // check block fills a corner and the corner is empty
            for (corner_row, corner_col) in [
                (0, 0),
                (0, self.data.ncols() - 1),
                (self.data.nrows() - 1, 0),
                (self.data.nrows() - 1, self.data.ncols() - 1),
            ] {
                if !self.free_at_row_col(corner_row, corner_col) {
                    continue; // corner already taken
                }
                let relative_row = corner_row as i32 - row as i32;
                let relative_col = corner_col as i32 - col as i32;
                if (relative_row >= 0)
                    && (relative_col >= 0)
                    && (relative_row < block.nrows() as i32)
                    && (relative_col < block.ncols() as i32)
                {
                    let block_row = relative_row as usize;
                    let block_col = relative_col as usize;
                    let block_cell = block.cell_at_row_col(block_row, block_col);
                    if block_cell {
                        placement_rule.no_corner = Some(false);
                        break;
                    }
                }
            }
            if placement_rule.no_corner.is_some() {
                return placement_rule;
            }
            placement_rule.no_corner = Some(true);
        } else {
            // check at least a corner with the same cell type as the block
            for block_row in 0..block.nrows() {
                for block_col in 0..block.ncols() {
                    let block_cell = block.cell_at_row_col(block_row, block_col);
                    if block_cell {
                        for (drow, dcol) in [(-1, -1), (1, 1), (1, -1), (-1, 1)] {
                            let board_row = row + block_row;
                            let board_col = col + block_col;
                            if (board_row as i32 + drow >= 0)
                                && (board_col as i32 + dcol >= 0)
                                && (board_row as i32 + drow < self.data.nrows() as i32)
                                && (board_col as i32 + dcol < self.data.ncols() as i32)
                            {
                                let board_row = ((board_row as i32) + drow) as usize;
                                let board_col = ((board_col as i32) + dcol) as usize;
                                let board_cell = self.data[(board_row, board_col)];
                                if board_cell == block_type {
                                    placement_rule.no_corner = Some(false);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if placement_rule.no_corner.is_some() {
                return placement_rule;
            }
            placement_rule.no_corner = Some(true);
        }

        placement_rule
    }

    pub fn place(&mut self, row: usize, col: usize, block: &Block, block_type: CellType) {
        for block_row in 0..block.nrows() {
            for block_col in 0..block.ncols() {
                let board_row = row + block_row;
                let board_col = col + block_col;
                let block_cell = block.cell_at_row_col(block_row, block_col);
                if block_cell {
                    self.data[(board_row, board_col)] = block_type;
                }
            }
        }
    }

    pub fn bruteforce_search_place(
        &self,
        block: &Block,
        block_type: CellType,
        first_block: bool,
    ) -> BruteForceSearchPlace {
        BruteForceSearchPlace {
            block: block.clone(),
            block_type,
            first_block,
            board: self.clone(),
            start: 0,
        }
    }
}
