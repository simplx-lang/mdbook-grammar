use mdbook::preprocess::CmdPreprocessor;
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

  let (_, mut book) = CmdPreprocessor::parse_input(std::io::stdin()).unwrap();

  run(&mut book);

  serde_json::to_writer(std::io::stdout(), &book).unwrap();
}
