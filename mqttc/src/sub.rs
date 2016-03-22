use std::option;
use {MAX_QOS};
use error::Result;
use mqtt3::{SubscribeTopic, TopicPath, PacketIdentifier, QoS};

#[derive(Debug, Clone)]
pub struct Subscription {
    pub pid: PacketIdentifier,
    pub topic_path: TopicPath,
    pub qos: QoS
}

pub trait ToSubTopics {
    type Iter: Iterator<Item=SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter>;
}

impl ToSubTopics for SubscribeTopic {
    type Iter = option::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        Ok(Some(self.clone()).into_iter())
    }
}

impl<'a> ToSubTopics for &'a str {
    type Iter = option::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        Ok(Some(SubscribeTopic { topic_path: self.to_string(), qos: MAX_QOS }).into_iter())
    }
}

impl ToSubTopics for (String, QoS) {
    type Iter = option::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        let (ref topic_path, qos): (String, QoS) = *self;
        Ok(Some(SubscribeTopic { topic_path: topic_path.clone(), qos: qos }).into_iter())
    }
}

pub trait ToUnSubTopics {
    type Iter: Iterator<Item=String>;
    fn to_unsubscribe_topics(&self) -> Result<Self::Iter>;
}
