import pytest
from app.utils import compare_dicts


def test_compare_dicts_basic():
    dict_A = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5
        }
    }

    dict_B = {
        "a": 1,
        "b": 3,
        "f": 6,
        "c": {
            "d": 4,
            "g": 7
        }
    }

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_A, dict_B)

    assert leftNotRight == ['/c/e']
    assert rightNotLeft == ['/f', '/c/g']
    assert sameKeySameValue == ['/a', '/c/d']
    assert sameKeyDiffValue == ['/b']


def test_compare_dicts_same():
    dict_C = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5
        }
    }

    dict_D = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5
        }
    }

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_C, dict_D)

    assert leftNotRight == []
    assert rightNotLeft == []
    assert sameKeySameValue == ['/a', '/b', '/c/d', '/c/e']
    assert sameKeyDiffValue == []


def test_compare_dicts_diff_keys_in_nested():
    dict_C_1 = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5,
            "f": 6,
            "h": 10
        }
    }

    dict_D_1 = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5,
            "f": 7,
            "g": 10
        }
    }

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_C_1, dict_D_1)

    assert leftNotRight == ['/c/h']
    assert rightNotLeft == ['/c/g']
    assert sameKeySameValue == ['/a', '/b', '/c/d', '/c/e']
    assert sameKeyDiffValue == ['/c/f']


def test_compare_dicts_all_diff():
    dict_E = {
        "a": 1,
        "b": 2,
        "c": {
            "d": 4,
            "e": 5
        }
    }

    dict_F = {
        "g": 6,
        "h": 7,
        "i": {
            "j": 8,
            "k": 9
        }
    }

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_E, dict_F)

    assert leftNotRight == ['/a', '/b', '/c']
    assert rightNotLeft == ['/g', '/h', '/i']
    assert sameKeySameValue == []
    assert sameKeyDiffValue == []


def test_compare_dicts_with_empty():
    dict_G = {
        "0.0.0": {
            "live-reloaded-config": {},
            "rolling-restart-config": {}
        }
    }

    dict_H = {
        "0.0.0": {
            "live-reloaded-config": {},
            "rolling-restart-config": {}
        }
    }

    leftNotRight, rightNotLeft, sameKeySameValue, sameKeyDiffValue = compare_dicts(dict_G, dict_H)

    assert leftNotRight == []
    assert rightNotLeft == []
    assert sorted(sameKeySameValue) == sorted(["/0.0.0/rolling-restart-config", "/0.0.0/live-reloaded-config"])
    assert sameKeyDiffValue == []
