use std::sync::{mpsc, Mutex, Arc};
use std::collections::HashMap;
use crate::threadpool::ThreadPool;

pub struct PubSub {
    subscribers: Arc<Mutex<HashMap<String, Vec<Subscriber>>>>,
    next_sub_id: Mutex<usize>,
    workers: ThreadPool,
}

impl PubSub {
    pub fn new() -> Self {
        let subscribers = Arc::new(Mutex::new(HashMap::new()));
        let next_sub_id = Mutex::new(0);
        let workers = ThreadPool::new(8);

        PubSub { subscribers, next_sub_id, workers }
    }

    pub fn publish(&mut self, channel: String, message: String) {
        let subscribers = self.subscribers.clone();

        self.workers.execute(move || {
            let subscribers = subscribers.lock().unwrap();

            match subscribers.get(&channel) {
                Some(subscribers) => {
                    for subscriber in subscribers {
                        let _result = subscriber.send(message.clone());
                    }
                },
                None => (),
            };
        });
    }

    pub fn subscribe(&mut self, channel: String) -> Result<mpsc::Receiver<String>, ()> {
        let (sender, receiver) = mpsc::channel();

        let mut next_sub_id = self.next_sub_id.lock().unwrap();
        let subscriber = Subscriber::new(*next_sub_id, sender);
        *next_sub_id += 1;

        let mut subscribers = self.subscribers.lock().unwrap();

        let subscriber_list = subscribers.entry(channel).or_insert(Vec::new());

        subscriber_list.push(subscriber);

        Ok(receiver)
    }
}

struct Subscriber {
    id: usize,
    sender: mpsc::Sender<String>,
}

impl Subscriber {
    pub fn new(id: usize, sender: mpsc::Sender<String>) -> Self {
        Subscriber { id, sender }
    }
    
    pub fn send(&self, message: String) -> Result<(), ()> {
        let _result = self.sender.send(message);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut ps = PubSub::new();

        let receiver = ps.subscribe(String::from("test")).unwrap();

        let _ = ps.publish(String::from("test"), String::from("Hello world!"));

        let message = receiver.recv().unwrap();

        assert_eq!(&message, "Hello world!");
    }
}
