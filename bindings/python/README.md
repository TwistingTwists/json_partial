`json_partial_py` is a resilient JSON parsing library written in Rust that goes beyond the strict JSON specification. It’s designed to parse not only valid JSON but also "JSON‐like" input that may include common syntax errors, multiple JSON objects, or JSON embedded in markdown code blocks.

This is python bindings for `json_partial` library (Rust).

Usage: 

```python
pip install json_partial_py
```

<pre>
from pydantic import BaseModel
from json_partial_py import to_json_string

# Example input: A malformed JSON wrapped in Markdown fences.
MALFORMED_JSON = r"""
```json
{"name": "Bob", "age": 25}
```
"""



# Define a simple Pydantic model that matches the expected JSON structure.
class Person(BaseModel):
    name: str
    age: int

json_str = to_json_string(MALFORMED_JSON) 
person = Person.model_validate_json(json_str)
print(person)

</pre>