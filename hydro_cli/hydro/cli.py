import hydro._core # type: ignore
import sys

def cli():
    try:
        hydro._core.cli.cli_entrypoint(sys.argv)
        return 0
    except Exception:
        return 1
