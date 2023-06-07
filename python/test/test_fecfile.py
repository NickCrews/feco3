import pytest
import feco3

from . import common


@pytest.mark.parametrize(
    "src",
    [
        common.get_case_path("text.fec"),
        str(common.get_case_path("text.fec")),
    ],
)
def test_create(src):
    fec = feco3.FecFile(src)
    assert fec.header is not None


def test_repr():
    p = common.get_case_path("too_few_fields.fec")
    p_full = repr(p.absolute())
    fec = feco3.FecFile(p)
    assert repr(fec) == f"FecFile(src={p_full})"


@pytest.mark.parametrize(
    "src, header",
    [
        pytest.param(
            "text.fec",
            feco3.Header(
                fec_version="8.1",
                software_name="FECfile",
                software_version="8.1.0.6(f30)",
                report_id="FEC-1119574",
                report_number="2",
            ),
            id="all_fields_present",
        ),
        pytest.param(
            "too_few_fields.fec",
            feco3.Header(
                fec_version="8.3",
                software_name="NGP",
                software_version="8",
                report_id=None,
                report_number=None,
            ),
            id="some_fields_None",
        ),
    ],
)
def test_header(src, header):
    fec = feco3.FecFile(common.get_case_path(src))
    assert fec.header == header
