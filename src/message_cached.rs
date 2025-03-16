use core::fmt;
use oze_canopen::{
    canopen::{RxMessage, RxMessageParsed, RxMessageToStringFormat, RxMessageType},
    proto::{
        emcy::Emcy,
        nmt::NmtCommand,
        sdo::{ResponseData, SdoRequest, SdoRequestData, SdoResponse},
    },
    BinRead,
};
use std::io::Cursor;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub enum RxMessageAdditional {
    SdoTx(SdoResponse),
    SdoRx(SdoRequest),
    Nmt(NmtCommand),
    Emcy(Emcy),
    None,
}

#[derive(Debug, Clone)]
pub struct MessageCached {
    pub index: u64,
    pub msg: RxMessageParsed,
    pub additional: RxMessageAdditional,
    pub cob_str: String,
    pub hex_str: String,
    pub bin_str: String,
    pub ascii_str: String,
}

impl RxMessageAdditional {
    fn from_server_resp_data(d: &ResponseData) -> String {
        match d {
            ResponseData::Download(i) => {
                format!(
                    "Resp Download ind={:X} sub={:X} size={:X}",
                    i.index, i.subindex, i.size
                )
            }
            ResponseData::DownloadSegment(i) => {
                format!("Resp DownSeg  {:X?}", i.data)
            }
            ResponseData::Upload(i) => {
                format!(
                    "Resp Upload   ind={:X} sub={:X} {:X?}",
                    i.index, i.subindex, i.data
                )
            }
            ResponseData::UploadSegment(i) => {
                format!("Resp UpSeg    {:X?}", i.data)
            }
            ResponseData::Abort(i) => {
                format!(
                    "Resp Abort    ind={:X} sub={:X} reason={}",
                    i.index,
                    i.subindex,
                    i.reason.to_str()
                )
            }
        }
    }

    fn from_req_data(sdo_request: &SdoRequestData) -> String {
        match sdo_request {
            SdoRequestData::InitiateDownload(req_initial) => {
                format!(
                    "Req  InitDown ind {:X?} sub {:X?} size {:X?}",
                    req_initial.index, req_initial.subindex, req_initial.size
                )
            }
            SdoRequestData::DownloadSegment(req) => {
                format!("Req  DownSeg  {:X?}", req.data)
            }
            SdoRequestData::InitiateUpload(req) => {
                format!(
                    "Req  InitUp   ind {:X} sub {:X} size {:X?}",
                    req.index, req.subindex, req.size
                )
            }
            SdoRequestData::InitiateDownloadExpedited(req) => {
                format!(
                    "Req  InitDown ind {:X} sub {:X} data {:X?}",
                    req.index, req.subindex, req.data
                )
            }
            SdoRequestData::UploadSegment(data) => {
                format!("Req  UpSeg    {data:X?}")
            }
        }
    }

    pub fn get_tooltip(&self) -> String {
        match &self {
            RxMessageAdditional::SdoRx(server_request) => server_request
                .cmd
                .iter_names()
                .map(|i| i.0)
                .collect::<Vec<_>>()
                .join(","),
            RxMessageAdditional::SdoTx(server_response) => server_response
                .cmd
                .iter_names()
                .map(|i| i.0)
                .collect::<Vec<_>>()
                .join(","),
            RxMessageAdditional::Nmt(n) => {
                format!("{n:?}")
            }
            RxMessageAdditional::Emcy(n) => {
                format!("{n:?}")
            }
            RxMessageAdditional::None => String::new(),
        }
    }
}

impl fmt::Display for RxMessageAdditional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RxMessageAdditional::SdoRx(server_request) => {
                write!(f, "{}", &Self::from_req_data(&server_request.req))
            }
            RxMessageAdditional::SdoTx(server_response) => {
                write!(f, "{}", Self::from_server_resp_data(&server_response.resp))
            }
            RxMessageAdditional::Nmt(n) => {
                write!(f, "{:?} node_id: {}", n.command_specifier, n.node_id)
            }
            RxMessageAdditional::Emcy(n) => {
                write!(
                    f,
                    "{:?} {:X} reg {:X} data {:X?}",
                    n.code, n.vendor_code, n.error_register, n.data
                )
            }
            RxMessageAdditional::None => write!(f, ""),
        }
    }
}

impl MessageCached {
    /// # Panics
    pub fn new(index: u64, msg: RxMessage) -> Self {
        let parsed = RxMessageParsed::new(msg);
        let mut dat = Cursor::new(msg.data);

        let additional = match parsed.parsed_type {
            RxMessageType::SdoTx => {
                let s = SdoResponse::read(&mut dat);
                if let Ok(s) = s {
                    RxMessageAdditional::SdoTx(s)
                } else {
                    RxMessageAdditional::None
                }
            }

            RxMessageType::SdoRx => {
                if let Ok(d) = SdoRequest::read(&mut dat) {
                    RxMessageAdditional::SdoRx(d)
                } else {
                    RxMessageAdditional::None
                }
            }
            RxMessageType::Nmt => {
                if let Ok(d) = NmtCommand::read(&mut dat) {
                    RxMessageAdditional::Nmt(d)
                } else {
                    RxMessageAdditional::None
                }
            }
            RxMessageType::Emcy => {
                if let Ok(d) = Emcy::read(&mut dat) {
                    RxMessageAdditional::Emcy(d)
                } else {
                    RxMessageAdditional::None
                }
            }
            RxMessageType::Guarding
            | RxMessageType::Lss
            | RxMessageType::Pdo
            | RxMessageType::Sync
            | RxMessageType::Unknown => RxMessageAdditional::None,
        };

        Self {
            index,
            msg: parsed,
            additional,
            cob_str: msg.cob_id_to_string(),
            hex_str: msg.data_to_string(RxMessageToStringFormat::Hex),
            bin_str: msg.data_to_string(RxMessageToStringFormat::Binary),
            ascii_str: msg.data_to_string(RxMessageToStringFormat::Ascii),
        }
    }

    /// # Panics
    pub fn get_by_format(&self, format: RxMessageToStringFormat) -> &str {
        assert_ne!(format, RxMessageToStringFormat::Utf8);
        match format {
            RxMessageToStringFormat::Binary => &self.bin_str,
            RxMessageToStringFormat::Ascii => &self.ascii_str,
            RxMessageToStringFormat::Hex => &self.hex_str,
            RxMessageToStringFormat::Utf8 => "utf8 not supported",
        }
    }

    pub fn get_timestamp(&self) -> Instant {
        self.msg.msg.timestamp
    }
}
