def get_version() -> str:
    try:
        from importlib.metadata import version
    except ImportError:
        from importlib_metadata import version
    return version("feco3")
