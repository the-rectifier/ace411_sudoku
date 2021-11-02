use sudoku::Sudoku;
use std::str;


pub fn display_sudoku(line: &str) {
    println!();
    for (i, c) in line.chars().enumerate() { 
        print!("|");
        if c == '.' {
            print!("  _  ");
        } else {
            print!("  {}  ", c);
        }
        if ((i+1) % 3 == 0) && (i+1) % 9 != 0 {
            print!("|");
        } else if (i+1) % 9 == 0 && (i+1) % 27 != 0 {
            println!("|");
        } else if (i+1) % 27 == 0 && (i+1) != 81 {
            println!("|\n=========================================================")
        } else if (i+1) == 81 { 
            println!("|");
        }
    }
}


fn get_tx_bytes(sudoku: &[u8; 81], sol: &[u8; 81]) -> Vec<u8> {
    let mut tx_bytes = Vec::new();
    for i in 0..=80 {
        if sudoku[i] == sol[i] {
            continue;
        } else {
            tx_bytes.push(sol[i]);
        }
    }
    return tx_bytes;
}


fn main() {
    let sudoku = Sudoku::generate_unique();
    let sudoku_bytes = sudoku.to_bytes();
    let solution_bytes: [u8; 81];

    display_sudoku(&sudoku.to_str_line());

    let solution = sudoku.solve_unique().unwrap();
    solution_bytes = solution.to_bytes();


    let solution_string = format!("{}", solution);
    display_sudoku(&solution_string.as_str());

    let tx_bytes = get_tx_bytes(&sudoku_bytes, &solution_bytes);
    println!("{:?}", tx_bytes);

}
