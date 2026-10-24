#!/usr/bin/env python3
"""Write an unreviewed identity manifest for a Linux Graphviz GD PNG oracle."""

import argparse
import json
from pathlib import Path

from gd_oracle import identify


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    identity = identify(args.oracle)
    args.output.write_text(json.dumps({**identity, "reviewed": False}, indent=2) + "\n")
    print(f"wrote unreviewed GD PNG identity to {args.output}")


if __name__ == "__main__":
    main()
