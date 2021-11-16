use anyhow::{bail, Context, Result};
use colored::*;
use log::{error, info, warn};
use serialport::{available_ports, DataBits, Parity, StopBits};
use simplelog::{ColorChoice, TermLogger, TerminalMode, ConfigBuilder};
use std::io::{ BufRead, BufReader, stdin, Read, Write };
use std::fs::{ File };
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use strum_macros::EnumString;

mod lib;

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
    author = "Stavrou Odysseas (canopus)"
)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "list")]
    List,

    #[structopt(name = "run")]
    Run(Run),

    #[structopt(name = "gen")]
    Gen(Gen),

    #[structopt(name = "prog")]
    Prog(Prog),
}

#[derive(StructOpt, Debug)]
struct Gen {
    #[structopt(long = "directory", short = "p")]
    directory: String,

    #[structopt(long = "number", short = "n")]
    number: u32,
}

#[derive(StructOpt, Debug)]
struct Prog {
    #[structopt(long = "dev", short = "u")]
    dev: String,

    #[structopt(long="stop-bits", default_value="1", possible_values(&["1", "2"]))]
    sb: u8,

    #[structopt(long="data-bits", default_value="8", possible_values(&["5", "6", "7", "8"]))]
    db: u8,

    #[structopt(long = "parity", short = "p", default_value = "None")]
    p: MyParity,

    #[structopt(long = "baud-rate", short = "r")]
    br: u32,

    #[structopt(long = "board-file", short = "b")]
    board: String,
}


#[derive(StructOpt, Debug)]
struct Run {
    #[structopt(long = "dev", short = "u")]
    dev: String,

    #[structopt(long = "difficulty", short = "d")]
    difficulty: lib::Difficulty,

    #[structopt(long="stop-bits", default_value="1", possible_values(&["1", "2"]))]
    sb: u8,

    #[structopt(long="data-bits", default_value="8", possible_values(&["5", "6", "7", "8"]))]
    db: u8,

    #[structopt(long = "parity", short = "p", default_value = "None")]
    p: MyParity,

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

fn run(dif: lib::Difficulty, port: &mut Box<dyn serialport::SerialPort>) -> Result<()> {
    let sudoku = lib::SudokuAvr::new(&dif);

    println!();
    info!("Generated Board!");
    sudoku.print_unsolved();

    info!("Generated Solution!");
    sudoku.print_solved();

    clear_to_send()?;
    info!("Sending Unsolved board to {:?}", port.name().unwrap());
    sudoku.send_board(port)?;
    //TODO Interactive
    Ok(())
}

fn open_port(port_config: &PortConfig) -> Result<Box<dyn serialport::SerialPort>> {
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

            clear_to_send()?;
            sudoku.send_board(&mut port)?;
            //TODO Interactive
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


fn clear_to_send() -> Result<()> {
    info!("Ready to send the Unsolved Board to the AVR");
    info!("Hit Enter when Ready!");
    let mut character = [0];

    while let Err(_) = stdin().read(&mut character) {
        bail!("Error Reading from Keyboard");
    }

    print!("Sending in ");
    for i in (0..=5).rev() {
        print!("{}...", i);
        std::io::stdout().flush()?;
        thread::sleep(Duration::from_millis(500));
    }
    println!();

    Ok(())
}
