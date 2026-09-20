//! Decode with sse-stream 0.3; adapt event fields to rmcp 3.4's public stream

//! item type. The SDK still depends on 0.2, but its parser is not called here.

use std::{future::ready, io::Cursor};

use futures_util::{Stream, StreamExt};
use rmcp::transport::{
    common::client_side_sse::BoxedSseResponse, streamable_http_client::SseError,
};

use crate::Failure;

pub(crate) const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn decode<S, B>(source: S, limit: usize) -> BoxedSseResponse
where
    S: Stream<Item = Result<B, Failure>> + Send + 'static,
    B: AsRef<[u8]> + Send + 'static,
{
    // Count all raw bytes, including comments and event delimiters. This bound
    // applies before the parser can retain an unfinished event, not per event.
    let source = source.scan((0usize, false), move |(received, ended), chunk| {
        if *ended {
            return ready(None);
        }
        let result = chunk.and_then(|bytes| {
            *received = received.saturating_add(bytes.as_ref().len());
            if *received > limit {
                Err(Failure::ResponseTooLarge)
            } else {
                Ok(Cursor::new(bytes))
            }
        });
        *ended = result.is_err();
        ready(Some(result))
    });
    Box::pin(
        sse_stream::SseByteStream::new(source).map(|result| match result {
            Ok(event) => sdk_event(event),
            Err(sse_stream::Error::Body(error)) => Err(SseError::Body(error)),
            Err(_) => Err(SseError::Body(Box::new(Failure::InvalidResponse))),
        }),
    )
}

// Use the SDK's exported Stream::Item rather than add a second direct
// sse-stream dependency just to name its old event struct. No encode/reparse.
fn sdk_event(event: sse_stream::Sse) -> <BoxedSseResponse as Stream>::Item {
    let mut result = <BoxedSseResponse as Stream>::Item::Ok(Default::default());
    if let Ok(value) = &mut result {
        value.event = event.event;
        value.data = event.data;
        value.id = event.id;
        value.retry = event.retry;
    }
    result
}

/// Local-only development check; no HTTP or credential access.
pub(crate) async fn check_decoder() -> Result<(), Failure> {
    let mut events = decode(
        futures_util::stream::iter([Ok(b"data: synthetic\n\n".as_slice())]),
        MAX_RESPONSE_BYTES,
    );
    match events.next().await {
        Some(Ok(event)) if event.data.as_deref() == Some("synthetic") => Ok(()),
        _ => Err(Failure::InvalidResponse),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::stream;

    #[tokio::test]
    async fn latest_decoder_preserves_sdk_fields_across_fragmented_input() {
        let mut events = decode(
            stream::iter([
                Ok(b": ignored\r\n\
                     event: prior\r\n\
                     event: message\r\n\
                     id: event-1\r\n\
                     retry: 250\r\n\
                     unknown: extension\r\n\
                     data: first\r"
                    .as_slice()),
                Ok(b"\ndata: second\r\n\r\n".as_slice()),
            ]),
            MAX_RESPONSE_BYTES,
        );
        let event = events.next().await.unwrap().unwrap();
        assert_eq!(event.event.as_deref(), Some("message"));
        assert_eq!(event.data.as_deref(), Some("first\nsecond"));
        assert_eq!(event.id.as_deref(), Some("event-1"));
        assert_eq!(event.retry, Some(250));
        assert!(events.next().await.is_none());
    }

    #[tokio::test]
    async fn cumulative_limit_includes_comments_and_completed_events() {
        let bytes = b"data: a\n\n";
        let mut events = decode(
            stream::iter([Ok(bytes.as_slice()), Ok(b": padding\n\n".as_slice())]),
            bytes.len(),
        );
        assert!(events.next().await.unwrap().is_ok());
        let error = events.next().await.unwrap().unwrap_err();
        let SseError::Body(error) = error else {
            panic!("expected bounded body error");
        };
        assert_eq!(
            error.downcast_ref::<Failure>(),
            Some(&Failure::ResponseTooLarge)
        );
        assert!(events.next().await.is_none());
    }

    #[tokio::test]
    async fn malformed_utf8_is_sanitized_once_and_terminates() {
        let mut events = decode(
            stream::iter([
                Ok(b"data: secret-\xff\n\n".as_slice()),
                Ok(b"data: later\n\n".as_slice()),
            ]),
            MAX_RESPONSE_BYTES,
        );
        let error = events.next().await.unwrap().unwrap_err();
        assert!(!format!("{error:?}").contains("secret"));
        assert!(events.next().await.is_none());
    }

    #[tokio::test]
    async fn incomplete_final_event_is_not_fabricated() {
        let mut events = decode(
            stream::iter([Ok(b"data: truncated".as_slice())]),
            MAX_RESPONSE_BYTES,
        );
        assert!(events.next().await.is_none());
    }

    #[tokio::test]
    async fn metadata_only_and_empty_data_remain_distinct() {
        let mut events = decode(
            stream::iter([Ok(b"id: one\n\ndata:\n\n".as_slice())]),
            MAX_RESPONSE_BYTES,
        );
        let first = events.next().await.unwrap().unwrap();
        assert_eq!(first.id.as_deref(), Some("one"));
        assert_eq!(first.data, None);
        let second = events.next().await.unwrap().unwrap();
        assert_eq!(second.data.as_deref(), Some(""));
        assert_eq!(second.id, None);
    }
}
