use mdbook::preprocess::{CmdPreprocessor, PreprocessorContext};
use mdbook_grammar_runner::run;

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        | Some("supports") => return,
        | Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        },
        | None => {},
    }

    let (context, mut book) =
        CmdPreprocessor::parse_input(std::io::stdin()).unwrap();
    run(&mut book, get_site_url(&context).unwrap_or("/"));
    serde_json::to_writer(std::io::stdout(), &book).unwrap();
}

fn get_site_url(context: &PreprocessorContext) -> Option<&str> {
    context
        .config
        .get("output")?
        .get("html")?
        .get("site-url")?
        .as_str()
}
