use kaja_html_macro::html;

struct Test {
    test_member: u64,
}

fn main() {
    let test = Test { test_member: 7 };

    // should be $(test.member) not $test.member
    let html = html! {{
        <html>
            <body>
                $test.test_member
            </body>
        </html>
    }};
}
