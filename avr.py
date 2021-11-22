from machine import UART, Pin

import time

def print_board(board):
    for i in range(0,9):
        print(board[i])

board = [[0 for i in range(0,9)] for i in range(0,9)]

OK = b"OK\r\n"

alice = UART(0, baudrate=9600, tx=Pin(0), rx=Pin(1))

print("Sniffing Started")
while True:
    if alice.any():
        chunk = alice.read()
    else:
        continue
    print_board(board)
    print(f"Alice read: {chunk}")
    if chunk == b"AT\r\n":
        print("Got AT")
        alice.write(OK)
    elif chunk == b"C\r\n":
        print("Got Clear")
        alice.write(OK)
    elif chunk == b"B\r\n":
        print("Got Break")
        alice.write(OK)
    elif chunk == b"P\r\n":
        print("Got Play")
        alice.write(OK)
        time.sleep(2)
        alice.write(b"D\r\n")
    elif chunk[0] == ord('N') and chunk[4] == ord('\r') and chunk[5] == ord('\n'):
        print("Got Number")
        board[chunk[2]][chunk[1]] = chunk[3]
        alice.write(OK)
    elif chunk[0] == ord('D') and chunk[3] == ord('\r') and chunk[4] == ord('\n'):
        print("Got Debug")
        alice.write(f"N{chunk[1]}{chunk[2]}{board[chunk[1]-1][chunk[2]-1]}\r\n".encode())
    elif chunk == b"S\r\n":
        print("Got Save")
        for i in range(0,9):
            for j in range(0,9):
                print("Sending Cell!")
                alice.write(f"N{i+1}{j+1}{board[i][j]}\r\n".encode())
                while True:
                    while not alice.any():
                        continue
                    if alice.read() == b"T\r\n":
                        break
                    else:
                        continue
        alice.write(b"D\r\n")
                    
    elif chunk == b"OK\r\n":
        print("Got OK from PC")
                


        
        

        