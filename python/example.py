import pyarrow as pa
import feco3

# ruff: noqa: E501

# You can supply a URL or a path to a file.
src = "https://docquery.fec.gov/dcdev/posted/1002596.fec"
# src = "path/to/file.fec"
# src = pathlib.Path("path/to/file.fec")

# This doesn't actually read or parse any data yet
fec = feco3.FecFile(src)
print(fec)
# FecFile(src='https://docquery.fec.gov/dcdev/posted/1002596.fec')

# Only when we access something do we actually start parsing.
# Still, we only parse as far as we need to, so this is quite fast.
# This is useful, for example, if you only need the header or cover,
# or if you only want to look at the itemizations in certain forms.
print(fec.header)
print(fec.cover)
# Header(fec_version='8.1', software_name='NetFile', software_version='199199', report_id=None, report_number='0')
# Cover(form_type='F3N', filer_committee_id='C00479188')

# Iterate through the itemizations in the file, yielding pyarrow.RecordBatch objects.
# This keeps us from having to load the entire file into memory.
# By using pyarrow, we can avoid copying the underlying data from Rust to Python.
# It integrates well with the rest of the Python data ecosystem, for example
# it's easy to convert to a pandas DataFrames.
batcher = feco3.PyarrowBatcher(fec, max_batch_size=1024 * 1024)
for batch in batcher:
    assert isinstance(batch, pa.RecordBatch)
    df = batch.to_pandas()
    print(df.head(3))
#   filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00479188        INCA994                                                          ...
# 1                 C00479188        INCA992                                                          ...
# 2                 C00479188        INCA993                                                          ...

# [3 rows x 44 columns]
#   filer_committee_id_number transaction_id_number receipt_line_number entity_type  ... lender_candidate_state lender_candidate_district memo_code memo_text_description
# 0                 C00479188               PAYC760                 13B         CAN  ...

# [1 rows x 37 columns]
#   filer_committee_id_number transaction_id_number back_reference_tran_id_number back_reference_sched_form_name            text
# 0                 C00479188              TPAYC760                       PAYC760                          SC/10  PERSONAL FUNDS                                            ...
