use kaja_html_macro::html;

// impl method does not exist

struct Main {}

impl Main {
    fn get_html(&self) {
        let content = html! {{
            <include self.get_outer_html() />
        }};
    }
}

fn main() {
    let main = Main {};
    main.get_html();
}
