use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;

use super::{Call, Reply, Request};

/// Hands `request` to the workspace on the gpui thread and waits for its answer.
pub async fn ask(calls: &UnboundedSender<Call>, request: Request) -> Reply {
    let (reply, answer) = oneshot::channel::<Reply>();
    if calls.unbounded_send(Call { request, reply }).is_err() {
        return Err("IVE is shutting down".to_owned());
    }
    answer.await.unwrap_or_else(|_| Err("IVE closed before answering".to_owned()))
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use futures::channel::mpsc::unbounded;
    use futures::executor::block_on;
    use serde_json::json;

    use super::*;
    use crate::api::{Call, Request};

    #[test]
    fn should_return_reply_of_answered_call() {
        // given
        let (calls, mut received) = unbounded::<Call>();
        let responder = std::thread::spawn(move || {
            let call = block_on(received.next()).unwrap();
            let _ = call.reply.send(Ok(json!("ok")));
        });

        // when
        let reply = block_on(ask(&calls, Request::ListTabs));

        // then
        responder.join().unwrap();
        assert_eq!(reply, Ok(json!("ok")));
    }

    #[test]
    fn should_report_dropped_reply() {
        // given
        let (calls, mut received) = unbounded::<Call>();
        let responder = std::thread::spawn(move || drop(block_on(received.next())));

        // when
        let reply = block_on(ask(&calls, Request::ListTabs));

        // then
        responder.join().unwrap();
        assert_eq!(reply, Err("IVE closed before answering".to_owned()));
    }
}
