use unscanny::Scanner;

pub fn parse_mode(text: &str) -> String {
    let mut s = Scanner::new(text);
    let mut content = String::new();

    loop {
        // Treat as normal text until we find a "{{"
        content += s.eat_until("{{");
        let start = s.cursor();

        if !s.eat_if("{{") {
            // If we don't find a "{{", we are done
            break;
        }

        s.eat_whitespace();
        if s.eat_if("#mode") {
            // If we find "#mode", we expect a list of modes
            s.eat_until("}}").split(",").for_each(|mode: &str| {
                content += &format!(
                    "<span class=\"syntax-mode\" mode=\"{mode}\">{mode}</span>",
                    mode = mode.trim()
                )
            });
            s.eat_if("}}");
        } else {
            // This is not a mode, so just treat as normal text
            content += s.from(start);
        }
    }

    content
}
