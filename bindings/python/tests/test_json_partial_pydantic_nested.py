import pytest
from typing import List
from pydantic import BaseModel

from jsonish import to_json_string  # Adjust the import if necessary.

# Define nested models for the JSON structure.
class Coordinates(BaseModel):
    lat: float
    lng: float

class Address(BaseModel):
    street: str
    city: str
    country: str
    coordinates: Coordinates

class Hobby(BaseModel):
    name: str
    years_active: int
    proficiency: str

class Person(BaseModel):
    name: str
    age: int
    address: Address
    hobbies: List[Hobby]

# Example input: A complex nested JSON wrapped in Markdown fences.
MALFORMED_NESTED_JSON = r"""
This is a nested json you asked for
```json
{
    "name": "Bob",
    "age": 42,
    "address": {
        "street": "789 Pine Rd",
        "city": "Metropolis",
        "country": "USA",
        "coordinates": {"lat": 40.7128, "lng": -74.0060}
    },
    "hobbies": [
        {"name": "Cooking", "years_active": 5, "proficiency": "intermediate"},
        {"name": "Cycling", "years_active": 10, "proficiency": "expert"}
    ]
}
```
"""

# Example input: Valid JSON string (without markdown fences).
VALID_NESTED_JSON = r"""{
    "name": "Bob",
    "age": 42,
    "address": {
        "street": "789 Pine Rd",
        "city": "Metropolis",
        "country": "USA",
        "coordinates": {"lat": 40.7128, "lng": -74.0060}
    },
    "hobbies": [
        {"name": "Cooking", "years_active": 5, "proficiency": "intermediate"},
        {"name": "Cycling", "years_active": 10, "proficiency": "expert"}
    ]
}"""

def test_pydantic_parsing_from_malformed_nested_json():
    """
    Test that the JSON string returned by to_json_string from a nested, malformed input
    can be parsed into a Pydantic model.
    """
    json_str = to_json_string(MALFORMED_NESTED_JSON)
    try:
        # Use model_validate_json for parsing JSON in Pydantic V2.
        person = Person.model_validate_json(json_str)
    except Exception as e:
        pytest.fail(f"Pydantic failed to parse the nested JSON from malformed input: {e}")
    
    # Verify top-level fields.
    assert person.name == "Bob"
    assert person.age == 42
    
    # Verify address fields.
    assert person.address.street == "789 Pine Rd"
    assert person.address.city == "Metropolis"
    assert person.address.country == "USA"
    assert person.address.coordinates.lat == 40.7128
    assert person.address.coordinates.lng == -74.0060

    # Verify hobbies.
    assert len(person.hobbies) == 2
    assert person.hobbies[0].name == "Cooking"
    assert person.hobbies[0].years_active == 5
    assert person.hobbies[0].proficiency == "intermediate"
    assert person.hobbies[1].name == "Cycling"
    assert person.hobbies[1].years_active == 10
    assert person.hobbies[1].proficiency == "expert"

def test_pydantic_parsing_from_valid_nested_json():
    """
    Test that valid nested JSON input processed by to_json_string can be parsed by Pydantic.
    """
    json_str = to_json_string(VALID_NESTED_JSON)
    try:
        person = Person.model_validate_json(json_str)
    except Exception as e:
        pytest.fail(f"Pydantic failed to parse the nested JSON from valid input: {e}")
    
    # Verify top-level fields.
    assert person.name == "Bob"
    assert person.age == 42
    
    # Verify address fields.
    assert person.address.street == "789 Pine Rd"
    assert person.address.city == "Metropolis"
    assert person.address.country == "USA"
    assert person.address.coordinates.lat == 40.7128
    assert person.address.coordinates.lng == -74.0060

    # Verify hobbies.
    assert len(person.hobbies) == 2
    assert person.hobbies[0].name == "Cooking"
    assert person.hobbies[0].years_active == 5
    assert person.hobbies[0].proficiency == "intermediate"
    assert person.hobbies[1].name == "Cycling"
    assert person.hobbies[1].years_active == 10
    assert person.hobbies[1].proficiency == "expert"
