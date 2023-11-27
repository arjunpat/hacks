use core::fmt;
use std::fmt::{Display, Formatter};

use rand::{random, seq::SliceRandom};

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    board: [u32; 16],
    move_num: u32,
    score: u32,
    game_over: bool,
}

impl Board {
    pub fn new() -> Self {
        let mut board = [0; 16];

        let mut numbers: Vec<usize> = (0..16).collect();
        assert!(numbers.len() == 16);
        numbers.shuffle(&mut rand::thread_rng());

        // place two random numbers on the board
        for i in 0..2 {
            if random::<f64>() > 0.8 {
                board[numbers[i]] = 4
            } else {
                board[numbers[i]] = 2;
            }
        }

        Self {
            board,
            move_num: 0,
            score: 0,
            game_over: false,
        }
    }

    // returns boolean tuple (board_changed, game_over)
    pub fn make_move(&mut self, dir: Move) -> (bool, bool) {
        self.move_num += 1;

        let (board_changed, score_increase) = Self::_make_move(&mut self.board, dir);
        self.score += score_increase;

        // add a random 2 or 4 to the board
        let zero_pos: Vec<usize> = (0..16 as usize).filter(|i| self.board[*i] == 0).collect();
        if zero_pos.len() != 0 && board_changed {
            let random_zero = zero_pos.choose(&mut rand::thread_rng()).unwrap();

            if random::<f64>() > 0.8 {
                self.board[*random_zero] = 4;
            } else {
                self.board[*random_zero] = 2;
            }
        }

        if zero_pos.len() == 1 && board_changed {
            // just placed last square; check if game over
            self.game_over = self.is_game_over();
        }

        (board_changed, self.game_over)
    }

    fn _make_move(board: &mut [u32; 16], dir: Move) -> (bool, u32) {
        let results: Vec<(bool, u32)> = (0..4)
            .map(|i| Self::handle_row(board, Self::make_idx_func(dir, i)))
            .collect();

        (
            results.iter().any(|e| e.0),
            results.iter().map(|e| e.1).sum(),
        )
    }

    fn make_idx_func(dir: Move, i: usize) -> impl Fn(usize) -> usize {
        assert!(i < 4);

        let func = move |j: usize| -> usize {
            assert!(j < 4);
            match dir {
                Move::Up => 4 * (3 - j) + i,
                Move::Down => 4 * j + i,
                Move::Left => 4 * i + (3 - j),
                Move::Right => 4 * i + j,
            }
        };
        func
    }

    fn is_game_over(&self) -> bool {
        let zero_pos: Vec<usize> = (0..16 as usize).filter(|e| self.board[*e] == 0).collect();

        if zero_pos.len() > 0 {
            return false;
        }

        let mut board_copy: [u32; 16] = [0; 16];
        for each in [Move::Up, Move::Down, Move::Left, Move::Right] {
            board_copy.copy_from_slice(&self.board);
            let (board_changed, _) = Self::_make_move(&mut board_copy, each);

            if board_changed {
                return false;
            }
        }

        true
    }

    // returns (board_changed, score_increase) from operating on this specific row
    fn handle_row(board: &mut [u32; 16], idx_func: impl Fn(usize) -> usize) -> (bool, u32) {
        let idx: Vec<usize> = (0..4).map(idx_func).collect();

        let row_sum: u32 = idx.iter().map(|e| board[*e]).sum();
        if row_sum == 0 {
            return (false, 0);
        }
        let mut board_changed = false;

        // move everything to the right
        let mut shift_right = 0;
        for i in (0..4).rev() {
            if board[idx[i]] == 0 {
                shift_right += 1;
            } else if shift_right != 0 {
                board_changed = true;
                board[idx[i + shift_right]] = board[idx[i]];
                board[idx[i]] = 0;
            }
        }

        let mut score_increase = 0;
        for i in (1..4).rev() {
            if board[idx[i]] == 0 {
                break;
            } else if board[idx[i]] == board[idx[i - 1]] {
                board[idx[i]] *= 2;
                score_increase += board[idx[i]];
                board[idx[i - 1]] = 0;
                for j in (0..(i - 1)).rev() {
                    if board[idx[j]] != 0 {
                        board[idx[j + 1]] = board[idx[j]];
                        board[idx[j]] = 0;
                    }
                }
            }
        }

        assert!(row_sum == idx.iter().map(|e| board[*e]).sum());

        let num_zeros = idx.iter().filter(|e| board[**e] == 0).count();

        for i in 0..num_zeros {
            assert!(board[idx[i]] == 0);
        }

        (board_changed || score_increase != 0, score_increase)
    }

    pub fn get_max(&self) -> u32 {
        *self.board.iter().max().unwrap()
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn num_moves(&self) -> u32 {
        self.move_num
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "--- Board (Move: {}; Score: {}) ---\n",
            self.move_num, self.score
        )?;

        for i in 0..4 {
            for j in 0..4 {
                write!(f, "{:4}", self.board[4 * i + j])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
