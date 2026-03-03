use htmlmacro::html;

fn main() {
    // missing variable does_not_exist
    let html = html! {{
        <html>
            <body>
                $does_not_exist
            </body>
        </html>
    }};
}
