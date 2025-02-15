import pytest
from pydantic import BaseModel
from jsonish import to_json_string, to_json_string_pretty

# Define a simple Pydantic model that matches the expected JSON structure.
class Person(BaseModel):
    name: str
    age: int

# Example input: A malformed JSON wrapped in Markdown fences.
MALFORMED_JSON = r"""
```json
{"name": "Bob", "age": 25}
```
"""

# Example input: Valid JSON string.
VALID_JSON = '{"name": "Alice", "age": 30}'

def test_pydantic_parsing_from_malformed_json():
    """
    Ensure that the JSON string returned by to_json_string from a malformed input
    can be successfully parsed into a Pydantic model.
    """
    json_str = to_json_string(MALFORMED_JSON)
    try:
        # Use model_validate_json instead of parse_raw in Pydantic V2.
        person = Person.model_validate_json(json_str)
    except Exception as e:
        pytest.fail(f"Pydantic failed to parse the JSON string from malformed input: {e}")

    assert person.name == "Bob"
    assert person.age == 25

def test_pydantic_parsing_from_valid_json():
    """
    Ensure that valid JSON input processed by to_json_string can be parsed by Pydantic.
    """
    json_str = to_json_string(VALID_JSON)
    try:
        # Use model_validate_json instead of parse_raw in Pydantic V2.
        person = Person.model_validate_json(json_str)
    except Exception as e:
        pytest.fail(f"Pydantic failed to parse the JSON string from valid input: {e}")

    assert person.name == "Alice"
    assert person.age == 30