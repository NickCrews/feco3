# FECo3

Python bindings for a `.fec` file parser in rust.

Install with `pip install feco3`

- [API docs](https://nickcrews.github.io/feco3/)
- [Repository](https://github.com/NickCrews/feco3)

## Example

```python
import urllib.request
import pyarrow as pa
from pathlib import Path
import feco3
# ruff: noqa: E501

path = Path("1572941.fec")
if not path.exists():
    url = f"https://docquery.fec.gov/dcdev/posted/{path.name}"
    # Without these headers, the FEC server returns a 500 error
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with open(path, "wb") as f:
        f.write(urllib.request.urlopen(req).read())

# This doesn't actually read or parse any data yet
fec = feco3.FecFile(path)
print(fec)
# FecFile("path/to/file.fec")

# Only when we access something do we actually start parsing.
# Still, we only parse as far as we need to, so this is quite fast.
# This is useful, for example, if you only need the header or cover,
# or if you only want to look at the itemizations in certain forms.
print(fec.header)
print(fec.cover)
# Header(fec_version='8.3', software_name='FECfile', software_version='8.3.0.0(f32)', report_id=None, report_number=None)
# Cover(form_type='F99', filer_committee_id='C00412569')

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
# filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00792689     SA11C.4481                                                          ...
# 1                 C00792689     SA11C.4483                                                          ...
# 2                 C00792689     SA11C.4477                                                          ...
#
# [3 rows x 44 columns]
#   filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00792689    SA11AI.4406                                                          ...
# 1                 C00792689    SA11AI.4286                                                          ...
# 2                 C00792689    SA11AI.4288                                                          ...
#
# [3 rows x 44 columns]
#   filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00792689     SA11D.4355                                                          ...
#
# [1 rows x 44 columns]
#   filer_committee_id_number transaction_id_number back_reference_tran_id_number back_reference_sched_form_name                                               text
# 0                 C00792689       SA11C.4481.TEXT                    SA11C.4481                          SA11C  When the Brice Wiggins for Congress campaign c...
# 1                 C00792689       SA11C.4483.TEXT                    SA11C.4483                          SA11C  When the Brice Wiggins for Congress campaign c...
# 2                 C00792689       SA11C.4477.TEXT                    SA11C.4477                          SA11C  When the Brice Wiggins for Congress campaign c...
#   filer_committee_id_number transaction_id_number  ... memo_text_description reference_to_si_or_sl_system_code_that_identifies_the_account
# 0                 C00792689             SB17.4353  ...
# 1                 C00792689             SB17.4476  ...
# 2                 C00792689             SB17.4482  ...
#
# [3 rows x 43 columns]
#
```
