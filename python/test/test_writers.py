from pathlib import Path

import feco3
import pyarrow as pa

from . import common


def test_pyarrow_batches():
    path = common.get_case_path("slash_form.fec")
    fec = feco3.FecFile(path)
    batcher = feco3.PyarrowBatcher(fec)
    # Can convert to list
    batches = list(batcher)
    assert len(batches) > 1
    seen_codes = set()
    for b in batches:
        assert isinstance(b, feco3.ItemizationBatch)
        assert isinstance(b.code, str)
        assert isinstance(b.records, pa.RecordBatch)
        assert b.records.num_rows > 0
        assert b.records.num_columns > 0
        seen_codes.add(b.code)

    assert seen_codes == {"SA11AI", "SD10", "SC2/10", "SC/10", "SB17"}

    # We have used up the fec file, so iterating again finds no itemizations
    assert list(feco3.PyarrowBatcher(fec)) == []


def test_csvs(tmp_path: Path):
    path = common.get_case_path("slash_form.fec")
    fec = feco3.FecFile(path)
    fec.to_csvs(tmp_path)
    assert len(list(tmp_path.glob("*.csv"))) == 5


def test_parquets(tmp_path: Path):
    path = common.get_case_path("slash_form.fec")
    fec = feco3.FecFile(path)
    fec.to_parquets(tmp_path)
    assert len(list(tmp_path.glob("*.parquet"))) == 5
