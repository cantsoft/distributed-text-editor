import os
import sys
import pathlib

CWD: str = pathlib.Path(__file__).parent.resolve()
FRONTEND_DIR: str = os.path.abspath(os.path.join(CWD, "..", "frontend"))

if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] != "-i":
            print("Usage:")
            print("python scripts/run.py to rebuild backend and run the application")
            print("python scripts/run.py -i rebuild backend and run the application with node package preinstallation")
            sys.exit(-1)
        os.system(f"cd {FRONTEND_DIR} && npm install")
    os.system(f"cd {FRONTEND_DIR} && npm run build:backend && npm run dev")