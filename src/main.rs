use serialport::{ available_ports, DataBits, StopBits };
use std::io::{self, Write};
use std::time::Duration;
use anyhow::{ Context, Result, bail };
use log::{ info, error };
use simplelog::{ ColorChoice, TermLogger, TerminalMode };

mod sudoku_lib;


fn get_ports() { 
    let ports = available_ports().expect("No ports found!");
    for (i, p) in ports.iter().enumerate() {
        info!("Found Port ({}): {}", i, p.port_name);
    }
}


fn grab_input() -> Result<(usize, usize, u8)> {
    let mut temp_nums: [u8; 3] = Default::default();
    let mut tup: (usize, usize, u8) = Default::default();

    let mut string = String::new();
    io::stdin().read_line(&mut string)
                .with_context(|| format!("Failed to read line"))?;
    
    let inputs: Vec<&str> = string.trim().split(&[' ', ','][..]).collect();

    if inputs.len() == 1 {
        match inputs.get(0) {
            Some(&"s") | Some(&"S") => {return Ok((0, 0, 1));},
            Some(&"q") | Some(&"Q") => {return Ok((0, 0, 2));},
            _ => (),
        };
    }

    if inputs.len() != 3 {
        bail!("Incorrect number of parameters supplied!");
    }

    for (i, num) in inputs.iter().enumerate() {
        let temp: u8 = num.parse::<u8>().with_context(|| format!("Not a valid number!"))?;

        if temp > 9 || temp == 0 {
            bail!("Number ouside valid range!!");
        }
        temp_nums[i] = temp;
    }

    tup.0 = temp_nums[0] as usize -1;
    tup.1 = temp_nums[1] as usize -1;
    tup.2 = temp_nums[2];

    Ok(tup)
}


fn run() -> Result<()> {

    let mut sudoku = sudoku_lib::create_board();
    let mut string = String::new();

    // get_ports();
    println!("Welcome to .....");
    println!("Keep in mind that anything you enter will be sent to the blah blah port");

    loop {
        sudoku.print_unsolved();
        print_menu();

        io::stdin().read_line(&mut string)
                    .with_context(|| format!("Failed to read line"))?;
        
        match string.replace("\n", "").as_str() { 
            "1" => game_mode(&mut sudoku),
            "2" => sudoku.solve_board(),
            "3" => {println!("Goodbyeeee!!");break;}
            _ => error!("Invalid choice!"),
        };
        string.truncate(0);
    }
    Ok(())
}

fn game_mode(sudoku: &mut sudoku_lib::SudokuAvr) {
    print_gamemode();
    loop {
        print!("> ");
        std::io::stdout().flush().expect("some error message");

        match grab_input() {
            Ok(tup) => {
                if tup.0 == 0 && tup.1 == 0 && tup.2 == 1 {
                    sudoku.solve_board();
                    //TODO send solved board
                    sudoku.print_solved();
                } else if tup.0 == 0 && tup.1 == 0 && tup.2 == 2{
                    break;
                } else {
                    sudoku.solve_cell(tup)
                    //TODO send cell
                }
            },
            Err(e) => {
                error!("{:?}", e);
                sudoku.print_unsolved();
            },
        };    
    }
}


fn main() -> Result<()> {
    
    TermLogger::init(
        log::LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ).expect("Failed to init logger");


    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(-1);
    }

    Ok(())
}


fn print_gamemode(){
    println!("Welcome to the Game Mode!!");
    println!("Enter Coordinates and Number separated with commas or spaces: [line, column Num]");
    info!("Enter q/Q to go back to the Main Menu");
    info!("Enter s/S to solve the whole board!");
}


fn print_menu() {
    println!("=============Main Menu=============");
    println!("1.) Enter Game Mode");
    println!("2.) Solve Whole board");
    println!("3.) Exit");
}