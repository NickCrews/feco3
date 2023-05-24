import os

import _feco3


def parse(fec_path: str | os.PathLike, out_dir: str | os.PathLike) -> None:
    """Parse a FEC file and output the results to a directory.

    Args:
        fec_path: The path to the FEC file to parse.
        out_dir: The directory to output the results to.
    """
    _feco3.parse_from_path(fec_path, out_dir)
