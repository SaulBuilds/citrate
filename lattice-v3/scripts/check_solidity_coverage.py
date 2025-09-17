#!/usr/bin/env python3
import sys

def parse_lcov(path: str) -> float:
    total = 0
    hit = 0
    with open(path, 'r') as f:
        for line in f:
            if line.startswith('LF:'):
                try:
                    total += int(line.strip().split(':')[1])
                except Exception:
                    pass
            elif line.startswith('LH:'):
                try:
                    hit += int(line.strip().split(':')[1])
                except Exception:
                    pass
    if total == 0:
        return 0.0
    return 100.0 * hit / total

def main():
    if len(sys.argv) < 3:
        print('Usage: check_solidity_coverage.py <lcov.info> <threshold>')
        sys.exit(2)
    path = sys.argv[1]
    threshold = float(sys.argv[2])
    pct = parse_lcov(path)
    print(f"Solidity coverage: {pct:.2f}% (threshold {threshold}%)")
    if pct < threshold:
        print('Coverage below threshold')
        sys.exit(1)

if __name__ == '__main__':
    main()

