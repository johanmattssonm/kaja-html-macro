// Copyright (c) 2026 Johan Mattsson
// License: MIT

use htmlmacro::html;

#[test]
fn test_interleaved() {
    let html = html! {{
        <h1>Headline</h1>
        <rust>
            let a = 1;
            <markup>Content</markup>
        </rust>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(!html.contains("let a = 1;"));
    assert!(!html.contains("markup"));
    assert!(html.contains("Content"));
}

#[test]
fn test_one_rust() {
    let html = html! {{
        <h1>Headline</h1>
        <rust>
            let a = 1;
        </rust>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(!html.contains("let a = 1;"));
}

#[test]
fn test_adding_rust_block_backwards() {
    let content = html! {{
        <div>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("row_{:?} ", i);
                    </rust>a little bit backwards but it has to work. $element_id.<rust>
                }
            </rust>
        </div>
    }};

    assert!(!content.contains("<rust>"));
    assert!(!content.contains("</rust>"));
    assert!(!content.contains("<markup>"));
    assert!(!content.contains("</markup>"));
    assert!(content.contains("little bit backwards"));
    assert!(content.contains("row_5"));
}

#[test]
fn test_adding_rust_block_and_markup() {
    let content = html! {{
        <div>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("id_{:?} ", i);
                    <markup>The proper way: $element_id</markup>
                }
            </rust>
        </div>
    }};

    assert!(!content.contains("<rust>"));
    assert!(!content.contains("</rust>"));
    assert!(!content.contains("<markup>"));
    assert!(!content.contains("</markup>"));
    assert!(content.contains("The proper way:"));
    assert!(content.contains("id_5"));
}
