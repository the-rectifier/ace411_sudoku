use anyhow::{bail, Context, Result};
use colored::*;
use log::{debug, error, info};
use rand::{thread_rng, Rng};
use std::fs::{create_dir, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::str;
use std::thread;
use std::time::Duration;
use strum_macros::{Display, EnumIter, EnumString};
use sudoku::Sudoku;

use crate as lib;

type Port = Box<dyn serialport::SerialPort>;

// Declare Amount of Cells to be removed based on difficulty level
const EASY: u8 = 35;
const MEDIUM: u8 = 40;
const HARD: u8 = 45;
const ULTRA: u8 = 81;

// Implement Appropriate Traits for Difficulty Enum
#[derive(Debug, EnumString, Clone, Display, EnumIter, PartialEq, Eq, Ord, PartialOrd)]
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

// Define Structs
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct SudokuAvr {
    pub dif: Difficulty,
    /* Hold the generated board */
    board: [[Cell; 9]; 9],
    /* Holds the whole solved board */
    solution: [[Cell; 9]; 9],
    filled: u8,
    pub tts: u64,
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Cell {
    pub value: u8,
    orig: bool,
}

impl SudokuAvr {
    // Constructor for struct Sudoku
    // Takes as argument the level of Difficulty and removes Cells accordingly
    // Cells are removed randomly, but still keeping the board uniquely solvable
    // returns instantiated Struct
    pub fn new(diff: &Difficulty) -> Self {
        let sudoku = Sudoku::generate_unique();
        let sudoku_bytes = sudoku.to_bytes();

        let solution = sudoku
            .solve_unique()
            .expect("Unable to find Unique Solution!")
            .to_bytes();

        debug!("Generating Board!");

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku_bytes),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
            filled: 0,
            tts: 0,
        };

        board.filled = SudokuAvr::count_filled(&board.board);
        // println!("Filled Before: {}", SudokuAvr::count_filled(&board.board));
        debug!("Solving Board");
        SudokuAvr::solve_board(&mut board);

        debug!("Removing Cells");
        match diff {
            Difficulty::Easy => SudokuAvr::remove_cells(&mut board, EASY),
            Difficulty::Medium => SudokuAvr::remove_cells(&mut board, MEDIUM),
            Difficulty::Hard => SudokuAvr::remove_cells(&mut board, HARD),
            Difficulty::Ultra => SudokuAvr::remove_cells(&mut board, ULTRA),
        };

        board.filled = SudokuAvr::count_filled(&board.board);
        board
    }

    // Constructor using a string slice as argument
    // Removes Cells depending on Difficulty level
    // Returns Instantiated Struct
    pub fn new_from_str(line: &str, diff: Difficulty) -> Self {
        debug!("Generating Board");

        let sudoku = Sudoku::from_str_line(line).expect("Unable to Create board from File");
        let solution = sudoku.solve_unique().expect("Unsolvable Board").to_bytes();

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku.to_bytes()),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
            filled: 0,
            tts: 0,
        };

        board.filled = SudokuAvr::count_filled(&board.board);

        board
    }

    // Counts filled cells
    fn count_filled(board: &[[Cell; 9]; 9]) -> u8 {
        let mut count: u8 = 0;
        for i in 0..board.len() {
            for j in 0..board[i].len() {
                if board[i][j].value != 0 {
                    count += 1;
                }
            }
        }
        count
    }

    // Copies the solution from one array to the other
    // SKIPPING original cells
    // Clone Trait would not have worked
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
        true
    }

    // Removes Cells based on an RNG
    // Skip Cell if original so that board will not loose uniqueness
    fn remove_cells(board: &mut SudokuAvr, no_cells: u8) {
        let mut limit = 0;
        let mut rng = thread_rng();

        while limit < no_cells && limit != 81 - board.filled {
            let i: usize = rng.gen_range(0..9) as usize;
            let j: usize = rng.gen_range(0..9) as usize;

            if board.board[i][j].orig {
                continue;
            } else if board.board[i][j].value == 0 {
                continue;
            } else {
                board.board[i][j].value = 0;
                limit += 1;
            }
        }
    }

    // Wrapper around print_board() Method that prints the unsolved Board
    pub fn print_unsolved(&self) {
        print!(
            "{}",
            "Printing Unsolved Board!\nDifficulty: ".green().bold()
        );
        match self.dif {
            Difficulty::Easy => println!("{}", "EASY".blue().bold()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow().bold()),
            Difficulty::Hard => println!("{}", "HARD".red().bold()),
            Difficulty::Ultra => println!("{}", "ULTRA".red().bold()),
        }
        print!("{}", "Filled Cells: ".green().bold());
        println!("{}", format!("{}", self.filled).white().bold());
        SudokuAvr::print_board(&self.board);
    }

    // Wrapper around print_board() Method that prints the unsolved Board
    pub fn print_solved(&self) {
        print!("{}", "Printing Solved Board!\nDifficulty: ".green().bold());
        match self.dif {
            Difficulty::Easy => println!("{}", "EASY".blue().bold()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow().bold()),
            Difficulty::Hard => println!("{}", "HARD".red().bold()),
            Difficulty::Ultra => println!("{}", "ULTRA".red().bold()),
        }
        SudokuAvr::print_board(&self.solution);
    }

    // Prints board with correct formatting
    pub fn print_board(board: &[[Cell; 9]; 9]) {
        println!("{}", "\n\t---------------------------".bold().white());
        for i in 0..board.len() {
            print!("{}", format!("\t{} | ", i + 1).white().bold());
            for j in 0..board[i].len() {
                if board[i][j].value == 0 {
                    print!("{}", "_ ".white().bold());
                } else {
                    print!("{}", format!("{} ", board[i][j].value).white().bold());
                }
                if (j + 1) % 3 == 0 && (j + 1) != 9 {
                    print!("{}", "| ".white().bold());
                }
            }
            print!("{}", "|".white().bold());
            if (i + 1) % 3 == 0 && (i + 1) != 9 {
                print!("{}", "\n\t===========================".white().bold());
            }
            println!();
        }
        println!("{}", "\t---------------------------".white().bold());
        println!("{}", "\t🤘| 1 2 3 | 4 5 6 | 7 8 9 |\n".white().bold());
    }

    // Parses a 81-byte array into a 9x9 Cell array
    // Marks the original Cells
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

        board
    }

    pub fn export_board(&self) -> Result<()> {
        if self.tts == 0 {
            error!("No Solution Time Found");
            return Ok(());
        }
        let dir = "exports";
        match create_dir(dir) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                ErrorKind::AlreadyExists => (),
                _ => {
                    error!("Unable to Create directory!");
                    bail!("{}", format!("{:#}", e));
                }
            },
        }

        let filename = format!("{}_{}s.txt", self.dif, self.tts);
        let path = PathBuf::from(format!("./{}", dir)).join(filename.clone());

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("Failed to create {}", path.display()))?;

        write!(f, "{}\n{}", self.dif, self.to_string())?;
        info!("{}", format!("{}: {}", filename, "Exported Successfully"));
        Ok(())
    }

    // returns a String representation of the 9x9 Array
    // 0,0 -> 1st, 0,1 -> 2nd etc
    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                string.push_str(self.board[i][j].value.to_string().as_str());
            }
        }
        // println!("{}", string);
        string
    }

    // Wrapper around do_send() Method
    // Will count the amount of cells to send to the MCU
    pub fn send_board(&self, port: &mut Port) -> Result<()> {
        debug!("Will send {} chunks to AVR!", self.filled);
        thread::sleep(Duration::from_millis(50));
        SudokuAvr::do_send(&self.board, port)?;
        Ok(())
    }

    // Loops over the given board and sends each Cell in the correct format
    // [N<X><Y><NUM><CR><LF>]: 6 bytes
    // Skip empty cells
    // Will flush the buffer and sleep for 50ms
    // Wait for the correct response from the MCU
    fn do_send(board: &[[Cell; 9]; 9], port: &mut Port) -> Result<()> {
        for i in 0..board.len() {
            for j in 0..board.len() {
                if board[i][j].value == 0 {
                    continue;
                }

                let chunk = &[
                    b'N',
                    (j as u8 + 1) + 0x30,
                    (i as u8 + 1) + 0x30,
                    board[i][j].value + 0x30,
                    b'\x0D',
                    b'\x0A',
                ];
                match port.write(chunk) {
                    Ok(_) => {
                        debug!(
                            "Wrote {} to {:?}",
                            str::from_utf8(chunk)?,
                            port.name().expect("Failed to get UART Name")
                        );
                        port.flush().expect("Unable to Flush!");
                        thread::sleep(Duration::from_millis(50));
                        lib::wait_response(port, b"OK\x0D\x0A")?;
                    }
                    Err(_) => {
                        bail!("Unable to Write!");
                    }
                }
            }
        }
        info!("{}", "Done Sending!".white().bold());
        Ok(())
    }
}
