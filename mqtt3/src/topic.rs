use std::vec::IntoIter;

const TOPIC_PATH_DELIMITER: char = '/';

use self::Topic::{
    Normal,
    System,
    Blank,
    SingleWildcard,
    MultiWildcard
};

/// FIXME: replace String with &str
#[derive(Debug, Clone, PartialEq)]
pub enum Topic {
    Normal(String),
    System(String), // $SYS = Topic::System("$SYS")
    Blank,
    SingleWildcard, // +
    MultiWildcard,  // #
}

impl Into<String> for Topic {
    fn into(self) -> String {
        match self {
            Normal(s) | System(s) => s,
            Blank => "".to_string(),
            SingleWildcard => "+".to_string(),
            MultiWildcard => "#".to_string()
        }
    }
}

impl Topic {
    pub fn fit(&self, other: &Topic) -> bool {
        match *self {
            Normal(ref str) => {
                match *other {
                    Normal(ref s) => str == s,
                    SingleWildcard | MultiWildcard => true,
                    _ => false
                }
            },
            System(ref str) => {
                match *other {
                    System(ref s) => str == s,
                    _ => false
                }
            },
            Blank => {
                match *other {
                    Blank | SingleWildcard | MultiWildcard => true,
                    _ => false
                }
            },
            SingleWildcard => {
                match *other {
                    System(_) => false,
                    _ => true
                }
            },
            MultiWildcard => {
                match *other {
                    System(_) => false,
                    _ => true
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopicPath {
    pub path: String,
    // Should be false for Topic Name
    pub wildcards: bool,
    topics: Vec<Topic>
}

impl TopicPath {
    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn get(&self, index: usize) -> Option<&Topic> {
        self.topics.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Topic> {
        self.topics.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.topics.len()
    }

    pub fn into_iter(self) -> IntoIter<Topic> {
        self.topics.into_iter()
    }

    pub fn is_final(&self, index: usize) -> bool {
        let len = self.topics.len();
        len == 0 || len-1 == index
    }

    pub fn is_multi(&self, index: usize) -> bool {
        match self.topics.get(index) {
            Some(topic) => *topic == Topic::MultiWildcard,
            None => false
        }
    }
}

impl<'a> From<&'a str> for TopicPath {
    fn from(str: &'a str) -> TopicPath {
        Self::from(String::from(str))
    }
}

impl From<String> for TopicPath {
    fn from(path: String) -> TopicPath {
        let topics: Vec<Topic> = path.split(TOPIC_PATH_DELIMITER).map( |topic| {
            match topic {
                "+" => Topic::SingleWildcard,
                "#" => Topic::MultiWildcard,
                "" => Topic::Blank,
                _ => {
                    if topic.chars().nth(0) == Some('$') {
                        Topic::System(String::from(topic))
                    } else {
                        // FIXME: validate the topic
                        Topic::Normal(String::from(topic))
                    }
                }
            }
        }).collect();

        // check for wildcards
        let wildcards = topics.iter().any(|topic| {
            match *topic {
                Topic::SingleWildcard | Topic::MultiWildcard => true,
                _ => false
            }
        });

        TopicPath {
            path: path,
            topics: topics,
            wildcards: wildcards
        }
    }
}

impl Into<String> for TopicPath {
    fn into(self) -> String {
        self.path
    }
}


pub trait ToTopicPath {
    fn to_topic_path(&self) -> TopicPath;
}

impl ToTopicPath for TopicPath {
    fn to_topic_path(&self) -> TopicPath {
        self.clone()
    }
}

impl ToTopicPath for String {
    fn to_topic_path(&self) -> TopicPath {
        TopicPath::from(self.clone())
    }
}

impl<'a> ToTopicPath for &'a str {
    fn to_topic_path(&self) -> TopicPath {
        TopicPath::from(*self)
    }
}

#[cfg(test)]
mod test {
    use super::{TopicPath, Topic};

    #[test]
    fn topic_path_test() {
        let path = "/$SYS/test/+/#";
        let topic = TopicPath::from(path);
        let mut iter = topic.into_iter();
        assert_eq!(iter.next().unwrap(), Topic::Blank);
        assert_eq!(iter.next().unwrap(), Topic::System("$SYS".to_string()));
        assert_eq!(iter.next().unwrap(), Topic::Normal("test".to_string()));
        assert_eq!(iter.next().unwrap(), Topic::SingleWildcard);
        assert_eq!(iter.next().unwrap(), Topic::MultiWildcard);
    }

    #[test]
    fn wildcards_test() {
        let topic = TopicPath::from("/a/b/c");
        assert!(!topic.wildcards);
        let topic = TopicPath::from("/a/+/c");
        assert!(topic.wildcards);
        let topic = TopicPath::from("/a/b/#");
        assert!(topic.wildcards);
    }
}
