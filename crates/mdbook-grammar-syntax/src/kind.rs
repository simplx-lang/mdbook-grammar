#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    /// the root of the syntax tree
    Root,
    /// comment (`// ...` or `/* ... */`)
    Comment,
    /// white spaces
    Whitespace,
    /// end of input
    End,
    /// error
    Error,

    /// name of rules
    Identifier,
    /// string literal
    String,
    /// integer literal
    Integer,
    /// meta description
    Meta,
    /// operation after `if` and `->`
    Operation,
    /// `if`
    If,

    /// `:`
    Colon,
    /// `;`
    SemiColon,
    /// `->`
    Arrow,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `,`
    Comma,
    /// `|`
    Bar,
    /// `~`
    Tilde,
    /// `.`
    Dot,
    /// `?`
    Question,
    /// `*`
    Star,
    /// `+`
    Plus,
    /// `..`
    Dots,
    /// `?=`
    LookAheadPos,
    /// `?!`
    LookAheadNeg,
    /// `?<=`
    LookBehindPos,
    /// `?<!`
    LookBehindNeg,

    /// a grammar rule
    Rule,
    /// the param of a rule
    Param,
    /// the definition of a rule
    Definition,

    /// a group expression
    Group,
    /// a converse expression
    Converse,
    /// a range expression
    Range,
    /// a repeating expression
    Repeating,
    /// the brace repeating indicator
    BraceIndicator,
    /// a lookahead or lookbehind expression
    Looking,
    /// an action expression
    Action,
    /// rule reference with argument
    Reference,
}

impl SyntaxKind {
    pub fn is_error(self) -> bool {
        matches!(self, SyntaxKind::Error)
    }

    pub fn is_end(self) -> bool {
        matches!(self, SyntaxKind::End)
    }

    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
    }

    pub fn is_looking(self) -> bool {
        matches!(
            self,
            SyntaxKind::LookAheadPos
                | SyntaxKind::LookAheadNeg
                | SyntaxKind::LookBehindPos
                | SyntaxKind::LookBehindNeg
        )
    }

    pub fn is_prefix(self) -> bool {
        matches!(
            self,
            SyntaxKind::Question
                | SyntaxKind::Star
                | SyntaxKind::Plus
                | SyntaxKind::LeftBrace
        )
    }

    pub fn is_operator(self) -> bool {
        matches!(
            self,
            SyntaxKind::Colon
                | SyntaxKind::SemiColon
                | SyntaxKind::Arrow
                | SyntaxKind::LeftParen
                | SyntaxKind::RightParen
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::Comma
                | SyntaxKind::Bar
                | SyntaxKind::Tilde
                | SyntaxKind::Dot
                | SyntaxKind::Question
                | SyntaxKind::Star
                | SyntaxKind::Plus
                | SyntaxKind::Dots
                | SyntaxKind::LookAheadPos
                | SyntaxKind::LookAheadNeg
                | SyntaxKind::LookBehindPos
                | SyntaxKind::LookBehindNeg
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            | SyntaxKind::Root => "root",
            | SyntaxKind::Comment => "comment",
            | SyntaxKind::Whitespace => "whitespace",
            | SyntaxKind::End => "end",
            | SyntaxKind::Error => "error",
            | SyntaxKind::Identifier => "identifier",
            | SyntaxKind::String => "string",
            | SyntaxKind::Integer => "integer",
            | SyntaxKind::Meta => "meta",
            | SyntaxKind::Operation => "operation",
            | SyntaxKind::If => "if",
            | SyntaxKind::Colon => "`:`",
            | SyntaxKind::SemiColon => "`;`",
            | SyntaxKind::Arrow => "`->`",
            | SyntaxKind::LeftBracket => "`[`",
            | SyntaxKind::RightBracket => "`]`",
            | SyntaxKind::LeftParen => "`(`",
            | SyntaxKind::RightParen => "`)`",
            | SyntaxKind::LeftBrace => "`{`",
            | SyntaxKind::RightBrace => "`}`",
            | SyntaxKind::Comma => "`,`",
            | SyntaxKind::Bar => "`|`",
            | SyntaxKind::Tilde => "`~`",
            | SyntaxKind::Dot => "`.`",
            | SyntaxKind::Question => "`?`",
            | SyntaxKind::Star => "`*`",
            | SyntaxKind::Plus => "`+`",
            | SyntaxKind::Dots => "`..`",
            | SyntaxKind::LookAheadPos => "`?=`",
            | SyntaxKind::LookAheadNeg => "`?!`",
            | SyntaxKind::LookBehindPos => "`?<=`",
            | SyntaxKind::LookBehindNeg => "`?<!`",
            | SyntaxKind::Rule => "rule",
            | SyntaxKind::Param => "param",
            | SyntaxKind::Definition => "definition",
            | SyntaxKind::Group => "group",
            | SyntaxKind::Converse => "converse",
            | SyntaxKind::Range => "range",
            | SyntaxKind::Repeating => "repeating",
            | SyntaxKind::BraceIndicator => "brace_indicator",
            | SyntaxKind::Looking => "looking",
            | SyntaxKind::Action => "action",
            | SyntaxKind::Reference => "reference",
        }
    }
}
