# FECo3

Python bindings for a .fec file parser in rust.

Install with `pip install feco3`

## Example

```python
import feco3

fec = feco3.FecFile("path/to/file.fec")
print(fec)
# FecFile("path/to/file.fec")
print(fec.header)
# Header(fec_version='8.3', software_name='FECfile', software_version='8.3.0.0(f32)', report_id=None, report_number=None)
print(fec.cover)
# Cover(form_type='F99', filer_committee_id='C00412569')
```

Read the [API docs](API.md) for more details.
