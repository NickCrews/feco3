"""FECo3: Python bindings to a .fec file parser written in Rust."""

from __future__ import annotations
from dataclasses import dataclass
from functools import cached_property

import os
from typing import TYPE_CHECKING


from . import _version
from . import _feco3

if TYPE_CHECKING:
    import pyarrow as pa

__version__ = _version.get_version()


@dataclass
class Header:
    """The header of an ``FecFile``.

    Attributes:
        fec_version: The version of the FEC file format.
        software_name: The name of the software that generated the file.
        software_version: The version of the software that generated the file.
            This isn't present in some older FEC files.
        report_id: If this .fec file is an amendment to a previous filing,
            the filing number of the original.
        report_number: If this .fec file is an amendment to a previous filing,
            which number amendement this is (1, 2, 3 etc)
    """
    fec_version: str
    software_name: str
    software_version: str | None
    report_id: str | None
    report_number: str | None


@dataclass
class Cover:
    """The Cover Line of an ``FecFile``.

    Attributes:
        form_type: The form type of the filing, eg. "F3"
        filer_committee_id: The FEC-assigned ID of the committee that filed the report,
            eg "C00618371"
    """
    form_type: str
    filer_committee_id: str


class FecFile:
    """An FEC file.

    Attributes:
        header: The header of the FEC file.
            The first time this is accessed, the FEC file will be read and parsed as
            far as needed.
        cover: The cover of the FEC file.
            The first time this is accessed, the FEC file will be read and parsed as
            far as needed.
    """

    def __init__(self, src: str | os.PathLike) -> None:
        """Create a new FecFile.

        This doesn't do any reading or parsing until you access one of the members.

        Args:
            src: The path to the FEC file to parse.
        """
        self._src = src
        self._wrapped = _feco3.FecFile.from_path(src)

    @cached_property
    def header(self) -> Header:
        h = self._wrapped.header
        return Header(
            fec_version=h.fec_version,
            software_name=h.software_name,
            software_version=h.software_version,
            report_id=h.report_id,
            report_number=h.report_number,
        )

    @cached_property
    def cover(self) -> Cover:
        c = self._wrapped.cover
        return Cover(
            form_type=c.form_type,
            filer_committee_id=c.filer_committee_id,
        )

    def to_parquet(self, out_dir: str | os.PathLike) -> None:
        """Write all itemizations in this FEC file to parquet files.

        There will be one parquet file for each record type, eg. ``sa11.parquet``.
        """
        parser = _feco3.ParquetProcessor(out_dir)
        parser.process(self._wrapped)

    def __repr__(self) -> str:
        src_str = f"src={self._src!r}"
        return f"{self.__class__.__name__}({src_str})"


# This is what rust parquet uses as a batch size
# https://docs.rs/parquet/40.0.0/src/parquet/file/properties.rs.html#83
# DEFAULT_PYARROW_RECORD_BATCH_MAX_SIZE = 1024 * 1024
DEFAULT_PYARROW_RECORD_BATCH_MAX_SIZE = 1024 * 1024


class PyarrowProcessor:
    """
    Iterates an [feco3.FecFile][] and yields [pyarrow.RecordBatch][]s of itemizations.
    """
    def __init__(self, max_batch_size: int | None = None):
        if max_batch_size is None:
            max_batch_size = DEFAULT_PYARROW_RECORD_BATCH_MAX_SIZE
        self._wrapped = _feco3.PyarrowProcessor(max_batch_size)

    def next_batch(self, fec_file: FecFile) -> pa.RecordBatch:
        """Get the next batch of itemizations from the FEC file."""
        return self._wrapped.next_batch(fec_file._wrapped)
