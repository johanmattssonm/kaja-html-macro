use htmlmacro::html;

fn main() {
    // missing slash in include
    let html = html! {{
        <html>
            <body>
                <include include_tag_function() >
            </body>
        </html>
    }};
}
