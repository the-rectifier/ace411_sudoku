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

// Declare Type for opened Port
type Port = Box<dyn serialport::SerialPort>;

// Declare Amount of Cells to be removed based on difficulty level
const EASY: u8 = 35;
const MEDIUM: u8 = 40;
const HARD: u8 = 45;
const ULTRA: u8 = 81;

// Implement Appropriate Traits for Difficulty Enum
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

// Define Structs
// #[derive(Default)]
pub struct SudokuAvr {
    /* Hold the generated board */
    board: [[Cell; 9]; 9],
    /* Holds the whole solved board */
    solution: [[Cell; 9]; 9],

    filled: u8,

    dif: Difficulty,
}

#[derive(Default, Debug)]
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

        let solution = sudoku.solve_unique().unwrap().to_bytes();

        info!("Generating Board!");

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku_bytes),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
            filled: 0,
        };

        board.filled = SudokuAvr::count_filled(&board.board);
        // println!("Filled Before: {}", SudokuAvr::count_filled(&board.board));
        info!("Solving Board");
        SudokuAvr::solve_board(&mut board);
        
        info!("Removing Cells");
        match diff {
            Difficulty::Easy => SudokuAvr::remove_cells(&mut board, EASY),
            Difficulty::Medium => SudokuAvr::remove_cells(&mut board, MEDIUM),
            Difficulty::Hard => SudokuAvr::remove_cells(&mut board, HARD),
            Difficulty::Ultra => SudokuAvr::remove_cells(&mut board, ULTRA),
        };
        
        board.filled = SudokuAvr::count_filled(&board.board);
        return board;
    }

    // Constructor using a string slice as argument
    // Removes Cells depending on Difficulty level
    // Returns Instantiated Struct 
    pub fn new_from_str(line: &str, diff: Difficulty) -> Self {
        info!("Generating Board");

        let sudoku = Sudoku::from_str_line(line).expect("Unable to Create board from File");
        let solution = sudoku.solve_unique().expect("Unsolvable Board").to_bytes();

        let mut board = SudokuAvr {
            board: SudokuAvr::parse_board(&sudoku.to_bytes()),
            solution: SudokuAvr::parse_board(&solution),
            dif: diff.clone(),
            filled: 0,
        };

        board.filled = SudokuAvr::count_filled(&board.board);

        return board;
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
        return count;
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
        return true;
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
        print!("{}", "Printing Unsolved Board!\nDifficulty: ".green().bold());
        match self.dif {
            Difficulty::Easy => println!("{}", "EASY".blue()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow()),
            Difficulty::Hard => println!("{}", "HARD".red()),
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
            Difficulty::Easy => println!("{}", "EASY".blue()),
            Difficulty::Medium => println!("{}", "MEDIUM".yellow()),
            Difficulty::Hard => println!("{}", "HARD".red()),
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

        return board;
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
        return string;
    }

    // Wrapper around do_send() Method
    // Will count the amount of cells to send to the MCU
    pub fn send_board(&self, port: &mut Port) -> Result<()> {
        info!("Will send {} chunks to AVR!", self.filled);
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

                let chunk = &[b'N', (j as u8 + 1) + 0x30, (i as u8 + 1) + 0x30, board[i][j].value + 0x30, b'\x0D', b'\x0A'];
                match port.write(chunk) {
                    Ok(_) => {
                        info!("Wrote {:?} to {:?}", chunk, port.name().expect("Failed to get UART Name"));
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

// Given a Directory dir as a string and a number n
// Generate n Boards of Each Difficulty inside dir
pub fn generate_boards(dir: String, num: u32) -> Result<()> {
    for diff in Difficulty::iter() {
        for i in 1..=num {
            let filename = format!("{}_{}.txt", diff, i);
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


// Waits for specified response
// returns Err if not valid Response
pub fn wait_response(port: &mut Port, response: &[u8]) -> Result<()> {
    let data = read_uart(port, response.len() as i32)?;
    if response == &*data {
        // info!("{:?}", data);
        return Ok(());
    } else {
        bail!("Invalid Response");
    }
}

// Writes data argument to UART port
// Flushes buffer and waits for 50ms before returning
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


// Read size bytes from UART
// if size is < 0 then reads entire buffer
pub fn read_uart(port: &mut Port, size: i32) -> Result<Vec<u8>> {
    let readable_bytes: usize;

    if size <= 0 {
        readable_bytes = port.bytes_to_read()? as usize;
    } else {
        readable_bytes = size as usize;
    }

    // info!("Reading {} bytes from {}", readable_bytes, port.name().expect("Failed to get Uart Name"));

    let mut data: Vec<u8> = vec![0 as u8; readable_bytes as usize];
    match port.read(data.as_mut_slice()) {
        Ok(_) => (),
        Err(ref e) => {
            if e.kind() == ErrorKind::TimedOut {
                bail!("Timed out!")
            }
        }
    }
    info!("Bytes Read: {:?}", data);
    Ok(data)
}

