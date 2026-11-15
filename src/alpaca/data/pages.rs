use std::{marker::PhantomData, mem};

use crate::alpaca::ClientError;

/// A response that may point to a following page.
pub(super) trait Page {
    fn next_page_token(&self) -> Option<&str>;
}

enum State {
    First,
    Next(String),
    Done,
}

/// Lazily fetches pages by passing each page's token to the next fetch.
/// Ends after a page without a token and after the first error.
/// A page whose token repeats the one it was fetched with is an invalid response.
pub(super) struct Pages<P, F> {
    fetch: F,
    state: State,
    page: PhantomData<fn() -> P>,
}

impl<P, F> Pages<P, F>
where
    P: Page,
    F: FnMut(Option<&str>) -> Result<P, ClientError>,
{
    pub(super) fn new(fetch: F) -> Self {
        Self {
            fetch,
            state: State::First,
            page: PhantomData,
        }
    }
}

impl<P, F> Iterator for Pages<P, F>
where
    P: Page,
    F: FnMut(Option<&str>) -> Result<P, ClientError>,
{
    type Item = Result<P, ClientError>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match mem::replace(&mut self.state, State::Done) {
            State::First => None,
            State::Next(token) => Some(token),
            State::Done => return None,
        };
        let result = (self.fetch)(token.as_deref());
        let next = result
            .as_ref()
            .ok()
            .and_then(Page::next_page_token)
            .filter(|token| !token.is_empty());
        match next {
            // Following a token that points to itself would never end.
            Some(next) if token.as_deref() == Some(next) => {
                return Some(Err(ClientError::InvalidResponse));
            }
            Some(next) => self.state = State::Next(next.to_owned()),
            None => {}
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Numbered {
        number: u32,
        token: Option<&'static str>,
    }

    impl Page for Numbered {
        fn next_page_token(&self) -> Option<&str> {
            self.token
        }
    }

    // Serves the scripted results in order and records the token of every call.
    fn scripted(
        results: Vec<Result<Numbered, ClientError>>,
        calls: &mut Vec<Option<String>>,
    ) -> impl FnMut(Option<&str>) -> Result<Numbered, ClientError> {
        let mut results = results.into_iter();
        move |token| {
            calls.push(token.map(str::to_owned));
            results.next().expect("fetch called after the last page")
        }
    }

    fn page(number: u32, token: Option<&'static str>) -> Result<Numbered, ClientError> {
        Ok(Numbered { number, token })
    }

    #[test]
    fn next_passes_no_token_when_fetching_first_page() {
        let mut calls = Vec::new();
        let mut pages = Pages::new(scripted(vec![page(1, None)], &mut calls));

        let first = pages.next();
        drop(pages);

        assert_eq!(first, Some(page(1, None)));
        assert_eq!(calls, [None]);
    }

    #[test]
    fn next_passes_previous_token_when_page_has_token() {
        let mut calls = Vec::new();
        let script = vec![page(1, Some("a")), page(2, Some("b")), page(3, None)];

        let pages: Vec<_> = Pages::new(scripted(script, &mut calls)).collect();

        assert_eq!(
            pages,
            [page(1, Some("a")), page(2, Some("b")), page(3, None)]
        );
        assert_eq!(calls, [None, Some("a".into()), Some("b".into())]);
    }

    #[test]
    fn next_ends_when_page_has_no_token() {
        let mut calls = Vec::new();
        let mut pages = Pages::new(scripted(vec![page(1, None)], &mut calls));
        pages.next();

        let after_last = (pages.next(), pages.next());
        drop(pages);

        assert_eq!(after_last, (None, None));
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn next_ends_when_page_token_is_empty() {
        let mut calls = Vec::new();
        let script = vec![page(1, Some(""))];

        let pages: Vec<_> = Pages::new(scripted(script, &mut calls)).collect();

        assert_eq!(pages, [page(1, Some(""))]);
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn next_yields_error_once_when_fetch_fails() {
        let mut calls = Vec::new();
        let script = vec![page(1, Some("a")), Err(ClientError::InvalidResponse)];

        let pages: Vec<_> = Pages::new(scripted(script, &mut calls)).collect();

        assert_eq!(
            pages,
            [page(1, Some("a")), Err(ClientError::InvalidResponse)]
        );
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn next_yields_invalid_response_when_page_repeats_its_token() {
        let mut calls = Vec::new();
        let script = vec![page(1, Some("a")), page(2, Some("a"))];

        let pages: Vec<_> = Pages::new(scripted(script, &mut calls)).collect();

        assert_eq!(
            pages,
            [page(1, Some("a")), Err(ClientError::InvalidResponse)]
        );
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn next_fetches_nothing_when_not_polled() {
        let mut calls = Vec::new();

        let pages = Pages::new(scripted(vec![page(1, None)], &mut calls));
        drop(pages);

        assert!(calls.is_empty());
    }
}
