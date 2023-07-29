import feco3
import pyarrow as pa

# ruff: noqa: E501

# You can supply a URL or a path to a file.
src = "https://docquery.fec.gov/dcdev/posted/1002596.fec"
# src = "path/to/file.fec"
# src = pathlib.Path("path/to/file.fec")

# The straightforward way is to just parse to a directory of files,
# one file for each itemization type, eg "csvs/SA11AI.csv", etc
feco3.FecFile(src).to_csvs("csvs/")
feco3.FecFile(src).to_parquets("parquets/")

# Or, you can look at the file at a lower level.
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

# Iterate through the itemizations in the file in batches of pyarrow RecordBatches.
# By iterating, this keeps us from having to load the entire file into memory.
# By using pyarrow, we can avoid copying the underlying data from Rust to Python.
# It integrates well with the rest of the Python data ecosystem, for example
# it's easy to convert to a pandas DataFrames.
batcher = feco3.PyarrowBatcher(fec, max_batch_size=1024 * 1024)
for batch in batcher:
    # The record code for this kind of itemizations, eg. 'SA11AI'
    assert isinstance(batch.code, str)
    # A pyarrow RecordBatch of the itemizations
    assert isinstance(batch.records, pa.RecordBatch)
    df = batch.records.to_pandas()
    print(batch.code)
    print(df.head(3))
# SA15
#   filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00479188        INCA994                                                          ...
# 1                 C00479188        INCA992                                                          ...
# 2                 C00479188        INCA993                                                          ...

# [3 rows x 44 columns]
# TEXT
#   filer_committee_id_number transaction_id_number back_reference_tran_id_number back_reference_sched_form_name            text
# 0                 C00479188              TPAYC760                       PAYC760                          SC/10  PERSONAL FUNDS
# SC/10
#   filer_committee_id_number transaction_id_number receipt_line_number entity_type  ... lender_candidate_state lender_candidate_district memo_code memo_text_description
# 0                 C00479188               PAYC760                 13B         CAN  ...

# [1 rows x 37 columns]                       ...
