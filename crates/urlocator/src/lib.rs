#![cfg_attr(all(test, feature = "nightly"), feature(test))]
#![cfg_attr(not(test), no_std)]

mod scheme;
#[cfg(test)]
mod tests;

use scheme::SchemeState;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UrlLocation {
    Url(u16, u16),
    Scheme,
    Reset,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum State {
    Scheme(SchemeState),
    Url,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        State::Scheme(SchemeState::default())
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UrlLocator {
    state: State,
    illegal_end_chars: u16,
    len: u16,
    open_parentheses: u8,
    open_brackets: u8,
}

impl UrlLocator {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn advance(&mut self, c: char) -> UrlLocation {
        self.len += 1;

        match self.state {
            State::Scheme(state) => self.advance_scheme(state, c),
            State::Url => self.advance_url(c),
        }
    }

    #[inline]
    fn advance_scheme(&mut self, state: SchemeState, c: char) -> UrlLocation {
        self.state = match state.advance(c) {
            SchemeState::RESET => return self.reset(),
            SchemeState::COMPLETE => State::Url,
            state => State::Scheme(state),
        };

        UrlLocation::Scheme
    }

    #[inline]
    fn advance_url(&mut self, c: char) -> UrlLocation {
        if Self::is_illegal_at_end(c) {
            self.illegal_end_chars += 1;
        } else {
            self.illegal_end_chars = 0;
        }

        self.url(c)
    }

    #[inline]
    fn url(&mut self, c: char) -> UrlLocation {
        match c {
            '(' => self.open_parentheses += 1,
            '[' => self.open_brackets += 1,
            ')' => {
                if self.open_parentheses == 0 {
                    return self.reset();
                } else {
                    self.open_parentheses -= 1;
                }
            },
            ']' => {
                if self.open_brackets == 0 {
                    return self.reset();
                } else {
                    self.open_brackets -= 1;
                }
            },
            '\u{00}'..='\u{1F}'
            | '\u{7F}'..='\u{9F}'
            | '<'
            | '>'
            | '"'
            | ' '
            | '{'..='}'
            | '\\'
            | '^'
            | '\u{27E8}'
            | '\u{27E9}'
            | '`' => return self.reset(),
            _ => (),
        }

        self.state = State::Url;

        UrlLocation::Url(self.len - self.illegal_end_chars, self.illegal_end_chars)
    }

    #[inline]
    fn is_illegal_at_end(c: char) -> bool {
        match c {
            '.' | ',' | ':' | ';' | '?' | '!' | '(' | '[' | '\''
            | '\u{3001}' | '\u{3002}'          // 、。
            | '\u{3009}' | '\u{300B}'          // 〉》
            | '\u{300D}' | '\u{300F}'          // 」』
            | '\u{3010}' | '\u{3011}'          // 【】
            | '\u{3014}' | '\u{3015}'          // 〔〕
            | '\u{FF01}' | '\u{FF0C}' | '\u{FF0E}'          // ！，．
            | '\u{FF09}' | '\u{FF3D}'          // ）］
            | '\u{FF1A}' | '\u{FF1B}'          // ：；
            | '\u{FF1F}'                       // ？
            | '\u{FF5D}'                       // ｝
            => true,
            _ => false,
        }
    }

    #[inline]
    fn reset(&mut self) -> UrlLocation {
        *self = Self::default();
        UrlLocation::Reset
    }
}
