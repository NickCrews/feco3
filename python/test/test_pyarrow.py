import feco3

from . import common
import pyarrow as pa


def test_pyarrow_batches():
    path = common.get_case_path("slash_form.fec")
    fec = feco3.FecFile(path)
    batcher = feco3.PyarrowBatcher(fec)
    # Can convert to list
    batches = list(batcher)
    assert len(batches) > 1
    for b in batches:
        assert isinstance(b, pa.RecordBatch)
        assert b.num_rows > 0
        assert b.num_columns > 0

    # We have used up the fec file, so iterating again finds no itemizations
    assert list(feco3.PyarrowBatcher(fec)) == []
