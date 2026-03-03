// Copyright (c) 2026 Johan Mattsson
// License: MIT

use htmlmacro::html;

struct Test {
    number: u32,
    message: String,
}

impl std::fmt::Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Test {{ number: {}, message: {} }}",
            self.number, self.message
        )
    }
}

fn the_number() -> u64 {
    return 77;
}

fn child_component_simple_loop() -> String {
    let html = html! {{
        <h1>Hello From Nested Component</h1>

        <div>
            <ul>
                <rust>
                    for i in 1..=3 {
                        let element_id = format!("List item {:?} ", i);

                        <markup>
                            <li>Simple $element_id</li>
                        </markup>
                    }
                </rust>
            </ul>
        </div>
    }};
    html
}

fn child_component_expr() -> String {
    let html = html! {{
        <h1>Hello From Nested Component</h1>

        <div>
            <ul>
                <rust>
                    for i in 1..=3 {
                        <markup>
                            <li>Adding 100 to $i: $(i + 100)</li>
                        </markup>
                    }
                </rust>
            </ul>
        </div>
    }};
    html
}

fn create_component(component_id: String) -> String {
    let html = html! {{
        <div id="$component_id">created</div>
    }};

    html
}
#[test]
fn test_compiles() {
    println!("Generating HTML");

    let some_variable = "this_is_runtime";
    let some_struct = Test {
        number: 42,
        message: "The answer".to_string(),
    };

    let other_struct = Test {
        number: 2,
        message: "Primy".to_string(),
    };

    // TODO: escape {}
    let html = html! {{
        <h1>Hello $some_variable</h1>
        <h1>Struct $some_struct</h1>
        <h1>Number Hello Prime $(&other_struct.number)</h1>
        <h1>Hello Number plus one $(the_number() + 1)</h1>

        <p>Lorem Ipsum ... (test)</p>
        <p>$("White space is important ... here: Lorem  Ipsum  (test)")</p>

        <include child_component_simple_loop() />
        <include child_component_expr() />

        <div id="self_closing" />

        <div>
            <rust>
                for i in 1..=5 {
                    </rust>inline in loop<rust>
                }
            </rust>

            <h2>A Generated List</h2>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("Fruits {:?} ", i);
                    <markup>SubTreeComponent: $element_id</markup>
                }
            </rust>

            <h2>Five Included Components</h2>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("generated_component_{:?}", i);
                    <markup><include create_component(element_id) /></markup>
                }
            </rust>
        </div>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(html.contains("<h2>Five Included Components</h2>"));
    assert!(html.contains("inline in loop"));
    assert!(html.contains("SubTreeComponent: Fruits 1"));
    assert!(html.contains("<div id=\"generated_component_1\">created</div>"));
}
