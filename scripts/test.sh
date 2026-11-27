#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Read only the test configuration; never source .env or execute env-file contents.
exec python3 - "$@" <<'PY'
import os
from pathlib import Path
import shlex
import sys

url = os.environ.get("TEST_DATABASE_URL")
if not url:
    path = Path(".env.test")
    if path.is_file():
        for line in path.read_text().splitlines():
            key, separator, value = line.partition("=")
            if separator and key.strip() == "TEST_DATABASE_URL":
                try:
                    parts = shlex.split(value, comments=True)
                except ValueError:
                    sys.exit("Invalid quoting in .env.test")
                if len(parts) != 1:
                    sys.exit("TEST_DATABASE_URL in .env.test must contain one URL")
                url = parts[0]
if not url:
    sys.exit("Set TEST_DATABASE_URL or configure .env.test using .env.test.example; .env is never used for tests")

environment = os.environ.copy()
environment["TEST_DATABASE_URL"] = url
environment["DATABASE_URL"] = url
# Each integration test binary is linked separately. Limit peak build memory.
environment.setdefault("CARGO_BUILD_JOBS", "1")
os.execvpe("cargo", ["cargo", "test", "--locked", *sys.argv[1:]], environment)
PY
