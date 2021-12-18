# Sudoku UART Interface (for AVR)

## Author: Odysseas Stavrou (canopus)

## Prof: A. Dollas

The course on Embedded Systems (ACE411) at TU Crete had us develop a Sudoku Solver on an ATMega16 (Using the STK-500). So I wrote a small frontend / interface in Rust to interact with the STK-500 over UART.

### Why

Because I wanted to learn more about Rust and I was definitely not going to be sending 81 Cells by hand using Putty

---

The Interface has the following functionality:

- Generate Uniquely Solvable Sudoku Boards using the [sudoku](https://crates.io/crates/sudoku) Crate
- Solve the board and remove Cells in order to create Difficulty Levels (whilst remaining Uniquely Solvable)
- Bulk Board Generation
- Download a board to the STK-500
- Receive Board and check solution
- Measure Time until receiving "Solve" signal
- Can Drop you into Interactive Shell suporting a specific Hayes Command Set
- Cross Platform (Windows / Linux) using the [serialport-rs](https://crates.io/crates/serialport) Crate
  - Variable UART Configurations (Data bits, Stop bits, etc) Provided by the Crate
- Export Current Board with time to solve

The interface has 4 modes:

- gen (Bulk Generate Boards)
- prog (Program a created board to the STK-500 and optionally drop into interactive shell)
- run (Same as above but generates a board at Runtime, drops into interactive shell afterwords)
- list (List available UART Ports)

The Hayes Command Set

| Command | From | To | Response | Description |
|-|-|-|-|-|
|AT|PC|AVR|OK|Attention|
|C|PC|AVR|OK|Clear|
|P|PC|AVR|OK|Start Solving|
|D*|AVR|PC|S|AVR Done Solving|
|T|PC|AVR|N\<X>\<Y>\<NUM>|AVR sends the contents of X,Y cell, PC replies with T to receive the next one|
|D*|AVR|PC|OK|AVR Done Sending Solved Board|
|B|PC|AVR|OK|AVR Stops any calculations|
|D\<X>\<Y>|PC|AVR|N\<X>\<Y>\<NUM>|AVR returns contents of X,Y Cell

Note that each Command and Response have **"\r\n"** in the end.

---

### Releases

You can find the latest release on the [Release Tab](https://github.com/the-rectifier/ace411_sudoku/releases).

Or you can [Build](#Build) it yourself.

### Usage

- Available commands:

 ```bash
 $ ./ace411_sudoku -h

 ace411_sudoku 0.1.0

 USAGE:
  ace411_sudoku <SUBCOMMAND>

 FLAGS:
  -h, --help       Prints help information
  -V, --version    Prints version information

 SUBCOMMANDS:
  gen     Generate Boards
  help    Prints this message or the help of the given subcommand(s)
  list    Lists Available Serial Ports
  prog    Download Board to MCU
  run     Run Mode
 ```

- List Available boards:

 ```bash
 ace411_sudoku list
 ```

- Bulk Generate boards:

 ```bash
 $ ace411_sudoku gen -h
 Generate Boards

 USAGE:
  ace411_sudoku gen --directory <directory> --number <number>

 FLAGS:
  -h, --help       Prints help information
  -V, --version    Prints version information

 OPTIONS:
  -d, --directory <directory>    Directory to place the Boards
  -n, --number <number>          Generate <number> boards for EACH
            difficulty level
 ```

- Download a Board to STK-500:

 ```bash
 $ ace411_sudoku prog -h
 Download Board to MCU

 USAGE:
 ace411_sudoku prog [FLAGS] [OPTIONS] --board-file <board> --baud-rate <br> --dev <dev>

 FLAGS:
  -h, --help           Prints help information
  -i, --interactive    Enter Interactive shell
  -V, --version        Prints version information

 OPTIONS:
  -b, --board-file <board>    Board file to download
  -r, --baud-rate <br>        Baudrate
   --data-bits <db>        Data Bits [default: 8]  [possible values: 5, 6, 7, 8]
  -u, --dev <dev>             Device Port
  -p, --parity <p>            Parity [default: None]
   --stop-bits <sb>        Stop Bits [default: 1]  [possible values: 1, 2]
 ```

- Run:

 ```bash ./ace411_sudoku run -h
 Run Mode

 USAGE:
  ace411_sudoku run [OPTIONS] --baud-rate <br> --dev <dev> --difficulty <difficulty>

 FLAGS:
  -h, --help       Prints help information
  -V, --version    Prints version information

 OPTIONS:
  -r, --baud-rate <br>             Baudrate
   --data-bits <db>             Data Bits [default: 8]  [possible values: 5, 6, 7, 8]
  -u, --dev <dev>                  Device Port
  -d, --difficulty <difficulty>    Difficulty of Game [possible values: Easy, Medium, Hard, Ultra]
  -p, --parity <p>                 Parity [default: None]
   --stop-bits <sb>             Stop Bits [default: 1]  [possible values: 1, 2]
 ```

- Example Usage:
- Download **Easy_1.txt** board to an STK-500 in **/dev/ttyUSB0** with a baudrate of **9600** and then drop into an interactive shell:
`./ace411_sudoku -b Easy_1.txt -u /dev/ttyUSB0 -r 9600 -i`
- Same as above but with Parity:
- `./ace411_sudoku -b Easy_1.txt -u /dev/ttyUSB0 -r 9600 -i -p Odd`

---

### Build

Building is only supported in Linux due to required packages by the [serialport-rs](https://crates.io/crates/serialport) Crate (See [Dependencies](#Dependencies))

- Install [Rust](https://www.rust-lang.org/learn/get-started) using rustup
- Add windows target:
  - `$ rustup target add x86_64-pc-windows-gnu`
- Clone this repo
- `$ cargo make sudoku`
- Binaries will be available under `out` directory
- Profit?

### Dependencies

The serialport-rs Crate requires `pkg-config` and `libudev` headers to be installed

---

### TODO

- [X] Add UART functionality

- [X] Send Unsolved Board

- [X] Receive Solved Board and Check

- [X] Await Responses Each time a write is issued

- [X] Interactive Shell Conforming to the Hayes Command Set

- [X] Replies when Necessary

- [X] Help Menu For Shell

- [X] Cross Platform UART Ports ([serialport-rs](https://github.com/Susurrus/serialport-rs))

- [X] Add Difficulty Levels [Easy, Medium, Hard, Ultra]

- [X] Randomly Remove Cells (Remaining Uniquely Solvable)

- [X] Create Boards

- [X] Printing

- [X] Solving

- [X] Explicit Board Generation

- [X] Explicit Board Download to AVR

- [X] Calculate Time until Solution arrives

- [X] Argument Passing

- [ ] Anything else I find down the road

- [ ] Comments & Documentation

- [X] Update README
