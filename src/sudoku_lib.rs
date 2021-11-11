use colored::*;
use log::{ info, error };
use sudoku::Sudoku;
use rand::{thread_rng, Rng};
use strum_macros::EnumString;

const EASY: u8 = 25;
const MEDIUM: u8 = 30;
const HARD: u8 = 35;

#[derive(Debug, EnumString, Clone)]
pub enum Difficulty {
    #[strum(ascii_case_insensitive)]
    Easy,
    #[strum(ascii_case_insensitive)]
    Medium,
    #[strum(ascii_case_insensitive)]
    Hard,
}

pub struct SudokuAvr {
    /* Hold the generated board */ 
    board: [[Cell; 9]; 9],
    /* Holds the whole solved board */
    solution: [[Cell; 9]; 9],

    dif: Difficulty,
}


#[derive(Default, Clone)]
struct Cell {
    value: u8,
    orig: bool,
}


impl SudokuAvr {
    pub fn new(diff: &Difficulty) -> Self {
        let sudoku = Sudoku::generate_unique();
        let sudoku_bytes = sudoku.to_bytes();
        
        let solution = sudoku.solve_unique().unwrap().to_bytes();

        info!("Generating Board!");

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku_bytes),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
        };

        info!("Solving Board");
        SudokuAvr::solve_board(&mut board);

        info!("Removing Cells");

        match diff {
            Difficulty::Easy => SudokuAvr::remove_cells(&mut board.board, EASY),
            Difficulty::Medium => SudokuAvr::remove_cells(&mut board.board, MEDIUM),
            Difficulty::Hard => SudokuAvr::remove_cells(&mut board.board, HARD),
        };

        return board;
    }

    fn solve_board(sud: &mut SudokuAvr) {
        for i in 0..sud.board.len() {
            for j in 0..sud.board[i].len() {
                if sud.board[i][j].orig {
                    continue;
                } else {
                    sud.board[i][j].value = sud.solution[i][j].value;
                }
            }
        }
    }

    fn remove_cells(board: &mut [[Cell; 9]; 9], no_cells: u8) {
        let mut empty = 0;
        let mut limit = 0;
        let mut rng = thread_rng();

        while limit < no_cells && !(empty == 81) {
            let i: usize = rng.gen_range(0..9) as usize; 
            let j: usize = rng.gen_range(0..9) as usize;    

            if board[i][j].orig {
                empty += 1;
                continue;
            } else if board[i][j].value == 0 {
                continue;
            } else {
                board[i][j].value = 0;
                limit += 1;
            }
        }
    }

    pub fn print_unsolved(&self) {
        print!("{}", "Printing Unsolved Board!\nDifficulty: ".green());
        match self.dif {
            Difficulty::Easy => println!("{}","EASY".blue()),
            Difficulty::Medium => println!("{}","MEDIUM".yellow()),
            Difficulty::Hard => println!("{}","HARD".red()),
        }
        
        SudokuAvr::print_board(&self.board);
    }

    pub fn print_solved(&self) {
        print!("{}", "Printing Solved Board!\nDifficulty: ".green());
        match self.dif {
            Difficulty::Easy => println!("{}","EASY".blue()),
            Difficulty::Medium => println!("{}","MEDIUM".yellow()),
            Difficulty::Hard => println!("{}","HARD".red()),
        }
        SudokuAvr::print_board(&self.solution);
    }

    fn print_board(board: &[[Cell; 9]; 9]) {
        println!("\n\t---------------------------");
        for i in 0..board.len() {
            print!("\t{} | ", i+1);
            for j in 0..board[i].len() {
                if board[i][j].value == 0 {
                    print!("_ ");
                } else {
                    print!("{} ", board[i][j].value);
                }
                if (j+1) % 3 == 0 && (j+1) != 9 {
                    print!("| ");
                }
            }
            print!("|");
            if (i+1) % 3 == 0 && (i+1) !=  9 {
                print!("\n\t===========================");
            }
            println!();
        }
        println!("\t---------------------------");
        println!("\t🤘| 1 2 3 | 4 5 6 | 7 8 9 |\n");
    }

    fn parse_board(bytes: &[u8; 81]) -> [[Cell; 9]; 9]{
        let mut board: [[Cell; 9]; 9] = Default::default();
        let mut byte = 0;

        for i in 0..board.len() {
            for j in 0..board[i].len() {
                board[i][j].value = bytes[byte];
                board[i][j].orig = if bytes[byte] != 0 { true } else { false };
                byte += 1;
            }
        }

        return board;
    }
}


