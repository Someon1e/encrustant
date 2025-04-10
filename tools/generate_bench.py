import random
import argparse

def pick_random_lines(file_path, num_lines):
    with open(file_path, 'r') as file:
        lines = file.readlines()

    if len(lines) < num_lines:
        raise ValueError("The file contains fewer lines than the number requested.")

    return random.sample(lines, num_lines)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Randomly take positions from a dataset for bench")
    parser.add_argument("-f", "--file", required=True, type=str, help="Path to dataset")
    parser.add_argument("-c", "--count", required=True, type=int, help="Number of positions to take")

    parser.add_argument("-low", required=True, type=int, help="Minimum depth")
    parser.add_argument("-high", required=True, type=int, help="Maximum depth")

    args = parser.parse_args()

    selected_lines = pick_random_lines(args.file, args.count)

    if selected_lines:
        for line in selected_lines:
            r = random.randint(args.low, args.high)
            print('("' + line[:-7] + f'", {r}),')
