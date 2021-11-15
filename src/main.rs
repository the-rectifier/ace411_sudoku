use serialport::{ available_ports, DataBits, StopBits, Parity };
use std::io::{self, Write};
use anyhow::{ Context, Result, bail };
use log::{ info, error };
use simplelog::{ ColorChoice, TermLogger, TerminalMode };
use colored::*;
use structopt::StructOpt;
use strum_macros::EnumString;

mod sudoku_lib;

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
}

#[derive(StructOpt, Debug)]
struct Run {
    #[structopt(long="dev", short="u", required_unless("ports"))]
    dev: String,

    #[structopt(long="difficulty", short="d")]
    difficulty: sudoku_lib::Difficulty,

    #[structopt(long="stop-bits", default_value="1", possible_values(&["1", "2"]))]
    sb: u32,

    #[structopt(long="data-bits", default_value="8", possible_values(&["5", "6", "7", "8"]))]
    db: u32,

    #[structopt(long="parity", short="p", default_value="None")]
    p: MyParity,

    #[structopt(long="baud-rate", short="r")]
    br: u32,
}

fn get_ports() { 
    let ports = available_ports().expect("No ports found!");
    for (i, p) in ports.iter().enumerate() {
        info!("Found Port ({}): {}", i, p.port_name);
    }
}



fn run(dif: sudoku_lib::Difficulty, port: &mut Box<dyn serialport::SerialPort>) -> Result<()> {
    let sudoku = sudoku_lib::SudokuAvr::new(dif);

    info!("Sending Unsolved board to {:?}", port.name().unwrap());
    sudoku.send_board(port)?;

    Ok(())
}


fn open_port(dev: &str, baud: u32, sb: StopBits, db: DataBits, p: Parity) -> Result<Box<dyn serialport::SerialPort>> {
    let builder = serialport::new(dev, baud)
                            .stop_bits(sb)
                            .data_bits(db)
                            .parity(p);

    let port = builder.open().with_context(|| format!("Unable to open port {}!", dev))?;

    info!("{}", "Opened Port Successfully!!".green());

    Ok(port)
}

fn main() -> Result<()> {
    let sb: StopBits;
    let db: DataBits;
    let p: Parity;
    let br: u32;

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
            br = args.br;

            match args.sb {
                1 => sb = StopBits::One,
                2 => sb = StopBits::Two,
                _ => bail!("Invalid Stop Bits"),
            }

            match args.db {
                5 => db = DataBits::Five,
                6 => db = DataBits::Six,
                7 => db = DataBits::Seven,
                8 => db = DataBits::Eight,
                _ => bail!("Invalid Data Bits"),
            }

            match args.p {
                MyParity::Even => p = Parity::Even,
                MyParity::Odd => p = Parity::Odd,
                MyParity::None => p = Parity::None,
            }

            let mut port = open_port(args.dev.as_str(), br, sb, db, p)?;

            if let Err(e) = run(args.difficulty, &mut port) {
                error!("{:?}", e);
                std::process::exit(-1);
            }
        }
    }

    Ok(())
}
