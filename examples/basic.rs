use enum_ordinalize::Ordinalize;
use logos::Logos;
use rowan::SyntaxKind;
use rowan_nom::{alt, eof, join, many0, node, root_node, t, DummyError, IResult, Input};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Logos, Ordinalize)]
#[repr(u16)]
enum Token {
    #[token("+")]
    Add,

    #[token("-")]
    Sub,

    #[token("*")]
    Mul,

    #[regex(r#"\d+"#)]
    Literal,

    #[regex(r#"\s+"#)]
    Space,

    #[error]
    Error,

    // Rowan nodes
    Additive,
    Multiplicative,
    Root,
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Language;

impl rowan::Language for Language {
    type Kind = Token;

    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        Token::from_ordinal(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        SyntaxKind(kind as u16)
    }
}

impl rowan_nom::RowanNomLanguage for Language {
    fn is_trivia(kind: Self::Kind) -> bool {
        matches!(kind, Token::Space)
    }

    fn get_error_kind() -> Self::Kind {
        Token::Error
    }
}

#[rustfmt::skip]
fn parse_multiplicative<'slice, 'src>(
    input: Input<'slice, 'src, Language>,
) -> IResult<'slice, 'src, Language, DummyError> {
    node(
        Token::Multiplicative,
        join((
            t(Token::Literal),
            many0(join((
                t(Token::Mul),
                t(Token::Literal),
            ))),
        )),
    )(input)
}

#[rustfmt::skip]
fn parse_additive<'slice, 'src>(
    input: Input<'slice, 'src, Language>,
) -> IResult<'slice, 'src, Language, DummyError> {
    node(
        Token::Additive,
        join((
            parse_multiplicative,
            many0(join((
                alt((t(Token::Add), t(Token::Sub))),
                parse_multiplicative,
            ))),
        )),
    )(input)
}

fn main() {
    let source = "10 - 2 * 3";

    let tokens = Token::lexer(source)
        .spanned()
        .map(|(token, span)| (token, &source[span]))
        .collect::<Vec<_>>();

    let input = Input::<Language>::from(tokens.as_slice());

    let (_, (ast, errors)) = root_node(Token::Root, join((parse_additive, eof)))(input).unwrap();

    dbg!(ast, errors);
}
