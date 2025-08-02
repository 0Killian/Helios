use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use axum_distributed_routing::route;
use futures::{Sink, SinkExt, StreamExt};
use futures_util::Stream;

use crate::{PostgresAppState, agents::Agents};

pub struct WebSocketStream {
    inner: WebSocket,
}

impl Stream for WebSocketStream {
    type Item = Result<String, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(Message::Text(message)))) => {
                Poll::Ready(Some(Ok(message.to_string())))
            }
            Poll::Ready(Some(Ok(Message::Binary(_)))) => Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received binary message",
            )))),
            Poll::Ready(Some(Ok(Message::Ping(_)))) => Poll::Pending,
            Poll::Ready(Some(Ok(Message::Pong(_)))) => Poll::Pending,
            Poll::Ready(Some(Ok(Message::Close(_)))) => Poll::Pending,
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::Other,
                err.to_string(),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Sink<String> for WebSocketStream {
    type Error = std::io::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn start_send(self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.get_mut()
            .inner
            .start_send_unpin(Message::Text(item.into()))
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .inner
            .poll_flush_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .inner
            .poll_close_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

route!(
    group = Agents,
    method = GET,
    path = "/websocket",

    async websocket_handler(state: State<PostgresAppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(|socket| async move {
            let stream = WebSocketStream { inner: socket };
            state.handle_agent_websocket.clone().execute(stream).await;
        })
    }
);
