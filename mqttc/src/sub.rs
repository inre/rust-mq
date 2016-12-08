use std::option;
use std::vec;
use {MAX_QOS};
use error::Result;
use mqtt3::{SubscribeTopic, TopicPath, PacketIdentifier, QoS};

#[derive(Debug, Clone)]
pub struct Subscription {
    pub pid: PacketIdentifier,
    pub topic_path: TopicPath,
    pub qos: QoS
}

impl Subscription {
    pub fn to_subscribe_topic(&self) -> SubscribeTopic {
        SubscribeTopic { topic_path: self.topic_path.path(), qos: self.qos }
    }
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

impl ToSubTopics for Vec<SubscribeTopic> {
    type Iter = vec::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        Ok(self.clone().into_iter()) //FIXME:
    }
}

impl<'a> ToSubTopics for &'a str {
    type Iter = option::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        Ok(Some(SubscribeTopic { topic_path: self.to_string(), qos: MAX_QOS }).into_iter())
    }
}

impl<S: Into<String> + Clone> ToSubTopics for (S, QoS) {
    type Iter = option::IntoIter<SubscribeTopic>;
    fn to_subscribe_topics(&self) -> Result<Self::Iter> {
        Ok(Some(SubscribeTopic { topic_path: self.0.clone().into(), qos: self.1 })
            .into_iter())
    }
}

pub trait ToUnSubTopics {
    type Iter: Iterator<Item=String>;
    fn to_unsubscribe_topics(&self) -> Result<Self::Iter>;
}

impl ToUnSubTopics for Vec<String> {
    type Iter = vec::IntoIter<String>;
    fn to_unsubscribe_topics(&self) -> Result<Self::Iter> {
        Ok(self.clone().into_iter())
    }
}

impl<'a> ToUnSubTopics for &'a str {
    type Iter = option::IntoIter<String>;
    fn to_unsubscribe_topics(&self) -> Result<Self::Iter> {
        Ok(Some(self.to_string()).into_iter())
    }
}
