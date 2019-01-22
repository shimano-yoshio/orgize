mod cookie;
mod emphasis;
mod entity;
mod fn_ref;
mod fragment;
mod inline_call;
mod inline_src;
mod link;
mod macros;
mod snippet;
mod target;

pub use self::cookie::Cookie;
pub use self::emphasis::Emphasis;
pub use self::fn_ref::FnRef;
pub use self::inline_call::InlineCall;
pub use self::inline_src::InlineSrc;
pub use self::link::Link;
pub use self::macros::Macros;
pub use self::snippet::Snippet;
pub use self::target::{RadioTarget, Target};

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Object<'a> {
    Cookie(Cookie<'a>),
    FnRef(FnRef<'a>),
    InlineCall(InlineCall<'a>),
    InlineSrc(InlineSrc<'a>),
    Link(Link<'a>),
    Macros(Macros<'a>),
    RadioTarget(RadioTarget<'a>),
    Snippet(Snippet<'a>),
    Target(Target<'a>),

    // `end` indicates the position of the second marker
    Bold { end: usize },
    Italic { end: usize },
    Strike { end: usize },
    Underline { end: usize },

    Verbatim(&'a str),
    Code(&'a str),
    Text(&'a str),
}

impl<'a> Object<'a> {
    pub fn next_2(src: &'a str) -> (Object<'a>, usize, Option<(Object<'a>, usize)>) {
        let bytes = src.as_bytes();

        if src.len() <= 2 {
            return (Object::Text(src), src.len(), None);
        }

        let chars = ascii_chars!('@', ' ', '"', '(', '\n', '{', '<', '[');

        let mut pos = 0;
        loop {
            macro_rules! brk {
                ($obj:expr, $off:expr, $pos:expr) => {
                    break if pos == 0 {
                        ($obj, $off, None)
                    } else {
                        (Object::Text(&src[0..$pos]), $pos, Some(($obj, $off)))
                    };
                };
            }

            let mut pre = pos;

            match (bytes[pos], bytes[pos + 1], bytes[pos + 2]) {
                (b'@', b'@', _) => {
                    if let Some((snippet, off)) = Snippet::parse(&src[pos..]) {
                        brk!(Object::Snippet(snippet), off, pos);
                    }
                }
                (b'{', b'{', b'{') => {
                    if let Some((macros, off)) = Macros::parse(&src[pos..]) {
                        brk!(Object::Macros(macros), off, pos);
                    }
                }
                (b'<', b'<', b'<') => {
                    if let Some((target, off)) = RadioTarget::parse(&src[pos..]) {
                        brk!(Object::RadioTarget(target), off, pos);
                    }
                }
                (b'<', b'<', third) => {
                    if third != b'\n' {
                        if let Some((target, off)) = Target::parse(&src[pos..]) {
                            brk!(Object::Target(target), off, pos);
                        }
                    }
                }
                (b'[', b'f', b'n') => {
                    if let Some((fn_ref, off)) = FnRef::parse(&src[pos..]) {
                        brk!(Object::FnRef(fn_ref), off, pos);
                    }
                }
                (b'[', b'[', _) => {
                    if let Some((link, off)) = Link::parse(&src[pos..]) {
                        brk!(Object::Link(link), off, pos);
                    }
                }
                (b'[', _, _) => {
                    if let Some((cookie, off)) = Cookie::parse(&src[pos..]) {
                        brk!(Object::Cookie(cookie), off, pos);
                    }
                    // TODO: Timestamp
                }
                (b'{', _, _)
                | (b' ', _, _)
                | (b'"', _, _)
                | (b',', _, _)
                | (b'(', _, _)
                | (b'\n', _, _) => pre += 1,
                _ => (),
            }

            match bytes[pre] {
                b'*' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'*') {
                        brk!(Object::Bold { end }, 1, pre);
                    }
                }
                b'+' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'+') {
                        brk!(Object::Strike { end }, 1, pre);
                    }
                }
                b'/' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'/') {
                        brk!(Object::Italic { end }, 1, pre);
                    }
                }
                b'_' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'_') {
                        brk!(Object::Underline { end }, 1, pre);
                    }
                }
                b'=' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'=') {
                        brk!(Object::Verbatim(&src[pre + 1..pre + end]), end + 1, pre);
                    }
                }
                b'~' => {
                    if let Some(end) = Emphasis::parse(&src[pre..], b'~') {
                        brk!(Object::Code(&src[pre + 1..pre + end]), end + 1, pre);
                    }
                }
                b'c' => {
                    if let Some((call, off)) = InlineCall::parse(&src[pre..]) {
                        brk!(Object::InlineCall(call), off, pre);
                    }
                }
                b's' => {
                    if let Some((src, off)) = InlineSrc::parse(&src[pre..]) {
                        brk!(Object::InlineSrc(src), off, pre);
                    }
                }
                _ => (),
            }

            if let Some(off) = chars
                .find(&src[pos + 1..])
                .map(|i| i + pos + 1)
                .filter(|&i| i < src.len() - 2)
            {
                pos = off;
            } else {
                break (Object::Text(src), src.len(), None);
            }
        }
    }
}

#[test]
fn next_2() {
    // TODO: more tests
    assert_eq!(Object::next_2("*bold*"), (Object::Bold { end: 5 }, 1, None));
    assert_eq!(
        Object::next_2("Normal =verbatim="),
        (
            Object::Text("Normal "),
            "Normal ".len(),
            Some((Object::Verbatim("verbatim"), "=verbatim=".len()))
        )
    );
}
