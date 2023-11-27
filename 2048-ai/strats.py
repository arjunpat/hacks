from engine import Board
import random
import numpy as np


def play_random_game(loud=False) -> Board:
    b = Board()
    moves = ["R", "D", "U", "L"]
    game_over = False
    while not game_over:
        move = random.choice(moves)
        # print('move:', move)
        _, game_over = b.make_move(move, loud=loud)
        # print(b)

    return b


def play_strat_a(loud=False) -> Board:
    """Move down or right until stuck, then randomly choose left or up"""
    b = Board()

    board_changed, game_over = True, False
    while not game_over:
        move = random.choice([0, 1])
        board_changed, game_over = b.make_move(["D", "R"][move], loud=loud)

        if not board_changed and not game_over:
            # try other move in board didnt change
            board_changed, game_over = b.make_move(
                ["D", "R"][(move + 1) % 2], loud=loud
            )

            # try up if still not changed
            if not board_changed and not game_over:
                board_changed, game_over = b.make_move("U", loud=loud)

                # try left
                if not board_changed and not game_over:
                    board_changed, game_over = b.make_move("L", loud=loud)

    return b


if __name__ == "__main__":
    # total_moves = 0
    # max_reached = []
    # score_reached = []
    # for i in range(10000):
    #   if i % 50 == 0:
    #     print("Iter:", i)
    #   b = play_strat_a()

    #   max_reached.append(b.get_max())
    #   score_reached.append(b.get_score())
    #   total_moves += b.num_moves()

    # max_reached = np.array(max_reached)
    # score_reached = np.array(score_reached)

    # unique, counts = np.unique(max_reached, return_counts=True)

    # # prints out the average score reached for each bucket of max_reached
    # for i in range(len(unique)):
    #   print(unique[i], np.mean(score_reached[max_reached == unique[i]]))

    # print("Unique:", unique)
    # print("Counts:", counts)
    # print("Total Moves:", total_moves)

    play_random_game(loud=True)
