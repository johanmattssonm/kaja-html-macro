// Copyright (c) 2026 Johan Mattsson
// License: MIT

use kaja_html_macro::html;

struct TestDataMessage {
    message: String,
    logged_in: bool,
}

struct TestData {
    data: TestDataMessage,
}

impl TestData {}

fn login_form(data: &TestDataMessage) -> String {
    let content = html! {{
        <h1>Form: $(&data.message)</h1>
    }};

    content
}

fn get_inner_html(data: &TestDataMessage) -> String {
    let content = html! {{
        <rust>
            if !data.logged_in {
                <markup>
                    <include login_form(&data) />
                </markup>
            }
        </rust>
    }};

    content
}

#[test]
fn include_inner_html() {
    let test = TestData {
        data: TestDataMessage {
            message: "test data".to_string(),
            logged_in: false,
        },
    };

    let content = html! {{
        <include login_form(&test.data) />
    }};

    assert!(content.contains("test data"));
    assert!(content.contains("Form"));

    let content = html! {{
        <include get_inner_html(&test.data) />
    }};

    println!("Generated: {}", content);
    assert!(content.contains("Form"));
}
