use nalgebra::DMatrix;
use thiserror::Error;
use crate::block::BlockError::{DimensionMismatch, EmptyBlock};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Block {
    data: DMatrix<bool>,
}

#[derive(Error, Debug)]
pub(crate) enum BlockError {
    #[error("Dimension mismatch")]
    DimensionMismatch,
    #[error("Empty block")]
    EmptyBlock,
}

impl Block {

    pub fn from_str(s: &str) -> Result<Self, BlockError> {
        let lines: Vec<String> = s.lines().map(String::from).collect();
        let rows: Vec<Vec<bool>> = lines.iter().map(|line| {
            line.chars().map(|c| match c {
                '#' => true,
                _ => false,
            }).collect()
        }).collect();

        // check all rows have the same number of columns
        let min_columns = rows.iter().map(|r| r.len()).min().unwrap_or(0);
        let max_columns = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        if min_columns != max_columns {
            return Err(DimensionMismatch)
        }
        if max_columns == 0 {
            return Err(EmptyBlock)
        }

        let rows_flattened: Vec<bool> = rows.iter().flatten().cloned().collect();

        Ok(
            Block {
                data: DMatrix::from_row_iterator(
                    rows.len(),
                    max_columns,
                    rows_flattened
                )
            }
        )
    }

    pub fn printable_string(&self) -> String {
        let mut s = String::new();
        for row in self.data.row_iter() {
            let row_str: Vec<char> = row.iter().map(|v| if *v {'#'} else {'_'}).collect();
            s.push_str(String::from_iter(row_str.iter()).as_str());
            s.push_str("\n");
        }
        s
    }

    pub fn nrows(&self) -> usize {
        self.data.nrows()
    }

    pub fn ncols(&self) -> usize {
        self.data.ncols()
    }

    pub fn cells(&self) -> usize {
        self.data.iter().map(|b| if *b {1} else {0}).sum()
    }

    pub fn transpose(&self) -> Block {
        Block {
            data: self.data.transpose()
        }
    }
    pub fn rotate_90(&self) -> Block {
        let transposed = self.data.transpose();
        let rows: Vec<Vec<bool>> = transposed.row_iter().map(
            |r| {
                let reversed_row: Vec<bool> = r.iter().rev().cloned().collect();
                reversed_row
            }
        ).collect();

        let rows_flattened: Vec<bool> = rows.iter().flatten().cloned().collect();

        Block {
            data: DMatrix::from_row_iterator(
                transposed.nrows(),
                transposed.ncols(),
                rows_flattened
            )
        }
    }

    pub fn cell_at_row_col(&self, row: usize, col: usize) -> bool {
        self.data[(row, col)]
    }

    pub fn default_block_set() -> Vec<Self> {
        vec![
            // 1
            Block::from_str("#").unwrap(),
            // 2
            Block::from_str("##").unwrap(),
            // 3
            Block::from_str("###").unwrap(),
            Block::from_str("#_\n\
                                ##").unwrap(),
            // 4
            Block::from_str("####").unwrap(),
            Block::from_str("#__\n\
                                ###").unwrap(),
            Block::from_str("_#_\n\
                                ###").unwrap(),
            Block::from_str("##\n\
                                ##").unwrap(),
            Block::from_str("##_\n\
                                _##").unwrap(),
            // 5
            Block::from_str("#####").unwrap(),
            Block::from_str("#___\n\
                                ####").unwrap(),
            Block::from_str("##__\n\
                                _###").unwrap(),
            Block::from_str("##_\n\
                                ###").unwrap(),
            Block::from_str("#_#\n\
                                ###").unwrap(),
            Block::from_str("__#_\n\
                                ####").unwrap(),
            Block::from_str("#__\n\
                                ###\n\
                                #__").unwrap(),
            Block::from_str("#__\n\
                                #__\n\
                                ###").unwrap(),
            Block::from_str("#__\n\
                                ##_\n\
                                _##").unwrap(),
            Block::from_str("#__\n\
                                ###\n\
                                __#").unwrap(),
            Block::from_str("#__\n\
                                ###\n\
                                _#_").unwrap(),
            Block::from_str("_#_\n\
                                ###\n\
                                _#_").unwrap(),
        ]
    }
}

#[cfg(test)]
mod test {
    use crate::block::Block;
    use crate::board::Board;

    #[test]
    fn test_block_from_str() {

        fn test_valid_block(block_str: &str, expected_nrows: usize, expected_ncols: usize) {
            let b = Block::from_str(block_str);
            assert!(b.is_ok());
            let b = b.unwrap();
            assert_eq!(b.nrows(), expected_nrows);
            assert_eq!(b.ncols(), expected_ncols);
        }

        test_valid_block("#", 1, 1);
        test_valid_block("##", 1, 2);
        test_valid_block("###", 1, 3);
        test_valid_block("#\n#", 2, 1);
        test_valid_block("# \n##", 2, 2);
        test_valid_block(" #\n##", 2, 2);
    }

    #[test]
    fn test_rotate_9() {
        let b = Block::from_str("#  \n###");
        assert!(b.is_ok());

        let b = b.unwrap();
        assert_eq!(b, b.rotate_90().rotate_90().rotate_90().rotate_90());
        assert_eq!(b.rotate_90(), Block::from_str("##\n# \n# ").unwrap());
        assert_eq!(b.rotate_90().rotate_90(), Block::from_str("###\n  #").unwrap());
        assert_eq!(b.rotate_90().rotate_90().rotate_90(), Block::from_str(" #\n #\n##").unwrap());

    }
    #[test]
    fn test_board_place() {
        let b = Block::from_str("#  \n###");
        assert!(b.is_ok());
        let b = b.unwrap();

        let mut board = Board::new(10, 10);
        board.place(0, 0, &b, 1);
        println!("{}", board.printable_string());

        let placement_rule = board.can_place(0, 3, &b.rotate_90().rotate_90(), 1, false);
        println!("{:?}", placement_rule);
        board.place(0, 3, &b.rotate_90().rotate_90(), 1);
        println!("{}", board.printable_string());
    }

    #[test]
    fn test_default_block_set() {
        let blocks = Block::default_block_set();
        for b in blocks {
            println!("{}", b.printable_string());
        }
    }
}