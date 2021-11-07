use sudoku::Sudoku;
use colored::*;

#[derive(Default)]
pub struct SudokuAvr {
    /* Hold the generated board */ 
    board: [[Cell; 9]; 9],
    /* Holds the whole solved board */
    solution: [[Cell; 9]; 9],
}


#[derive(Default)]
struct Cell {
    value: u8,
    orig: bool,
}

impl SudokuAvr {
    pub fn solve_board(&mut self) { 
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                self.board[i][j].value = self.solution[i][j].value;
            }
        }
    }

    pub fn solve_cell(&mut self, tup: ( usize, usize, u8 )) -> bool {
        let i = tup.0;
        let j = tup.1;
        let num = tup.2;

        if self.check_cell(i, j, num) {
            self.board[i][j].value = num;
            SudokuAvr::print_diff(&self.board, i, j);
            return true;
        } else {
            return false;
        }
    }

    fn check_cell(&self, i: usize, j: usize, num: u8) -> bool {
        // check if original
        if self.board[i][j].orig {
            println!("{}", "Cannot modify original Cell!".red());
            return false;
        }

        // check box
        let x = i - i % 3;
        let y = j - j % 3;

        for k in x..x+3 {
            for l in y..y+3 {
                if self.board[k][l].value == num {
                    let msg: String = format!("[{}] {} {}", 
                            "x".red(),
                            num, 
                            "exists in the same Square!!".red());
                println!("{}", msg);
                return false;
                }
            }
        }

        // check line
        for k in 0..9 {
            if self.board[i][k].value == num {
                let msg: String = format!("[{}] {} {}", 
                            "x".red(),
                            num, 
                            "exists in the same Line!!".red());
                println!("{}", msg);
                return false;
            }
        }

        // check row
        for k in 0..9 {
            if self.board[k][j].value == num {
                let msg: String = format!("[{}] {} {}", 
                            "x".red(),
                            num, 
                            "exists in the same Row!!".red());
                println!("{}", msg);
                return false;
            }
        }
        return true;
    }

    pub fn print_unsolved(&self) {
        SudokuAvr::print_board(&self.board);
    }

    pub fn print_solved(&self) {
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

    fn print_diff(board: &[[Cell; 9]; 9], x: usize, y: usize) {
        println!("\n\t---------------------------");
        for i in 0..board.len() {
            print!("\t{} | ", i+1);
            for j in 0..board[i].len() {
                if board[i][j].value == 0 {
                    print!("_ ");
                } else if i==x && j==y{
                    let msg: String = format!("{} ", board[i][j].value);
                    print!("{}", msg.green());
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
}

fn parse_board(board: &mut [[Cell; 9]; 9], bytes: &[u8; 81]) {
    let mut byte = 0;
    for i in 0..board.len() {
        for j in 0..board[i].len() {
            board[i][j].value = bytes[byte];
            board[i][j].orig = if bytes[byte] != 0 { true } else { false };
            byte += 1;
        }
    }
}

pub fn create_board() -> SudokuAvr {
    let mut sudoku_avr = SudokuAvr::default();

    let sudoku = Sudoku::generate_unique();
    let sudoku_bytes = sudoku.to_bytes();
    let solution = sudoku.solve_unique().unwrap().to_bytes();

    parse_board(&mut sudoku_avr.board, &sudoku_bytes);
    parse_board(&mut sudoku_avr.solution, &solution);

    return sudoku_avr;
}


