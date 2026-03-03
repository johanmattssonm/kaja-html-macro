# html! — Declarative HTML Macro for Rust

`html!` is a declarative macro for generating HTML directly inside your Rust
project. It expands at compile time into `String`.

## Concept

- Write HTML-like markup directly in Rust
- Embed Rust control flow with `<rust>`, plain Rust code, no DSL
- Emit markup dynamically using `<markup>`
- Interpolate variables and expressions
- Returns a `String`

## Interpolation Rules

### `$variable`

Inserts a variable and escapes HTML tags.

```rust
use htmlmacro::html;
let name = "<b>Johan</b>";

let html = html! {{
    <p>Hello $name</p>
}};
```

Output:

```html
<p>Hello &lt;b&gt;Johan&lt;/b&gt;</p>
```

### `$(expression)`

You can evaluate arbitrary Rust expressions:

- Function calls
- Arithmetic
- Method calls
- Complex expressions

Expressions are also automatically escaped.

```rust
use htmlmacro::html;
fn get_value() -> String {
    "42".to_string()
}

let test = 5;

let html = html! {{
    <div>
        <p>Value: $(get_value())</p>
        <p>Next: $(test + 1)</p>
    </div>
}};

```

## Generating HTML strings with this Rust macro

Rust logic can be embedded inside `<rust>` blocks.
Markup emitted from Rust should be wrapped in `<markup>`.

Use `<include />` to insert HTML content from another component.

```rust
use htmlmacro::html;

fn parent() -> String {
    html! {{
        <div>
            <include child_component_simple_loop() />
        </div>
    }}
}

fn child_component_simple_loop() -> String {
    html! {{
        <h1>Hello From Nested Component</h1>

        <div>
            <ul>
                <rust>
                    for i in 1..=3 {
                        let element_id = format!("list item {:?}", i);

                        <markup>
                            <li>Simple $element_id</li>
                        </markup>
                    }
                </rust>
            </ul>
        </div>
    }}
}
```

## Output

```html
<h1>Hello From Nested Component</h1>

<div>
    <ul>
        <li>Simple list item 1</li>
        <li>Simple list item 2</li>
        <li>Simple list item 3</li>
    </ul>
</div>
```

## Variables and Functions in the Markup

- `$variable_name` -- insert content of the variable and escape tags.
- `$(get_value())` -- insert string from the get_value function and escape tags.
- `$(x + 2)` -- compute x + 2 and add the content to the html string, escaped if needed.
- `<include some_other_component() />` insert trusted HTML content from the function some_other_component, html tags are allowed.
