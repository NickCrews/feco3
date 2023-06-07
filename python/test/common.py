from pathlib import Path

_THIS_DIR = Path(__file__).parent
_REPO_DIR = _THIS_DIR.parent.parent
CASES_DIR = _REPO_DIR / "test/fecs"


def get_case_path(name: str) -> Path:
    return CASES_DIR / name
