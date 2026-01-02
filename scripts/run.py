#!/usr/bin/env python3
import os
import sys
import shutil
import pathlib

CWD: str = str(pathlib.Path(__file__).parent.resolve())
FRONTEND_DIR: str = os.path.abspath(os.path.join(CWD, "..", "frontend"))
BACKEND_DIR: str = os.path.abspath(os.path.join(CWD, "..", "backend"))
BINNAME = "backend.exe" if os.name == "nt" else "backend"

if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] != "-i":
            print("Usage:")
            print("python scripts/run.py to rebuild backend and run the application")
            print("python scripts/run.py -i rebuild backend and run the application with node package preinstallation")
            sys.exit(-1)
        os.system(f"cd {FRONTEND_DIR} && npm install")

    os.system(f"cd {BACKEND_DIR} && cargo build --release")
    native_path = os.path.join(FRONTEND_DIR, "native")
    if not os.path.exists(native_path):
        os.makedirs(native_path)

    shutil.move(
        os.path.join(BACKEND_DIR, f"target/release/{BINNAME}"),
        os.path.join(native_path, BINNAME)
    )

    os.system(f"cd {FRONTEND_DIR} && npm run dev")