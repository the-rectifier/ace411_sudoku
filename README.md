# Sudoku UART Interface (for AVR)

## TODO

- [ ] Add UART functionality
  - [X] Send Unsolved Board
  - [ ] Recieve Solved Board and Check
  - [X] Await Responses Each time a write is issued
- [X] Cross Platform UART Ports ([serialport-rs](https://github.com/Susurrus/serialport-rs))
- [X] Add Difficulty Levels [Easy, Medium, Hard]
  - Randomly Remove Cells (Remaining Uniquely Solvable)
- [X] Create Boards
  - [X] Printing
  - [X] Solving
- [X] Explicit Board Generation
- [ ] Explicit Board Download to AVR
- [ ] Calculate Time until Solution arrives
- [X] Argument Passing
- [ ] Anything else I find down the road
- [ ] Update README
