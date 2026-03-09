// Copyright (c) 2026 Johan Mattsson
// License: MIT

use kaja_html_macro::html;

fn another_injection() -> String {
    return "<script>anothertest</script>".to_string();
}

fn include_tag_function() -> String {
    return "<includedtag>OK</includedtag>".to_string();
}

#[test]
fn test_include() {
    let html = html! {{
        <html>
            <body>
                <include include_tag_function() />
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);
    assert!(html.contains("<includedtag>OK</includedtag>"));
}

#[test]
fn test_escape_tag() {
    let injection = "<script>test</script>";
    let html = html! {{
        <html>
            <body>
                $injection
                $(another_injection())
                <include include_tag_function() />
            </body>
        </html>
    }};

    println!("Generated HTML: {:?}", html);

    assert!(html.contains("<html>"));
    assert!(html.contains("</html>"));

    assert!(!html.contains("<script>test</script>"));
    assert!(html.contains("&lt;script&gt;test&lt;/script&gt;"));
    assert!(html.contains("&lt;script&gt;anothertest&lt;/script&gt;"));

    assert!(html.contains("<includedtag>OK</includedtag>"));
}
