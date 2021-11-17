use log::info;
use colored::*;
use std::thread;
use sudoku::Sudoku;
use std::time::Duration;
use std::fs::OpenOptions;
use std::path::{ PathBuf };
use strum::IntoEnumIterator;
use rand::{ thread_rng, Rng };
use std::io::{ Write, ErrorKind };
use anyhow::{bail, Context, Result};
use strum_macros::{ EnumString, Display, EnumIter };

type Port = Box<dyn serialport::SerialPort>;

const EASY: u8 = 35;
const MEDIUM: u8 = 40;
const HARD: u8 = 45;
const ULTRA: u8 = 65;

#[derive(Debug, EnumString, Clone, Display, EnumIter)]
pub enum Difficulty {
    #[strum(ascii_case_insensitive)]
    Easy,
    #[strum(ascii_case_insensitive)]
    Medium,
    #[strum(ascii_case_insensitive)]
    Hard,
    #[strum(ascii_case_insensitive)]
    Ultra,
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
pub struct Cell {
    pub value: u8,
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
            Difficulty::Ultra => SudokuAvr::remove_cells(&mut board.board, ULTRA),
        };

        board.filled = SudokuAvr::count_filled(&board.board);
        return board;
    }

    pub fn new_from_str(line: &str, diff: Difficulty) -> Self {
        info!("Generating Board");

        let sudoku = Sudoku::from_str_line(line).expect("Unable to Create board from File");
        let solution = sudoku.solve_unique().unwrap().to_bytes();

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku.to_bytes()),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
            filled: 0,
        };

        board.filled = SudokuAvr::count_filled(&board.board);

        return board;
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


    pub fn check(&self, board: &[[Cell; 9]; 9]) -> bool {
        for i in 0..self.solution.len() {
            for j in 0..self.solution[i].len() {
                if self.solution[i][j].value != board[i][j].value {
                    return false;
                }
            } 
        }
        return true;
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
            Difficulty::Ultra => println!("{}", "ULTRA".red().bold()),
        }

        SudokuAvr::print_board(&self.board);
    }

    pub fn print_solved(&self) {
        print!("{}", "Printing Solved Board!\nDifficulty: ".green());
        match self.dif {
            Difficulty::Easy => println!("{}", "EASY".blue()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow()),
            Difficulty::Hard => println!("{}", "HARD".red()),
            Difficulty::Ultra => println!("{}", "ULTRA".red().bold()),
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

    pub fn send_board(&self, port: &mut Port) -> Result<()> {
        info!("Will send {} chunks to AVR!", self.filled);
        thread::sleep(Duration::from_millis(50));
        SudokuAvr::do_send(&self.board, port)?;
        Ok(())
    }

    fn do_send(board: &[[Cell; 9]; 9], port: &mut Port) -> Result<()> {
        for i in 0..board.len() {
            for j in 0..board.len() {
                if board[i][j].value == 0 {
                    continue;
                }

                let chunk = &[b'N', i as u8, j as u8, board[i][j].value, b'\x0D', b'\x0A'];
                match port.write(chunk) {
                    Ok(_) => {
                        info!("Wrote {:?} to {:?}", chunk, port.name().expect("Failed to get Uart Name"));
                        port.flush().expect("Unable to Flush!");
                        thread::sleep(Duration::from_millis(50));
                        wait_response(port, b"OK\x0D\x0A")?;
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


pub fn generate_boards(dir: String, num: u32) -> Result<()> {
    for diff in Difficulty::iter() {
        for i in 1..=num {
            let filename = format!("./{}_{}.txt", diff, i);
            let path = PathBuf::from(format!("./{}/", dir)).join(filename);
            
            let sudoku = SudokuAvr::new(&diff);

            let mut f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&path)
                        .with_context(|| format!("Failed to create {}", path.display()))?;

            write!(f, "{}", sudoku.to_string())?;
            info!("Created '{}'", path.display());
        }
    }
    Ok(())
}


pub fn wait_response(port: &mut Port, response: &[u8]) -> Result<()> {
    let data = read_uart(port, response.len() as i32)?;
    if response == &*data {
        info!("OK");
        return Ok(());
    } else {
        bail!("Invalid Response");
    }
}

pub fn wait_response_silent(port: &mut Port, response: &[u8]) -> Result<()> {
    let data = read_uart(port, response.len() as i32)?;
    if response == &*data {
        return Ok(());
    } else {
        bail!("");
    }
}



pub fn write_uart(port: &mut Port, data: &[u8]) -> Result<()> {
    info!("Writing {} bytes to {}", data.len(), port.name().expect("Failed to get Uart Name"));
    match port.write(data) {
        Ok(len) => { info!("Wrote {} bytes!", len); }
        Err(_) => { bail!("Unable to Write to Uart"); }
    }
    port.flush()?;
    thread::sleep(Duration::from_millis(50));
    Ok(())
}


pub fn read_uart(port: &mut Port, size: i32) -> Result<Vec<u8>> {
    let readable_bytes: usize;

    if size <= 0 {
        readable_bytes = port.bytes_to_read()? as usize;
    } else {
        readable_bytes = size as usize;
    }

    // info!("Reading {} bytes from {}", readable_bytes, port.name().expect("Failed to get Uart Name"));

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

