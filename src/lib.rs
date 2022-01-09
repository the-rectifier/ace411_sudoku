use anyhow::{bail, Context, Result};
use colored::*;
use log::{debug, error, info};
use serialport::ClearBuffer;
use std::fs::{self, create_dir, File, OpenOptions};
use std::io::{stdin, BufRead, BufReader, ErrorKind, Write};
use std::path::PathBuf;
use std::str;
use std::thread;
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;

#[path = "sudoku_avr.rs"]
pub mod sudoku_avr;

pub use sudoku_avr::{Cell, Difficulty, SudokuAvr};
// Declare Type for opened Port
type Port = Box<dyn serialport::SerialPort>;
// Define constants replies
const OK: &[u8] = b"OK\r\n";
const AT: &[u8] = b"AT\r\n";
const DONE: &[u8] = b"D\r\n";
const CLEAR: &[u8] = b"C\r\n";
const T: &[u8] = b"T\r\n";
const PLAY: &[u8] = b"P\r\n";
const SAVE: &[u8] = b"S\r\n";

// Given a Directory dir as a string and a number ns
// Generate n Boards of Each Difficulty inside dir
pub fn generate_boards(dir: String, num: u32) -> Result<()> {
    for diff in Difficulty::iter() {
        for i in 1..=num {
            // let filename = format!("{}_{}.txt", diff, i);
            let filename = format!("{}_{}.txt", diff, i);
            let path = PathBuf::from(format!("./{}/", dir)).join(filename);
            let sudoku = SudokuAvr::new(&diff);

            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .with_context(|| format!("Failed to create {}", path.display()))?;

            write!(f, "{}\n{}", sudoku.dif, sudoku.to_string())?;
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
        debug!(
            "Read: {}",
            str::from_utf8(&data)?.replace("\r", "").replace("\n", "")
        );
        return Ok(());
    } else {
        bail!("Invalid Response {:?}", data);
    }
}

// Writes data argument to UART port
// Flushes buffer and waits for 50ms before returning
pub fn write_uart(port: &mut Port, data: &[u8]) -> Result<()> {
    debug!(
        "Writing {} bytes to {}",
        data.len(),
        port.name().expect("Failed to get Uart Name")
    );
    match port.write(data) {
        Ok(len) => {
            debug!("Wrote {} bytes!", len);
        }
        Err(_) => {
            bail!("Unable to Write to Uart");
        }
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
    debug!("Bytes Read: {:?}", data);
    Ok(data)
}

// Traverse Directory and find board files
// Construct a Vector with the board files and sort it based on Difficulty
fn prep_boards(dir: &String) -> Result<Vec<SudokuAvr>> {
    let paths = fs::read_dir(dir).with_context(|| format!("{}{}", "Unable to read", dir))?;
    let mut boards: Vec<SudokuAvr> = Vec::new();

    for path in paths {
        let path = path?;
        if path.path().is_dir() {
            continue;
        }
        let file = File::open(path.path())?;
        let mut line = String::new();
        let mut reader = BufReader::new(file);
        reader.read_line(&mut line)?;

        let diff = match line.replace("\n", "").as_str() {
            "Easy" => Difficulty::Easy,
            "Medium" => Difficulty::Medium,
            "Hard" => Difficulty::Hard,
            "Ultra" => Difficulty::Ultra,
            _ => bail!("Invalid Dificulty!"),
        };
        line.clear();
        reader.read_line(&mut line)?;
        let board = SudokuAvr::new_from_str(&line, diff);
        boards.push(board);
    }
    boards.sort();
    Ok(boards)
}

// For a specific team, iterate over all provided boards
// Play each board and log time and solution to a file
pub fn play_tournament(dir: &String, team: &String, port: &mut Port) -> Result<()> {
    info!("{}", "Prepairing Boards!".white().bold());
    let boards = prep_boards(dir)?;

    let dir = "tournament";
    let mut total_time: f64 = 0.0;
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

    let filename = format!("team_{}.txt", team);
    let path = PathBuf::from(format!("./{}", dir)).join(filename.clone());

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .with_context(|| format!("Failed to create {}", path.display()))?;

    write!(f, "Team: {}\n", team)?;
    write!(f, "-------------------\n")?;

    for (i, board) in boards.iter().enumerate() {
        // Check if Board is Live
        // send at
        write_uart(port, AT)?;
        // wait for ok
        wait_response(port, OK)?;
        // send clear
        write_uart(port, CLEAR)?;
        // wait for ok
        wait_response(port, OK)?;
        // send board
        info!("{}", "Sending Board".white().bold());
        board.send_board(port)?;
        // clear buffers
        port.clear(ClearBuffer::All)
            .with_context(|| format!("Unable to Clear Buffers"))?;

        info!(
            "{}",
            format!("Playing Board: {} Difficulty: {}", i, board.dif)
                .white()
                .bold()
        );
        write_uart(port, PLAY)?;

        let time_now = Instant::now();

        wait_response(port, OK)?;

        // Wait until solution
        loop {
            match wait_response(port, DONE) {
                Ok(()) => break,
                Err(_) => continue,
            }
        }

        let time_elapsed = time_now.elapsed();
        total_time += time_elapsed.as_secs_f64();

        let mut sol = false;

        // log time and solution
        match recv_and_check(port, board) {
            Ok(()) => {
                info!("{}", format!("Valid Solution!!").green().bold());
                sol = true;
            }
            Err(_) => info!("{}", format!("Invalid Solution! :( ").red().bold()),
        }

        // Clear Buffers
        port.clear(ClearBuffer::All)
            .with_context(|| format!("Unable to Clear Buffers"))?;

        // Log solution
        write!(
            f,
            "Board: {}\nDifficulty: {}\nTime to solve: {:?}\nValid Solution: {}\n",
            i, board.dif, time_elapsed, sol
        )?;

        write!(f, "-------------------\n")?;
        info!(
            "Board: {} Difficulty: {} Solved in: {:?}",
            i, board.dif, time_elapsed
        );

        info!("Press Enter to Send Next Board!");
        let mut junk = String::new();
        stdin()
            .read_line(&mut junk)
            .with_context(|| format!("Unable to Read Line!"))?;
        junk.clear();
    }
    write!(f, "Total Time: {:.4} seconds\n", total_time)?;
    write!(f, "Finished Playing!\n")?;
    info!(
        "{}",
        format!("Team {} done! in {:.4} seconds", team, total_time)
            .white()
            .bold()
    );
    Ok(())
}

pub fn recv_and_check(port: &mut Port, sudoku: &SudokuAvr) -> Result<()> {
    let mut p_board: [[Cell; 9]; 9] = Default::default();

    write_uart(port, &SAVE)?;
    let mut data: Vec<u8>;

    loop {
        data = read_uart(port, 6)?;
        // println!("{:?}", data);
        debug!("{:?}", data);
        if &data[..3] == DONE {
            write_uart(port, &OK)?;
            break;
        }
        p_board[(data[2] - 0x31) as usize][(data[1] - 0x31) as usize].value = data[3] - 0x30;
        write_uart(port, &T)?;
    }

    info!("{}", "Player Board: ".white().bold());
    SudokuAvr::print_board(&p_board);
    port.clear(ClearBuffer::All)
        .with_context(|| format!("Unable to Clear Buffers"))?;

    match sudoku.check(&p_board) {
        true => Ok(()),
        false => bail!(""),
    }
}
