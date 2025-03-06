`json_partial_py` is a resilient JSON parsing library written in Rust that goes beyond the strict JSON specification. It’s designed to parse not only valid JSON but also "JSON‐like" input that may include common syntax errors, multiple JSON objects, or JSON embedded in markdown code blocks.

This is python bindings for `json_partial` library (Rust).

Usage: 

```python
pip install json_partial_python
```

## Simple Example

<pre> 
from pydantic import BaseModel
from json_partial_py import to_json_string

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

json_str = to_json_string(MALFORMED_JSON)
person = Person.model_validate_json(json_str)
print(person)
</pre> 

## Advanced Example

<pre> 
from typing import List
from pydantic import BaseModel
from json_partial_py import to_json_string

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

json_str = to_json_string(MALFORMED_NESTED_JSON)
person = Person.model_validate_json(json_str)
print(person)

</pre> 

One more corner case handled
<pre>
MALFORMED_JSON=r"""
{ rec_one: "and then i said \"hi\", and also \"bye\"", rec_two: "and then i said "hi", and also "bye"", "also_rec_one": ok }
"""

json_str = to_json_string(MALFORMED_NESTED_JSON)
print(json.loads(json_str))

</pre>

## Publishing

To compile to the desired target using Zig, execute the following command:
`maturin build --release --target aarch64-unknown-linux-gnu --zig`
`maturin build --release --target x86_64-unknown-linux-gnu --zig`

After building, move the files from the `target/wheels` directory to the `dist/` folder using the following command:
```bash
cp target/wheels/* dist/
```

Finally, use the command `uv publish` to publish the package.
