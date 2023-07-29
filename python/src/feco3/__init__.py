"""FECo3: Python bindings to a .fec file parser written in Rust."""

from __future__ import annotations

import os
from functools import cached_property
from pathlib import Path
from typing import TYPE_CHECKING, NamedTuple

from . import _feco3, _version

if TYPE_CHECKING:
    import pyarrow as pa

__version__ = _version.get_version()
"""Version string for this package."""


class Header(NamedTuple):
    """The header of a [FecFile][feco3.FecFile].

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


class Cover(NamedTuple):
    """The Cover Line of an [FecFile][feco3.FecFile].

    Attributes:
        form_type: The form type of the filing, eg. "F3"
        filer_committee_id: The FEC-assigned ID of the committee that filed the report,
            eg "C00618371"
    """

    form_type: str
    filer_committee_id: str


class FecFile:
    """An FEC file."""

    def __init__(self, src: str | os.PathLike) -> None:
        """Create a new FecFile.

        This doesn't do any reading or parsing until you access one of the members.

        Args:
            src: A path or a URL to an FEC file.
                If a string that starts with "http://" or "https://", it will be
                treated as a URL. Otherwise, it will be treated as a path.
        """
        if isinstance(src, str) and (
            src.startswith("http://") or src.startswith("https://")
        ):
            self._src = src
            self._wrapped = _feco3.FecFile.from_https(self._src)
        else:
            self._src = Path(src)
            self._wrapped = _feco3.FecFile.from_path(self._src)

    @cached_property
    def header(self) -> Header:
        """The [Header][feco3.Header] of the FEC file.

        The first time this is accessed, the FEC file will be read and parsed as
        far as needed. Subsequent accesses will return the same object.
        """
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
        """The [Cover][feco3.Cover] of the FEC file.

        The first time this is accessed, the FEC file will be read and parsed as
        far as needed. Subsequent accesses will return the same object.
        """
        c = self._wrapped.cover
        return Cover(
            form_type=c.form_type,
            filer_committee_id=c.filer_committee_id,
        )

    def to_parquets(self, out_dir: str | os.PathLike) -> None:
        """Write all itemizations in this FEC file to parquet files.

        There will be one parquet file for each record type, eg. ``sa11.parquet``.
        """
        parser = _feco3.ParquetProcessor(out_dir)
        parser.process(self._wrapped)

    def to_csvs(self, out_dir: str | os.PathLike) -> None:
        """Write all itemizations in this FEC file to CSV files.

        There will be one CSV file for each record type, eg. ``sa11.csv``.
        """
        parser = _feco3.CsvProcessor(out_dir)
        parser.process(self._wrapped)

    def __repr__(self) -> str:
        src_str = f"src={self._src!r}"
        return f"{self.__class__.__name__}({src_str})"


# This is what rust parquet uses as a batch size
# https://docs.rs/parquet/40.0.0/src/parquet/file/properties.rs.html#83
DEFAULT_PYARROW_RECORD_BATCH_MAX_SIZE = 1024 * 1024


class ItemizationBatch(NamedTuple):
    """A batch of itemizations.

    Attributes:
        code: The code of the itemization type, eg. "SA11AI"
        records: A [pyarrow.RecordBatch][pyarrow.RecordBatch] of itemizations.
    """

    code: str
    records: pa.RecordBatch


class PyarrowBatcher:
    """
    Iterates an [FecFile](feco3.FecFile) and yields [ItemizationBatch](feco3.ItemizationBatch)s of itemizations.
    """  # noqa: E501

    def __init__(self, fec_file: FecFile, max_batch_size: int | None = None) -> None:
        """Create a new PyarrowBatcher.

        Args:
            fec_file: The [FecFile][feco3.FecFile] to iterate.
            max_batch_size: The max rows per [pyarrow.RecordBatch][pyarrow.RecordBatch].
                Defaults to 1024 * 1024, which is what rust parquet uses.
        """
        self._fec_file = fec_file
        if max_batch_size is None:
            max_batch_size = DEFAULT_PYARROW_RECORD_BATCH_MAX_SIZE
        self._wrapped = _feco3.PyarrowBatcher(max_batch_size)

    def __iter__(self) -> PyarrowBatcher:
        return self

    def __next__(self) -> ItemizationBatch:
        """Get the next batch of itemizations from the FEC file."""
        pair = self._wrapped.next_batch(self._fec_file._wrapped)
        if pair is None:
            raise StopIteration
        code, batch = pair
        return ItemizationBatch(code, batch)
