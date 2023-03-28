use super::RowanNomLanguage as Language;
use rowan::{GreenNode, GreenToken, NodeOrToken, SyntaxNode};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// A partial green AST builder, used as an output for all parsers
///
/// The use of [rowan] in a parser generally implies that the parser should be as resilient as
/// possible. That is, it shouldn't return an error. This structure represents the output that a
/// [rowan-nom][crate] parser should return everywhere. It contains a list of [rowan] nodes or
/// tokens, and a list of currently accumulated errors. This way, all parser can return [Ok] and
/// are still able to emit errors.
///
/// Most of the time, [`Children`] will be built using one of this crate's parsers.
pub struct Children<Lang: Language, E> {
    errors: Vec<E>,
    inner: Vec<NodeOrToken<GreenNode, GreenToken>>,
    _lang: PhantomData<Lang>,
}

impl<Lang: Language, E> Default for Children<Lang, E> {
    fn default() -> Self {
        Self {
            errors: vec![],
            inner: vec![],
            _lang: PhantomData,
        }
    }
}

impl<Lang: Language, E: Debug> Debug for Children<Lang, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Children")
            .field("inner", &self.inner)
            .field("errors", &self.errors)
            .finish()
    }
}

impl<Lang: Language, E> Children<Lang, E> {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_err(error: E) -> Self {
        Self {
            errors: vec![error],
            // TODO include inner non-matched nodes/tokens ?
            inner: vec![NodeOrToken::Node(GreenNode::new(
                Lang::kind_to_raw(Lang::get_error_kind()),
                [],
            ))],
            ..Self::default()
        }
    }

    pub fn from_rowan_children(children: rowan::Children, errors: Vec<E>) -> Self {
        Self {
            errors,
            inner: children
                .map(|e| match e {
                    NodeOrToken::Token(t) => NodeOrToken::Token(t.to_owned()),
                    NodeOrToken::Node(n) => NodeOrToken::Node(n.to_owned()),
                })
                .collect(),
            ..Self::default()
        }
    }

    pub fn into_node(self, kind: Lang::Kind) -> Self {
        Self {
            errors: self.errors,
            inner: vec![NodeOrToken::Node(GreenNode::new(
                Lang::kind_to_raw(kind),
                self.inner,
            ))],
            ..Self::default()
        }
    }

    pub fn into_root_node(self, kind: Lang::Kind) -> (SyntaxNode<Lang>, Vec<E>) {
        let node = SyntaxNode::new_root(GreenNode::new(Lang::kind_to_raw(kind), self.inner));
        (node, self.errors)
    }
}

impl<'src, Lang: Language, E> FromIterator<super::RichToken<'src, Lang>> for Children<Lang, E> {
    fn from_iter<T: IntoIterator<Item = super::RichToken<'src, Lang>>>(iter: T) -> Self {
        Self {
            inner: iter
                .into_iter()
                .map(|(token, str)| {
                    NodeOrToken::Token(GreenToken::new(Lang::kind_to_raw(token), str))
                })
                .collect(),
            ..Self::default()
        }
    }
}

impl<'src, 'a, Lang: Language, E> FromIterator<&'a super::RichToken<'src, Lang>>
    for Children<Lang, E>
{
    fn from_iter<T: IntoIterator<Item = &'a super::RichToken<'src, Lang>>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|x| *x))
    }
}

impl<Lang: Language, E> std::ops::Add for Children<Lang, E> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.errors.extend(rhs.errors);
        self.inner.extend(rhs.inner);
        self
    }
}

impl<Lang: Language, E> std::ops::AddAssign for Children<Lang, E> {
    fn add_assign(&mut self, rhs: Self) {
        self.errors.extend(rhs.errors);
        self.inner.extend(rhs.inner);
    }
}
