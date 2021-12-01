use colored::*;
use std::thread;
use pad::PadStr;
use std::fs::File;
use std::time::Duration;
use structopt::StructOpt;
use std::time::{Instant};
use log::{ error, info };
use strum_macros::EnumString;
use anyhow::{ bail, Context, Result };
use std::io::{ BufRead, BufReader, stdin, Write };
use serialport::{ available_ports, DataBits, Parity, StopBits };
use simplelog::{ ColorChoice, TermLogger, TerminalMode, ConfigBuilder };

mod lib;

// Define a new Type for Open Port
type Port = Box<dyn serialport::SerialPort>;

// Define constants replies
const OK: &[u8] = b"OK\r\n";
const AT: &[u8] = b"AT\r\n";
const DONE: &[u8] = b"D\r\n";
const CLEAR: &[u8] = b"C\r\n";
const T: &[u8] = b"T\r\n";
const BREAK: &[u8] = b"B\r\n";
const PLAY: &[u8] = b"P\r\n";
const SAVE: &[u8] = b"S\r\n";


#[derive(Debug, EnumString)]
enum MyParity {
    #[strum(ascii_case_insensitive)]
    None,
    #[strum(ascii_case_insensitive)]
    Even,
    #[strum(ascii_case_insensitive)]
    Odd,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ACE411 - Sudoku <=> AVR Interface",
    author = "Stavrou Odysseas (canopus)",
    version = "0.6",
)]
struct Opts {
    /// Command to run
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Lists Available Serial Ports
    #[structopt(name = "list")]
    List,

    /// Run Mode
    #[structopt(name = "run")]
    Run(Run),

    /// Generate Boards
    #[structopt(name = "gen")]
    Gen(Gen),

    /// Download Board to MCU
    #[structopt(name = "prog")]
    Prog(Prog),
}

#[derive(StructOpt, Debug)]
struct Gen {
    /// Directory to place the Boards
    #[structopt(long = "directory", short = "d")]
    directory: String,

    /// Generate <number> boards for EACH difficulty level
    #[structopt(long = "number", short = "n")]
    number: u32,
}

#[derive(StructOpt, Debug)]
struct Prog {
    /// Device Port 
    #[structopt(long = "dev", short = "u")]
    dev: String,

    /// Stop Bits
    #[structopt(long="stop-bits", default_value="1", possible_values(&["1", "2"]))]
    sb: u8,

    /// Data Bits
    #[structopt(long="data-bits", default_value="8", possible_values(&["5", "6", "7", "8"]))]
    db: u8,

    /// Parity
    #[structopt(long = "parity", short = "p", default_value = "None")]
    p: MyParity,

    /// Baudrate
    #[structopt(long = "baud-rate", short = "r")]
    br: u32,

    /// Board file to download
    #[structopt(long = "board-file", short = "b")]
    board: String,

    /// Enter Interactive shell
    #[structopt(long = "interactive", short = "i")]
    inter: bool,
}


#[derive(StructOpt, Debug)]
struct Run {
    /// Device Port
    #[structopt(long = "dev", short = "u")]
    dev: String,

    /// Difficulty of Game
    /// [possible values: Easy, Medium, Hard, Ultra]
    #[structopt(long = "difficulty", short = "d")]
    difficulty: lib::Difficulty,

    /// Stop Bits
    #[structopt(long="stop-bits", default_value="1", possible_values(&["1", "2"]))]
    sb: u8,

    /// Data Bits
    #[structopt(long="data-bits", default_value="8", possible_values(&["5", "6", "7", "8"]))]
    db: u8,

    /// Parity
    #[structopt(long = "parity", short = "p", default_value = "None")]
    p: MyParity,

    /// Baudrate
    #[structopt(long = "baud-rate", short = "r")]
    br: u32,

    // #[structopt(long = "timeout", short = "t", default_value = "200")]
    // timeout: u64,
}


struct PortConfig {
    baud_rate: u32, 
    stop_bits: StopBits,
    data_bits: DataBits,
    parity: Parity,
    dev: String,
}


fn get_ports() {
    let ports = available_ports().expect("No ports found!");
    for (i, p) in ports.iter().enumerate() {
        info!("Found Port ({}): {}", i, p.port_name);
    }
}

fn run(dif: lib::Difficulty, port: &mut Port) -> Result<()> {
    let sudoku = lib::SudokuAvr::new(&dif);

    println!();
    info!("Generated Board!");
    sudoku.print_unsolved();

    info!("Generated Solution!");
    sudoku.print_solved();

    info!("Going Interactive!");
    go_interactive(port, &sudoku, false)?;

    Ok(())
}

fn open_port(port_config: &PortConfig) -> Result<Port> {
    let builder = serialport::new(port_config.dev.as_str(), port_config.baud_rate)
        .stop_bits(port_config.stop_bits)
        .data_bits(port_config.data_bits)
        // .timeout(Duration::from_millis(timeout))
        .parity(port_config.parity);

    let port = builder
        .open()
        .with_context(|| format!("Unable to open port {}!", port_config.dev))?;

    info!("{}", "Opened Port Successfully!!".green());

    Ok(port)
}

fn main() -> Result<()> {
    TermLogger::init(
        log::LevelFilter::Info,
        ConfigBuilder::new().set_time_to_local(true).build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to init logger");

    match Command::from_args() {
        Command::List => {
            get_ports();
            return Ok(());
        }
        Command::Run(args) => {
            let port_config = PortConfig {
                baud_rate: args.br,
                stop_bits: check_stop_bits(args.sb)?,
                data_bits: check_data_bits(args.db)?,
                parity: check_parity(args.p)?,
                dev: args.dev,
            };

            let mut port = open_port(&port_config)?;

            if let Err(e) = run(args.difficulty, &mut port) {
                error!("{:?}", e);
                std::process::exit(-1);
            }
        }
        Command::Prog(args) => {
            let port_config = PortConfig {
                baud_rate: args.br,
                stop_bits: check_stop_bits(args.sb)?,
                data_bits: check_data_bits(args.db)?,
                parity: check_parity(args.p)?,
                dev: args.dev,
            };

            let mut port = open_port(&port_config)?;
            let file = File::open(&args.board)?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let vec: Vec<&str> = args.board.split('/').collect();
            let vec: Vec<&str> = vec[vec.len()-1].split('_').collect();
            let diff: lib::Difficulty;

            match vec[0] {
                "Easy" => diff = lib::Difficulty::Easy,
                "Medium" => diff = lib::Difficulty::Medium,
                "Hard" => diff = lib::Difficulty::Hard,
                _ => bail!("Invalid File Name")
            }

            reader.read_line(&mut line)?;
            let sudoku = lib::SudokuAvr::new_from_str(&mut line, diff);
            sudoku.send_board(&mut port)?;
            if args.inter {
                info!("Going Interactive!");
                go_interactive(&mut port, &sudoku, true)?;
            }
        }
        Command::Gen(gen) => { lib::generate_boards(gen.directory, gen.number)?; }
    }

    Ok(())
}


fn check_stop_bits(sb: u8) -> Result<StopBits> {
    match sb {
        1 => Ok(StopBits::One),
        2 => Ok(StopBits::Two),
        _ => bail!("Invalid Stop Bits"),
    }
}


fn check_data_bits(db: u8) -> Result<DataBits> {
    match db {
        5 => Ok(DataBits::Five),
        6 => Ok(DataBits::Six),
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => bail!("Invalid Data Bits"),
    }
}


fn check_parity(parity: MyParity) -> Result<Parity> {
    match parity {
        MyParity::Even => Ok(Parity::Even),
        MyParity::Odd => Ok(Parity::Odd),
        MyParity::None => Ok(Parity::None),
    }
}


fn ct_msg(msg: &str) -> Result<()> {
    info!("Hit Enter when Read!");

    let mut input = String::new();
    stdin().read_line(&mut input).with_context(|| format!("Unable to Read Line!"))?;

    print!("{}", msg);
    print!("Sending in ");
    for i in (0..=5).rev() {
        print!("{}...", i);
        std::io::stdout().flush()?;
        thread::sleep(Duration::from_millis(500));
    }
    println!();


    Ok(())
}


fn go_interactive(port: &mut Port, sudoku: &lib::SudokuAvr, flag: bool) -> Result<()> {
    let mut flag_send = flag;
    let mut user_input = String::new();
    
    loop {
        print!("{}", "> ".green().bold());
        std::io::stdout().flush().expect("Couldn't Flush STDOUT");

        user_input.clear();
        stdin().read_line(&mut user_input).with_context(|| format!("Unable to Read Line!"))?;
        let user_input_vec: Vec<&str> = user_input.split_ascii_whitespace().collect(); 

        match user_input_vec[0] {
            "at" => {
                lib::write_uart(port, AT)?;
                lib::wait_response(port, OK)?;
            },
            "clear" => { 
                lib::write_uart(port, CLEAR)?; 
                lib::wait_response(port, OK)?;
                flag_send = false; 
            },
            "break" => {
                lib::write_uart(port, BREAK)?;
                lib::wait_response(port, OK)?;
            },
            "play" => {
                if !flag_send {
                    error!("No board Downloaded!");
                    continue;
                }
                lib::write_uart(port, PLAY)?;

                let time_now = Instant::now();
                
                lib::wait_response(port, OK)?; 
                
                loop {
                    match lib::wait_response(port, DONE) {
                        Ok(_) => break,
                        Err(_) => continue
                    }
                }
                let time_elapsed = time_now.elapsed();
                info!("{}", format!("Solved in: {:?}", time_elapsed).green().bold());
                info!("Ready to Receive the Solved Board from the AVR?");
                ct_msg("Receiving in ")?;
                match recv_and_check(port, &sudoku) {
                    Ok(_) => info!("{}", format!("Valid Solution!!").green().bold()),
                    Err(_) => info!("{}", format!("Invalid Solution! :( ").red().bold()),
                }
            },
            "exit" => break,
            "download" => {
                if flag_send {
                    error!("Game Already Running!");
                    continue;
                }
                
                info!("Ready to Send the Unsolved Board to the AVR?");
                ct_msg("Sending in ")?;
                info!("Sending Unsolved board to {:?}", port.name().expect("Couldn't Get Port Name!"));
                sudoku.send_board(port)?;
                flag_send = true;
            }
            "fill" => {
                if user_input_vec.len() > 4 {
                    error!("Invalid Command!");
                    continue;
                }

                let x: u8;
                let y: u8;
                let z: u8;

                match user_input_vec[1].parse::<u8>() {
                    Ok(num) => x = num,
                    Err(_) => { error!("Arguments must be within 1-9"); continue;}
                }
                match user_input_vec[2].parse::<u8>() {
                    Ok(num) => y = num,
                    Err(_) => { error!("Arguments must be within 1-9"); continue;}
                }
                match user_input_vec[3].parse::<u8>() {
                    Ok(num) => z = num,
                    Err(_) => { error!("Arguments must be within 1-9"); continue;}
                }

                if x > 9 || y > 9 || z > 9 {
                    error!("Arguments must be within 1-9");
                    continue;
                }
                lib::write_uart(port, [b'N', x, y, z, b'\x0D', b'\x0A'].as_ref())?;
                lib::wait_response(port, OK)?;
            },
            "debug" => {
                if user_input_vec.len() > 3 {
                    error!("Invalid Command!");
                    continue;
                }
                let x: u8;
                let y: u8;

                match user_input_vec[1].parse::<u8>() {
                    Ok(num) => x = num,
                    Err(_) => { error!("Arguments must be within 1-9"); continue;}
                }
                match user_input_vec[2].parse::<u8>() {
                    Ok(num) => y = num,
                    Err(_) => { error!("Arguments must be within 1-9"); continue;}
                }

                if x > 9 || y > 9 {
                    error!("Arguments must be within 1-9");
                    continue;
                }

                lib::write_uart(port, [b'D', x + 0x30, y + 0x30, b'\x0D', b'\x0A'].as_ref())?;
                let data = lib::read_uart(port, 6)?;

                info!("{}", format!("[{},{}]: {}", data[1] & 0x0F, data[2] & 0x0F, data[3] & 0x0F).yellow().bold());

            },
            "solution" => sudoku.print_solved(),
            "unsolved" => sudoku.print_unsolved(),
            "help" | "?" => print_help(),
            _ => error!("Invalid Command!"),
        }
    }
    Ok(())
}


fn recv_and_check(port: &mut Port, sudoku: &lib::SudokuAvr) -> Result<()> {
    let mut p_board: [[lib::Cell; 9]; 9] = Default::default();

    lib::write_uart(port, &SAVE)?;
    let mut data: Vec<u8>; 

    loop {
        data = lib::read_uart(port, 6)?;
        // println!("{:?}", data);
        if &data[..3] == DONE {
            lib::write_uart(port, &OK)?;
            break;
        }
        p_board[(data[2] & 0x0F) as usize -1][(data[1] & 0x0F) as usize -1].value = data[3] & 0x0F;
        lib::write_uart(port, &T)?;
    }
    
    if sudoku.check(&p_board) { return Ok(()); } else { bail!(""); }
}


fn print_help() {
    println!("{}", "Available Commands: ".yellow().bold());
    println!("{}{}", "at".pad_to_width(20).white().bold(), "Attention".white().bold());
    println!("{}{}", "clear".pad_to_width(20).white().bold(), "Clear Board".white().bold());
    println!("{}{}", "play".pad_to_width(20).white().bold(), "Play Game".white().bold());
    println!("{}{}", "fill".pad_to_width(20).white().bold(), "Fill Cell [x y num]".white().bold());
    println!("{}{}", "solution".pad_to_width(20).white().bold(), "Print Solution".white().bold());
    println!("{}{}", "unsolved".pad_to_width(20).white().bold(), "Print Board".white().bold());
    println!("{}{}", "download".pad_to_width(20).white().bold(), "Download Board to AVR".white().bold());
    println!("{}{}", "break".pad_to_width(20).white().bold(), "Break".white().bold());
    println!("{}{}", "debug".pad_to_width(20).white().bold(), "Return the contents of a Cell [x y num]".white().bold());
    println!("{}{}", "exit".pad_to_width(20).white().bold(), "Exit".white().bold());
    println!("{}{}", "help or ?".pad_to_width(20).white().bold(), "Print this Help message".white().bold());
}
