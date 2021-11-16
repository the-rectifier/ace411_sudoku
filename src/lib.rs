use anyhow::{bail, Context, Result};
use colored::*;
use log::{error, info};
use rand::{thread_rng, Rng};
use std::io::ErrorKind;
use std::thread;
use std::str::from_utf8;
use std::time::Duration;
use strum_macros::{ EnumString, Display };
use sudoku::Sudoku;
use std::path::{ PathBuf };
use std::io::Write;
use std::fs::{ self, File, OpenOptions };


const EASY: u8 = 35;
const MEDIUM: u8 = 40;
const HARD: u8 = 45;



#[derive(Debug, EnumString, Clone, Display)]
pub enum Difficulty {
    #[strum(ascii_case_insensitive)]
    Easy,
    #[strum(ascii_case_insensitive)]
    Medium,
    #[strum(ascii_case_insensitive)]
    Hard,
}

// #[derive(Default)]
pub struct SudokuAvr {
    /* Hold the generated board */
    board: [[Cell; 9]; 9],
    /* Holds the whole solved board */
    solution: [[Cell; 9]; 9],

    filled: u8,

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
            filled: 0,
        };

        info!("Solving Board");
        SudokuAvr::solve_board(&mut board);

        info!("Removing Cells");

        match diff {
            Difficulty::Easy => SudokuAvr::remove_cells(&mut board.board, EASY),
            Difficulty::Medium => SudokuAvr::remove_cells(&mut board.board, MEDIUM),
            Difficulty::Hard => SudokuAvr::remove_cells(&mut board.board, HARD),
        };

        board.filled = SudokuAvr::count_filled(&board.board);
        return board;
    }

    pub fn new_from_str(line: &str) {
        info!("Generating Board");

        let board = SudokuAvr::parse_board(line.as_bytes());

    }

    fn count_filled(board: &[[Cell; 9]; 9]) -> u8 {
        let mut count: u8 = 0;
        for i in 0..board.len() {
            for j in 0..board[i].len() {
                if board[i][j].value != 0 {
                    count += 1;
                }
            }
        }

        return count;
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
            Difficulty::Easy => println!("{}", "EASY".blue()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow()),
            Difficulty::Hard => println!("{}", "HARD".red()),
        }

        SudokuAvr::print_board(&self.board);
    }

    pub fn print_solved(&self) {
        print!("{}", "Printing Solved Board!\nDifficulty: ".green());
        match self.dif {
            Difficulty::Easy => println!("{}", "EASY".blue()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow()),
            Difficulty::Hard => println!("{}", "HARD".red()),
        }
        SudokuAvr::print_board(&self.solution);
    }

    fn print_board(board: &[[Cell; 9]; 9]) {
        println!("\n\t---------------------------");
        for i in 0..board.len() {
            print!("\t{} | ", i + 1);
            for j in 0..board[i].len() {
                if board[i][j].value == 0 {
                    print!("_ ");
                } else {
                    print!("{} ", board[i][j].value);
                }
                if (j + 1) % 3 == 0 && (j + 1) != 9 {
                    print!("| ");
                }
            }
            print!("|");
            if (i + 1) % 3 == 0 && (i + 1) != 9 {
                print!("\n\t===========================");
            }
            println!();
        }
        println!("\t---------------------------");
        println!("\t🤘| 1 2 3 | 4 5 6 | 7 8 9 |\n");
    }

    fn parse_board(bytes: &[u8]) -> [[Cell; 9]; 9] {
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

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                string.push_str(self.board[i][j].value.to_string().as_str());
            }
        }
        println!("{}", string);
        return string;
    }

    pub fn send_board(&self, port: &mut Box<dyn serialport::SerialPort>) -> Result<()> {
        info!("Will send {} chunks to AVR!", self.filled);

        match port.write(&self.filled.to_ne_bytes()) {
            Ok(_) => {
                info!("Finished Sending");
                port.flush().unwrap();
                thread::sleep(Duration::from_millis(2));
                wait_ok(port)?;
            }
            Err(_) => {
                bail!("Unable to Write!");
            }
        }

        for i in 0..self.board.len() {
            for j in 0..self.board.len() {
                if self.board[i][j].value == 0 {
                    continue;
                }
                let chunk = &[i as u8, j as u8, self.board[i][j].value, b'\x0D', b'\x0A'];

                match port.write(chunk) {
                    Ok(_) => {
                        info!("Wrote {:?} to {:?}", chunk, port.name().unwrap());
                        port.flush().unwrap();
                        
                        wait_ok(port)?;
                    }
                    Err(_) => {
                        bail!("Unable to Write!");
                    }
                }
            }
        }
        info!("Done Sending!");
        Ok(())
    }
}

pub fn generate_boards(dir: String, diff: Difficulty, num: u32) -> Result<()> {
    for i in 1..=num {
        let filename = format!("{}_{}.txt", diff, i);
        let path = PathBuf::from(format!("./{}/", dir)).join(filename);
        
        let sudoku = SudokuAvr::new(&diff);

        let mut f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&path)
                    .with_context(|| format!("Failed to create {}", path.display()))?;

        write!(f, "{}", sudoku.to_string());
        info!("Created '{}'", path.display());
    }
    Ok(())
}

fn wait_ok(port: &mut Box<dyn serialport::SerialPort>) -> Result<()> {
    let data = read_uart(port)?;
    if b"OK\x0D\x0A" == &*data {
        info!("OK");
        return Ok(());
    } else {
        bail!("Invalid Response");
    }
}



fn read_uart(port: &mut Box<dyn serialport::SerialPort>) -> Result<Vec<u8>> {
    thread::sleep(Duration::from_millis(50));
    let readable_bytes = port.bytes_to_read()?;
    info!("Reading {} bytes from {}", readable_bytes, port.name().unwrap());

    let mut data: Vec<u8> = vec![0; readable_bytes as usize];
    match port.read(data.as_mut_slice()) {
        Ok(_) => (),
        Err(ref e) => {
            if e.kind() == ErrorKind::TimedOut {
                bail!("Timed out!")
            }
        }
    }

    Ok(data)
}