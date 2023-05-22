import feco3


def test_sum_as_string():
    assert feco3.sum_as_string(1, 2) == "3"


def test_add_42():
    assert feco3.add_42(1) == 43
