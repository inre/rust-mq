use std::sync::Arc;
use std::vec::Vec;
use {Publish, TopicPath, PacketIdentifier, QoS, LastWill, Error, Result};

#[derive(Debug, Clone)]
pub struct Message {
    pub topic: TopicPath,
    pub qos: QoS,
    pub retain: bool,
    // Only for QoS 1,2
    pub pid: Option<PacketIdentifier>,
    pub payload: Arc<Vec<u8>>
}

impl Message {
    pub fn from_pub(publish: Arc<Publish>) -> Result<Arc<Message>> {
        let topic = TopicPath::from(publish.topic_name.as_str());
        if topic.wildcards {
            return Err(Error::TopicNameMustNotContainWildcard);
        }
        Ok(Arc::new(Message {
            topic: topic,
            qos: publish.qos,
            retain: publish.retain,
            pid: publish.pid,
            payload: publish.payload.clone()
        }))
    }

    pub fn from_last_will(last_will: LastWill) -> Arc<Message> {
        let topic = TopicPath::from(last_will.topic);

        Arc::new(Message {
            topic: topic,
            qos: last_will.qos,
            retain: last_will.retain,
            pid: None,
            payload: Arc::new(last_will.message.into_bytes())
        })
    }

    pub fn to_pub(&self, qos: Option<QoS>, dup: bool) -> Arc<Publish> {
        let qos = qos.unwrap_or(self.qos);
        Arc::new(Publish {
            dup: dup,
            qos: qos,
            retain: false,
            topic_name: self.topic.path.clone(),
            pid: self.pid,
            payload: self.payload.clone()
        })
    }

    pub fn transform(&self, pid: Option<PacketIdentifier>, qos: Option<QoS>) -> Arc<Message> {
        let pid = pid.or(self.pid);
        let qos = qos.unwrap_or(self.qos);
        Arc::new(Message {
            topic: self.topic.clone(),
            qos: qos,
            retain: self.retain,
            pid: pid,
            payload: self.payload.clone()
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use super::{Message};
    use {Publish, QoS, ToTopicPath, PacketIdentifier};

    #[test]
    fn message_to_pub_test() {
        let msg = Message {
            topic: "/a/b".to_topic_path().unwrap(),
            qos: QoS::AtLeastOnce,
            retain: false,
            pid: Some(PacketIdentifier(1)),
            payload: Arc::new(vec![0x80, 0x40])
        };
        let publish = msg.to_pub(None, false);

        assert_eq!(publish, Arc::new(Publish {
            dup: false,
            qos: QoS::AtLeastOnce,
            retain: false,
            topic_name: "/a/b".to_owned(),
            pid: Some(PacketIdentifier(1)),
            payload: Arc::new(vec![0x80, 0x40])
        }));
    }

    #[test]
    fn message_from_pub_test() {
        let publish = Arc::new(Publish {
            dup: true,
            qos: QoS::ExactlyOnce,
            retain: true,
            topic_name: "/a/b/c".to_owned(),
            pid: Some(PacketIdentifier(2)),
            payload: Arc::new(vec![0x10, 0x20, 0x30])
        });
        let msg = Message::from_pub(publish).unwrap();

        assert_eq!(msg.topic.path(), "/a/b/c".to_string());
        assert_eq!(msg.qos, QoS::ExactlyOnce);
        assert_eq!(msg.pid, Some(PacketIdentifier(2)));
        assert_eq!(msg.payload, Arc::new(vec![0x10, 0x20, 0x30]));
        assert!(msg.retain);
    }

    #[test]
    fn to_topic_name_test() {
        assert!("/a/b/c".to_topic_name().is_ok());
        assert!("/a/+/c".to_topic_name().is_err());
    }
}
