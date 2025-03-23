# Use Stockfish to check correctness of dataset labeling

from random import shuffle
import chess.engine
import chess
import asyncio
from math import exp
import argparse

async def check(file, iterations):
    _, engine = await chess.engine.popen_uci("./stockfish")

    await engine.configure({"Hash": 3000, "Threads": 4})

    dataset = []
    with open(file) as d1:
        for line in d1:
            dataset.append(line)

    shuffle(dataset)

    limit = chess.engine.Limit(nodes=150_000)

    total_mse = 0
    total_samples = 0

    for line in dataset[:iterations]:
        fen = line[:-7]
        wdl = float(line[-5:-2])
        board = chess.Board(fen)

        score = (await engine.analyse(board, limit))["score"].white()

        mate_score = score.mate()
        centipawn_score = score.score()

        if mate_score is not None:
            stockfish_wdl = 1.0 if mate_score > 0 else 0.0
        else:
            stockfish_wdl = 1.0 / (1.0 + exp(-centipawn_score / 400.0))

        mse = (wdl - stockfish_wdl) ** 2
        total_mse += mse
        total_samples += 1

        mean_mse = total_mse / total_samples
        print(f"{file} Mean Squared Error: {mean_mse:.5f}")

    await engine.quit()
    return total_mse / total_samples


async def main():
    parser = argparse.ArgumentParser(description="Use Stockfish to check correctness of dataset labeling.")
    parser.add_argument("paths", type=str, nargs='+', help="Paths to datasets")
    args = parser.parse_args()

    results = await asyncio.gather(*(check(path, 500) for path in args.paths))

    for path, mse in zip(args.paths, results):
        print(f"Error in {path}: {mse}")

asyncio.run(main())
