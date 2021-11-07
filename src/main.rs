
use std::io;
use serialport::available_ports;

mod sudoku_lib;

fn get_ports() { 
    let ports = available_ports().expect("No ports found!");
    for (i, p) in ports.iter().enumerate() {
        println!("[*] Found Port ({}): {}", i, p.port_name);
    }
}

fn grab_input() -> (usize, usize, u8) {
    let mut string = String::new();
    io::stdin().read_line(&mut string).expect("Error reading string");

    
    let inputs: Vec<u8> = string.trim().split(' ')
    .map(|x| x.parse().expect("Not an integer!"))
    .collect();

    return (inputs[0] as usize -1, inputs[1] as usize -1, inputs[2]);
}

fn main() {
    let mut sudoku = sudoku_lib::create_board();
    // uart::get_ports();
    let mut tup: (usize, usize, u8);
    println!("UnSolved:");
    sudoku.print_unsolved();

    loop {
        tup = grab_input();
        sudoku.solve_cell(tup);
    }

    
    

}
