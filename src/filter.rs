use crate::message_cached::MessageCached;
use bitflags::bitflags;
use oze_canopen::canopen::{NodeId, RxMessageType};
use regex::Regex;
use std::{cell::RefCell, rc::Rc};

/// Represents a filter for CAN messages based on node id and regular expressions.
#[derive(Default, Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DataFilter {
    pub regex: Option<Regex>,
    pub node_id: Option<NodeId>,
    pub regex_cob: Option<Regex>,
}

/// Represents a global filter that includes data filters and flag-based type filters.
#[derive(Default, Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct GlobalFilter {
    pub ignore_type: Flags,
    pub data: Rc<RefCell<DataFilter>>,
}

bitflags! {
    /// Flags for different types of CAN messages.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        const NONE = 0b0000_0000;
        const SYNC = 0b0000_0001;
        const PDO = 0b0000_0010;
        const SDO = 0b0000_0100;
        const NMT = 0b0000_1000;
        const LSS = 0b0001_0000;
        const EMCY = 0b0010_0000;
        const GUARD = 0b0100_0000;
        const UNKNOWN = 0b1000_0000;
        const ALL = 0b1111_1111;
    }
}

impl DataFilter {
    /// Filters messages based on node id and regular expressions.
    ///
    /// Returns `true` if the message should be filtered out, `false` otherwise.
    pub fn filter(&self, msg: &MessageCached) -> bool {
        if self.node_id.is_some() && msg.msg.parsed_node_id != self.node_id {
            return true;
        }

        if let Some(re) = &self.regex_cob {
            if !re.is_match(&msg.cob_str) {
                return true;
            }
        }

        if let Some(re) = &self.regex {
            if !re.is_match(&msg.hex_str) {
                return true;
            }
        }

        false
    }
}

impl GlobalFilter {
    /// Filters messages based on data filters and message type flags.
    ///
    /// Returns `true` if the message should be filtered out, `false` otherwise.
    pub fn filter(&self, msg: &MessageCached) -> bool {
        if self.data.borrow().filter(msg) {
            return true;
        }

        match msg.msg.parsed_type {
            RxMessageType::SdoTx | RxMessageType::SdoRx => self.ignore_type.contains(Flags::SDO),
            RxMessageType::Pdo => self.ignore_type.contains(Flags::PDO),
            RxMessageType::Sync => self.ignore_type.contains(Flags::SYNC),
            RxMessageType::Nmt => self.ignore_type.contains(Flags::NMT),
            RxMessageType::Lss => self.ignore_type.contains(Flags::LSS),
            RxMessageType::Guarding => self.ignore_type.contains(Flags::GUARD),
            RxMessageType::Emcy => self.ignore_type.contains(Flags::EMCY),
            RxMessageType::Unknown => self.ignore_type.contains(Flags::UNKNOWN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DataFilter, GlobalFilter};
    use crate::message_cached::MessageCached;
    use oze_canopen::receiver::RxMessage;
    use regex::Regex;
    use std::{cell::RefCell, rc::Rc};
    use tokio::time::Instant;

    #[test]
    fn test_data_filter() {
        // Create test messages
        let msg183 = RxMessage {
            timestamp: Instant::now(),
            cob_id: 0x183,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
            dlc: 3,
        };

        let msg585 = RxMessage {
            timestamp: Instant::now(),
            cob_id: 0x585,
            data: [0xAB, 0xFE, 3, 4, 5, 6, 7, 8],
            dlc: 8,
        };

        let msg183 = &MessageCached::new(0, msg183);
        let msg585 = &MessageCached::new(0, msg585);

        // Test various filters
        let filt = DataFilter {
            regex: Regex::new("^01").ok(),
            node_id: None,
            regex_cob: None,
        };
        assert!(!filt.filter(msg183));
        assert!(filt.filter(msg585));

        let filt = DataFilter {
            regex: Regex::new("^01 02 03$").ok(),
            node_id: None,
            regex_cob: None,
        };
        assert!(!filt.filter(msg183));
        assert!(filt.filter(msg585));

        let filt = DataFilter {
            regex: Regex::new("03").ok(),
            node_id: None,
            regex_cob: None,
        };
        assert!(!filt.filter(msg183));
        assert!(!filt.filter(msg585));

        let filt = DataFilter {
            regex: Regex::new("03").ok(),
            node_id: Some(3),
            regex_cob: None,
        };
        assert!(!filt.filter(msg183));
        assert!(filt.filter(msg585));

        let filt = DataFilter {
            regex: Regex::new("03").ok(),
            node_id: Some(5),
            regex_cob: None,
        };
        assert!(filt.filter(msg183));
        assert!(!filt.filter(msg585));

        let filt = DataFilter {
            regex: None,
            node_id: Some(5),
            regex_cob: Regex::new("^58").ok(),
        };
        assert!(filt.filter(msg183));
        assert!(!filt.filter(msg585));

        let filt = DataFilter {
            regex: None,
            node_id: Some(5),
            regex_cob: Regex::new("^18").ok(),
        };
        assert!(filt.filter(msg183));
        assert!(filt.filter(msg585));

        let filt = DataFilter {
            regex: Regex::new("AB").ok(),
            node_id: Some(5),
            regex_cob: Regex::new("58").ok(),
        };
        assert!(filt.filter(msg183));
        assert!(!filt.filter(msg585));
    }

    #[test]
    fn test_global_filter() {
        // Create test messages
        let msg183 = RxMessage {
            timestamp: Instant::now(),
            cob_id: 0x183,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
            dlc: 3,
        };

        let msg585 = RxMessage {
            timestamp: Instant::now(),
            cob_id: 0x585,
            data: [0xAB, 0xFE, 3, 4, 5, 6, 7, 8],
            dlc: 8,
        };

        let msg80 = RxMessage {
            timestamp: Instant::now(),
            cob_id: 0x080,
            data: [0, 0, 0, 0, 0, 0, 0, 0],
            dlc: 0,
        };

        let msg183 = &MessageCached::new(0, msg183);
        let msg585 = &MessageCached::new(0, msg585);
        let msg80 = &MessageCached::new(0, msg80);

        // Test various global filters
        let filt = GlobalFilter {
            ignore_type: super::Flags::NONE,
            data: Rc::new(RefCell::new(DataFilter {
                regex: None,
                node_id: None,
                regex_cob: None,
            })),
        };
        assert!(!filt.filter(msg183));
        assert!(!filt.filter(msg585));
        assert!(!filt.filter(msg80));

        let filt = GlobalFilter {
            ignore_type: super::Flags::all() & !super::Flags::PDO,
            data: Rc::new(RefCell::new(DataFilter {
                regex: None,
                node_id: Some(3),
                regex_cob: None,
            })),
        };
        assert!(!filt.filter(msg183));
        assert!(filt.filter(msg585));
        assert!(filt.filter(msg80));

        let filt = GlobalFilter {
            ignore_type: super::Flags::all() & !super::Flags::PDO,
            data: Rc::new(RefCell::new(DataFilter {
                regex: None,
                node_id: Some(5),
                regex_cob: None,
            })),
        };
        assert!(filt.filter(msg183));
        assert!(filt.filter(msg585));
        assert!(filt.filter(msg80));

        let filt = GlobalFilter {
            ignore_type: (super::Flags::all() & !super::Flags::PDO) & !super::Flags::SYNC,
            data: Rc::new(RefCell::new(DataFilter {
                regex: None,
                node_id: None,
                regex_cob: None,
            })),
        };
        assert!(!filt.filter(msg183));
        assert!(filt.filter(msg585));
        assert!(!filt.filter(msg80));

        let filt = GlobalFilter {
            ignore_type: super::Flags::all(),
            data: Rc::new(RefCell::new(DataFilter {
                regex: None,
                node_id: None,
                regex_cob: None,
            })),
        };
        assert!(filt.filter(msg183));
        assert!(filt.filter(msg585));
        assert!(filt.filter(msg80));
    }
}
