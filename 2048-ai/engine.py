import random
from typing import Callable


class Board:
    """Board will look like the following:
    [
        0,0,0,0,
        0,0,0,0,
        0,0,0,0,
        0,0,0,0,
    ]

    0 means there is nothing there
    all numbers are positive
    """

    def __init__(self):
        self.game_over: bool = False
        self.move_num: int = 0
        self.score: int = 0

        # make board
        self.board: list[int] = [0 for _ in range(16)]

        idxs = list(range(16))
        random.shuffle(idxs)
        for each in idxs[:2]:
            if random.random() > 0.8:
                self.board[each] = 4
            else:
                self.board[each] = 2

    def make_move(self, dir: str, loud: bool = False) -> tuple[bool, bool]:
        """Returns a tuple where first value is whether the board changed during this move; second value
        is whether the game is over.
        """
        assert dir in ("U", "D", "L", "R") or True

        if self.game_over:
            raise Exception("Cannot move after game is over")

        self.move_num += 1
        board_changed, score_increase = self._make_move(self.board, dir)
        self.score += score_increase

        # add a random 2 or 4 to the board
        zero_pos = [i for i in range(16) if self.board[i] == 0]
        if len(zero_pos) != 0 and board_changed:
            random_zero = random.choice(zero_pos)

            if random.random() > 0.8:
                self.board[random_zero] = 4
            else:
                self.board[random_zero] = 2

        if len(zero_pos) == 1 and board_changed:
            # just placed last square
            self.game_over = self.is_game_over()

        if loud:
            print("Move:", dir)
            print(self)

        return board_changed, self.game_over

    def _make_move(self, board: list[int], dir: str) -> tuple[bool, int]:
        results = [self.handle_row(board, self.make_idx_func(dir, i)) for i in range(4)]
        results = list(zip(*results))
        return any(results[0]), sum(results[1])

    def is_game_over(self):
        zero_pos = [i for i in range(16) if self.board[i] == 0]
        if len(zero_pos) > 0:
            return False

        # want to check if any move can be made
        for move in ("U", "D", "L", "R"):
            board_copy = self.board.copy()
            board_changed, _ = self._make_move(board_copy, move)

            if board_changed:
                return False

        return True

    def num_moves(self) -> int:
        return self.move_num

    def is_done(self) -> bool:
        return self.game_over

    def get_max(self) -> int:
        return max(self.board)

    def get_score(self) -> int:
        return self.score

    def make_idx_func(self, dir: str, i: int) -> Callable[[int], int]:
        assert dir in ("U", "D", "L", "R")
        assert i >= 0 and i < 4

        def idx(j: int) -> int:
            assert j >= 0 and j < 4

            if dir == "R":
                return 4 * i + j
            elif dir == "D":
                return 4 * j + i
            elif dir == "L":
                return 4 * i + (3 - j)
            elif dir == "U":
                return 4 * (3 - j) + i

        return idx

    def handle_row(
        self, board: list[int], idx_func: Callable[[int], int]
    ) -> tuple[bool, int]:
        """Returns tuple with two values.
        First value is whether the board changed.
        Second value is the score increase by this movement on this row.
        """
        assert len(board) == 16
        idx = [idx_func(i) for i in range(4)]

        row_sum = sum([board[idx[i]] for i in range(4)])
        if row_sum == 0:
            return False, 0
        board_changed = False

        # first move everything to the right
        shift_right = 0
        for i in range(3, -1, -1):
            if board[idx[i]] == 0:
                shift_right += 1
            elif shift_right != 0:
                board_changed = True
                board[idx[i + shift_right]] = board[idx[i]]
                board[idx[i]] = 0

        score_increase = 0
        # now combine similar values
        for i in range(3, 0, -1):
            if board[idx[i]] == 0:
                break
            elif board[idx[i]] == board[idx[i - 1]]:
                board[idx[i]] *= 2
                score_increase += board[idx[i]]
                board[idx[i - 1]] = 0
                for j in range(i - 2, -1, -1):
                    if board[idx[j]] != 0:
                        board[idx[j + 1]] = board[idx[j]]
                        board[idx[j]] = 0

        # assert the sum of the row didn't change
        assert row_sum == sum([board[idx[i]] for i in range(4)])

        # # assert all the zeros are at the beginning
        num_zeros = sum([1 for i in range(4) if board[idx[i]] == 0])

        for i in range(num_zeros):
            assert board[idx[i]] == 0

        return board_changed or score_increase != 0, score_increase

    def __str__(self) -> str:
        result = f"--- Board State (Move {self.move_num}; Score: {self.score}) ---\n"
        for i in range(4):
            for j in range(4):
                result += "{:4}".format(self.board[4 * i + j])
            result += "\n"

        return result
