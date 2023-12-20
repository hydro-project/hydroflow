import hydro
import json
from pathlib import Path
from aiostream import stream

async def main(args):
    while True:
        pass

if __name__ == "__main__":
    import sys
    import hydro.async_wrapper
    hydro.async_wrapper.run(main, sys.argv[1:])
