import os
import sys
from engine import Board

if __name__ == "__main__":
    b = Board()
    print(b)
    keymap = {"w": "U", "a": "L", "s": "D", "d": "R"}
    while True:
        key = sys.stdin.read(1)
        if key in keymap.keys():
            board_changed, game_over = b.make_move(keymap[key])
            os.system("clear")
            print(b)
