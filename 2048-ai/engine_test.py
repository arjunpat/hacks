import unittest
from engine import Board

class TestBoard(unittest.TestCase):
  def test_board_changed(self):
    b = Board()

    b.board = [   0,    0,    0,    2,
    0,    0,    2,    4,
    0,    0,    4,   16,
    2,    0,    8,   32,]

    assert b.make_move('D') == (False, False)

  def test_game_over(self):
    b = Board()

    b.board = [2,   4,   2,  16,
      16,   2,   8,  32,
      4,  64,  32,  16,
      2,   8,   2,   4]
    
    assert b.is_game_over()


if __name__ == "__main__":
  unittest.main()
