import json
import pytest


from jsonish import to_json_string, to_json_string_pretty
# from json_partial import to_json_string, to_json_string_pretty
# from json_partial.jsonish import to_json_string, to_json_string_pretty
# from json_partial.jsonish import to_json_string, to_json_string_pretty

# from json_partial_py import to_json_string, to_json_string_pretty
# from json_partial_py.jsonish import to_json_string, to_json_string_pretty
# from json_partial_py.jsonish import to_json_string, to_json_string_pretty


# This string is taken from your a.py example.
# It contains a JSON code block (with Markdown fences) and a missing comma between the key/value pairs.
MALFORMED_JSON = r"""
Here is your json
```json
{"name": "Bob" , "age": 25}
```
json finishes here
"""

# A simple valid JSON string (without markdown fences)
VALID_JSON = '{"a": 1, "b": 2}'


def test_to_json_string_with_malformed_input():
    """
    Test that to_json_string() returns a valid JSON string even when given a malformed input.
    In this case, we expect the function to strip Markdown fences and fix the missing comma.
    """
    result = to_json_string(MALFORMED_JSON)
    # Ensure we got a string back.
    assert isinstance(result, str)

    # Try parsing the result to verify it is valid JSON.
    try:
        data = json.loads(result)
    except Exception as e:
        pytest.fail(f"to_json_string output is not valid JSON: {e}")

    # Verify that the parsed JSON has the expected keys and values.
    # (Adjust these expectations if your function should behave differently.)
    assert data.get("name") == "Bob"
    assert data.get("age") == 25


def test_to_json_string_pretty_with_malformed_input():
    """
    Test that to_json_string_pretty() returns a pretty-printed valid JSON string from the malformed input.
    """
    result = to_json_string_pretty(MALFORMED_JSON)
    assert isinstance(result, str)

    # Check that the pretty output is valid JSON.
    try:
        data = json.loads(result)
    except Exception as e:
        pytest.fail(f"to_json_string_pretty output is not valid JSON: {e}")

    # Verify expected content.
    assert data.get("name") == "Bob"
    assert data.get("age") == 25

    # Check that the result contains newlines and indentation (pretty printing)
    assert "\n" in result
    # Optionally check for at least two-space indent (this depends on your implementation).
    assert "  " in result


def test_valid_json_input():
    """
    When valid JSON is provided (without Markdown fences or errors),
    both functions should return a JSON string that parses to the same data.
    """
    result_normal = to_json_string(VALID_JSON)
    result_pretty = to_json_string_pretty(VALID_JSON)

    try:
        data_normal = json.loads(result_normal)
        data_pretty = json.loads(result_pretty)
    except Exception as e:
        pytest.fail(f"Valid JSON input did not produce valid JSON output: {e}")

    expected = {"a": 1, "b": 2}
    assert data_normal == expected
    assert data_pretty == expected


def test_consistency_between_functions():
    """
    Ensure that both functions produce output that, when parsed, results in the same Python object.
    """
    data_normal = json.loads(to_json_string(MALFORMED_JSON))
    data_pretty = json.loads(to_json_string_pretty(MALFORMED_JSON))
    assert data_normal == data_pretty