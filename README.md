# CHATBOT-IO CLI

Currently this tool is just a convenience reading files and template pre-processing. Essentially string interpolation (via liquid template files) for formatting file contents in such a way that you can quickly paste the contents into a chatbot UI.

It accepts a list of files, a liquid template file path, and will compile the liquid template into a string that will be printed to `STDOUT`.

A basic example template:
````liquid
# Source Files
{% for file in files %}
## `{{ file.name }}`:
```
{{ file.contents }}
```
{% endfor %}
````

This is the JSON schema of the liquid template environment:
```json
{
  "type": "object",
  "properties": {
    "files": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "path": { "type": "string" },
          "contents": { "type": "string" }
        }
      }
    }
  }
}
```


# Example

**The following command:**

```shell
$ cargo run -- format --input test/source-1/*.py --template test/source-1/template.liquid
```

Or if you have installed `chatbot-io`:
```shell
$ chatbot-io format --input test/source-1/*.py --template test/source-1/template.liquid
```
> Note that you may install `chatbot-io` via cargo like so:
> ```
> cargo install --path .
> ```

**Will print to STDOUT:**
````
# Source Files

## `main_script.py`:
```
def main():
    print("Main script running")

if __name__ == "__main__":
    main()
```

## `script1.py`:
```
print("This is script 1")
```

## `script2.py`:
```
print("This is script 2")
```
````
