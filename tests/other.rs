// Copyright (c) 2026 Johan Mattsson
// License: MIT

#[cfg(test)]
use std::time::Duration;

use kaja_html_macro::html;

#[test]
fn test_var() {
    let variable = "test";

    let html = html! {{
        <html>
            <body>
                $variable
            </body>
        </html>
    }};

    eprintln!("Generated HTML: {:?}", html);
    assert!(html.contains("test"));
}

#[test]
fn test_html_js_included() {
    let test_var = "Headline";

    let html = html! {{
        <html>
        <header>
            <script type="module">
                import init, { update_text } from "/static/wasm_module.js";

                async function start() {
                    await init();
                    update_text("output", "tested");
                }

                start();
            </script>
        </header>
        <body>
        <h1>$test_var<h1/>
        </body>
        </html>
    }};

    eprintln!("Generated HTML: {:?}", html);
    assert!(html.contains("<html>"));
    assert!(html.contains("<script "));
    assert!(html.contains("update_text(\"output\", \"tested\");"));
}

#[test]
fn test_exp() {
    let variable = "test";
    let html = html! {{
        <html>
            <body>
                $(variable)
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(html.contains("test"));
}

fn project_table() -> String {
    return "<table></table>".into();
}

#[test]
fn test_include() {
    let html = html! {{
        <include project_table() />
    }};

    assert_eq!(html.trim(), "<table></table>")
}

#[test]
fn test_separate_code_and_html() {
    let html = html! {{
        <h1>Hello World</h1>

        <div>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("row_{}", i);
                    <markup>$element_id</markup>
                }
            </rust>
        </div>
    }};

    println!("Generated HTML: {:?}", html);

    assert!(html.contains("<h1>Hello World</h1>"));
    assert!(html.contains("<div>"));
    assert!(html.contains("</div>"));
    assert!(html.contains("row_5"));
    assert!(!html.contains("<rust>"));
}

fn menu(title: &str) -> String {
    let html = html! {{
        <span>$title <a href="index.html">Start</a></span>
    }};

    html
}

#[test]
fn test_should_not_panic() {
    let page_title = "Start";

    let content = html! {{
        <p>Lorem Ipsum</p>
        <p>White space is important here: Lorem Ipsum     (test) or (test)</p>

        <include menu(page_title) />

        <div>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("row_{:?} ", i);
                    </rust>a little bit backwards but it has to work. $element_id.<rust>
                }
            </rust>
        </div>

        <div>
            <rust>
                for i in 1..=5 {
                    let element_id = format!("id_{:?} ", i);
                    <markup>The proper way: $element_id</markup>
                }
            </rust>
        </div>
    }};

    assert!(content.contains("White space is important here: Lorem Ipsum     (test) or (test)"));
    assert!(!content.contains("<rust>"));
    assert!(!content.contains("</rust>"));
    assert!(!content.contains("<markup>"));
    assert!(!content.contains("</markup>"));
    assert!(content.contains("<span>Start"));
}

#[test]
fn test_html_tag() {
    let html = html! {{
        <html>
            <header>
            </header>
            <body>
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);

    assert!(html.contains("<html>"));
    assert!(html.contains("</html>"));

    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
}

#[test]
fn test_expression_algebra() {
    let html = html! {{
        <html>
            <body>
                $(1 + 1), $((1.0 * 3) / 2)
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);

    assert!(html.contains("<html>"));
    assert!(html.contains("</html>"));
}

#[test]
fn test_expression() {
    let some_arg = "test";

    let html = html! {{
        <html>
            <body>
                <div arg="$(some_arg)"></div>
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(html.contains("<div arg=\"test\"></div>"));
}

fn test_import() -> String {
    return String::from("TEST");
}

#[test]
fn test_include_fn() {
    let content = html! {{
        <include test_import() />
    }};

    assert!(content.contains("TEST"));
}

#[test]
fn test_space() {
    struct Response {
        status: u16,
        elapsed: u64,
    }

    let resp = Response {
        status: 200,
        elapsed: 100,
    };

    let content = html! {{
        <div>Status: $(resp.status) Elapsed: $(resp.elapsed)ms</div>
    }};

    let expected = "Status: 200 Elapsed: 100ms";

    println!("Got: {}", content);
    println!("Expected: {}", expected);

    assert!(content.contains(expected));
}

#[test]
fn test_script_compiles() {
    let content = html! {{
        <script>
            function test() {
                let result = "TEST";

                if (result === "TEST") {
                    console.log(result);
                }

                return result;
            }
            </script>
    }};

    assert!(content.contains("<script>"));
}

#[test]
fn test_script_string_compiles() {
    let test = String::from("bird");

    let content = html! {{r#"
        <script>
            function test() {
                let result = 'TEST';
                let inter = "$test";
                return result;
            }
            </script>
    "#}};

    assert!(content.contains("bird"));
}

#[test]
fn test_space_after_var() {
    pub struct Task {
        title: String,
        target_time: Duration,
    }

    let task = Task {
        title: String::from("Test Task"),
        target_time: Duration::from_secs(60),
    };

    let all_tasks = vec![task];

    let content = html! {{
        <rust>
            for task in &all_tasks {
                let minutes: u64 = task.target_time.as_secs() / 60;

                <markup>
                    <h2>$(task.title)</h2>
                    <p>Time to complete: $minutes minutes</p>
                </markup>
            }
        </rust>
    }};

    println!("Content {}", content);
    assert!(content.contains("1 minutes"));
}
