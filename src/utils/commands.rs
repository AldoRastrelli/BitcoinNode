use crate::message_structs::addr::Addr;
use crate::message_structs::addr2::Addr2;
use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::block_txn::BlockTxn;
use crate::message_structs::filter_load_message::FilterLoadMessage;
use crate::message_structs::get_block_message::GetBlockMessage;
use crate::message_structs::get_block_txn::GetBlockTxn;
use crate::message_structs::get_headers_message::GetHeadersMessage;
use crate::message_structs::headers_message::HeadersMessage;
use crate::message_structs::inv_or_get_data_message::InvOrGetDataMessage;
use crate::message_structs::merkel_block::MerkleBlock;
use crate::message_structs::ping_or_pong::PingOrPong;
use crate::message_structs::reject_message::RejectMessage;
use crate::message_structs::tx_message::TXMessage;
use crate::message_structs::version_message::VersionMessage;
use crate::utils::messages::Messages;
use std::error::Error;

#[derive(PartialEq, Debug)]
pub enum MessageType {
    Addr,
    Addr2,
    Verack,
    GetAddr,
    BlockHeader,
    BlockMessage,
    BlockTxn,
    FilterLoadMessage,
    GetBlockMessage,
    GetBlockTxn,
    GetHeadersMessage,
    HeadersMessage,
    InvMessage,
    GetDataMessage,
    MerkleBlock,
    Mempool,
    Ping,
    Pong,
    RejectMessage,
    VersionMessage,
    Tx,
    NotFound,
    Unknown,
    End,
}

/// Returns the Type of a message from its command
pub fn get_type(command: &[u8; 12]) -> MessageType {
    match command {
        // ADDR
        [b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::Addr
        }
        // ADDR2
        [b'a', b'd', b'd', b'r', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::Addr2
        }
        // VERACK
        [b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::Verack
        }
        // GET_ADDR
        [b'g', b'e', b't', b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::GetAddr
        }
        // BLOCK_HEADERS1
        [b'b', b'l', b'o', b'c', b'k', b'h', b'e', b'a', b'd', b'e', b'r', b's'] => {
            MessageType::BlockHeader
        }
        // BLOCK_HEADERS2
        [b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::BlockMessage
        }
        // BLOCK_TXNS
        [b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00, 0x00, 0x00, 0x00] => {
            MessageType::BlockTxn
        }
        // FILTER_LOAD_MESSAGE
        [b'f', b'i', b'l', b't', b'e', b'r', b'l', b'o', b'a', b'd', 0x00, 0x00] => {
            MessageType::FilterLoadMessage
        }
        // GET_BLOCK_MESSAGE
        [b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00] => {
            MessageType::GetBlockMessage
        }
        // HEADERS_MESSAGE
        [b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::HeadersMessage
        }
        // INV_MESSAGE
        [b'i', b'n', b'v', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::InvMessage
        }
        // GET_DATA_MESSAGE
        [b'g', b'e', b't', b'd', b'a', b't', b'a', 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::GetDataMessage
        }
        // MERKEL_BLOCK_MESSAGE
        [b'm', b'e', b'r', b'k', b'l', b'e', b'b', b'l', b'o', b'c', b'k', 0x00] => {
            MessageType::MerkleBlock
        }
        // PING_MESSAGE
        [b'p', b'i', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::Ping
        }
        // PONG_MESSAGE
        [b'p', b'o', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::Pong
        }
        // REJECT_MESSAGE
        [b'r', b'e', b'j', b'e', b'c', b't', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::RejectMessage
        }
        // VERSION_MESSAGE
        [b'v', b'e', b'r', b's', b'i', b'o', b'n', 0x00, 0x00, 0x00, 0x00, 0x00] => {
            MessageType::VersionMessage
        }
        // GET_BLOCK_TXN
        [b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00] => {
            MessageType::GetBlockTxn
        }
        //TX
        [b't', b'x', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => MessageType::Tx,
        // GET_HEADERS_MESSAGE
        [b'g', b'e', b't', b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00] => {
            MessageType::GetHeadersMessage
        }
        // NOT_FOUND
        [b'n', b'o', b't', b'f', b'o', b'u', b'n', b'd', 0x00, 0x00, 0x00, 0x00] => {
            MessageType::NotFound
        }
        //MEMPOOL
        
        [b'm', b'e', b'm', b'p', b'o', b'o', b'l', 0x00, 0x00, 0x00, 0x00, 0x00,]=>MessageType::Mempool,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => MessageType::End,
        _ => MessageType::Unknown,
    }
}

/// Match command to message type and return the message
pub fn match_command(
    command: &[u8; 12],
    payload: &mut Vec<u8>,
) -> Result<Messages, Box<dyn Error>> {
    match get_type(command) {
        MessageType::Addr => {
            let addr = match Addr::deserialize(payload) {
                Ok(a) => a,
                Err(_e) => return Err("Error deserializing Addr".into()),
            };

            Ok(Messages::Addr(addr))
        }

        MessageType::Addr2 => {
            let addr2 = Addr2::deserialize(payload);
            Ok(Messages::Addr2(addr2))
        }

        MessageType::Verack => {
            let verack = BitcoinMessageHeader::verack();
            Ok(Messages::BitcoinMessageHeader(verack))
        }

        MessageType::GetAddr => {
            let get_addr = BitcoinMessageHeader::get_addr();
            Ok(Messages::BitcoinMessageHeader(get_addr))
        }

        MessageType::BlockHeader => {
            let block_headers = match BlockHeader::deserialize(payload) {
                Ok(b) => b,
                Err(_e) => return Err("Error deserializing BlockHeader".into()),
            };

            Ok(Messages::BlockHeader(block_headers))
        }

        MessageType::BlockMessage => {
            let block = match BlockMessage::deserialize(payload) {
                Ok(b) => b,
                Err(_e) => return Err("Error deserializing BlockHeader".into()),
            };

            Ok(Messages::BlockMessage(block))
        }

        MessageType::BlockTxn => {
            let block_txn = match BlockTxn::deserialize(payload) {
                Ok(b) => b,
                Err(_e) => return Err("Error deserializing BlockTxn".into()),
            };

            Ok(Messages::BlockTxn(block_txn))
        }

        MessageType::FilterLoadMessage => {
            let filter_load_message = FilterLoadMessage::deserialize(payload);
            Ok(Messages::FilterLoadMessage(filter_load_message))
        }

        MessageType::GetBlockMessage => {
            let get_block_message = match GetBlockMessage::deserialize(payload) {
                Ok(msg) => msg,
                Err(_e) => return Err("Error deserializing GetBlockMessage".into()),
            };

            Ok(Messages::GetBlockMessage(get_block_message))
        }

        MessageType::HeadersMessage => {
            let headers = match HeadersMessage::deserialize(payload) {
                Ok(h) => h,
                Err(_e) => return Err("Error deserializing HeadersMessage".into()),
            };

            Ok(Messages::HeadersMessage(headers))
        }

        MessageType::InvMessage => {
            let inv_or_get_data = match InvOrGetDataMessage::deserialize(payload) {
                Ok(data) => data,
                Err(_e) => return Err("Error deserializing InvOrGetDataMessage".into()),
            };

            Ok(Messages::InvOrGetDataMessage(inv_or_get_data))
        }

        MessageType::GetDataMessage => {
            let inv_or_get_data = match InvOrGetDataMessage::deserialize(payload) {
                Ok(data) => data,
                Err(_e) => return Err("Error deserializing InvOrGetDataMessage".into()),
            };

            Ok(Messages::InvOrGetDataMessage(inv_or_get_data))
        }

        MessageType::MerkleBlock => {
            let merkel_block = match MerkleBlock::deserialize(payload) {
                Ok(b) => b,
                Err(_e) => return Err("Error deserializing MerkleBlock".into()),
            };

            Ok(Messages::MerkleBlock(merkel_block))
        }

        MessageType::Ping => {
            let ping_pong = PingOrPong::deserialize(payload);
            Ok(Messages::PingOrPong(ping_pong))
        }

        MessageType::Pong => {
            let ping_pong = PingOrPong::deserialize(payload);
            Ok(Messages::PingOrPong(ping_pong))
        }

        MessageType::RejectMessage => {
            let reject_message = match RejectMessage::deserialize(payload) {
                Ok(msg) => msg,
                Err(_e) => return Err("Error deserializing RejectMessage".into()),
            };

            Ok(Messages::RejectMessage(reject_message))
        }

        MessageType::VersionMessage => {
            let version_message = match VersionMessage::deserialize(payload) {
                Ok(msg) => msg,
                Err(_e) => return Err("Error deserializing VersionMessage".into()),
            };

            Ok(Messages::VersionMessage(version_message))
        }

        MessageType::GetBlockTxn => {
            let get_block_message = match GetBlockTxn::deserialize(payload) {
                Ok(msg) => msg,
                Err(_) => {
                    return Err("Error deserializing GetBlockTxn".into());
                }
            };

            Ok(Messages::GetBlockTxn(get_block_message))
        }

        MessageType::Tx => {
            let get_block_message = match TXMessage::deserialize(payload) {
                Ok(msg) => msg,
                Err(_) => {
                    return Err("Error deserializing GetBlockTxn".into());
                }
            };

            Ok(Messages::Tx(get_block_message))
        }

        MessageType::GetHeadersMessage => {
            let get_headers_message = match GetHeadersMessage::deserialize(payload) {
                Ok(msg) => msg,
                Err(_) => return Err("Error deserializing GetHeadersMessage".into()),
            };

            Ok(Messages::GetHeadersMessage(get_headers_message))
        }

        MessageType::Mempool=>{
            let mempool = BitcoinMessageHeader::mempool();
            Ok(Messages::Mempool(mempool))
        }

        MessageType::NotFound => {
            let inv_or_get_data = match InvOrGetDataMessage::deserialize(payload) {
                Ok(data) => data,
                Err(_e) => return Err("Error deserializing InvOrGetDataMessage".into()),
            };

            Ok(Messages::InvOrGetDataMessage(inv_or_get_data))
        }
        MessageType::End => Ok(Messages::End),

        MessageType::Unknown => Ok(Messages::Unknown),
    }
}

#[cfg(test)]

mod commands_tests {
    use super::*;

    #[test]
    fn test_addr() {
        let command = [
            b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::Addr);
    }

    #[test]
    fn test_addr2() {
        let command = [
            b'a', b'd', b'd', b'r', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::Addr2);
    }

    #[test]
    fn test_verack() {
        let command = [
            b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::Verack);
    }

    #[test]
    fn test_get_addr() {
        let command = [
            b'g', b'e', b't', b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::GetAddr);
    }

    #[test]
    fn test_block_header() {
        let command = [
            b'b', b'l', b'o', b'c', b'k', b'h', b'e', b'a', b'd', b'e', b'r', b's',
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::BlockHeader);
    }

    #[test]
    fn test_block() {
        let command = [
            b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::BlockMessage);
    }

    #[test]
    fn test_block_txn() {
        let command = [
            b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::BlockTxn);
    }

    #[test]
    fn test_filter_load() {
        let command = [
            b'f', b'i', b'l', b't', b'e', b'r', b'l', b'o', b'a', b'd', 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::FilterLoadMessage);
    }

    #[test]
    fn test_get_block() {
        let command = [
            b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::GetBlockMessage);
    }

    #[test]
    fn test_headers() {
        let command = [
            b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::HeadersMessage);
    }

    #[test]
    fn test_inv() {
        let command = [
            b'i', b'n', b'v', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::InvMessage);
    }

    #[test]
    fn test_get_data() {
        let command = [
            b'g', b'e', b't', b'd', b'a', b't', b'a', 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::GetDataMessage);
    }

    #[test]
    fn test_merkle_block() {
        let command = [
            b'm', b'e', b'r', b'k', b'l', b'e', b'b', b'l', b'o', b'c', b'k', 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::MerkleBlock);
    }

    #[test]
    fn test_ping() {
        let command = [
            b'p', b'i', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::Ping);
    }

    #[test]
    fn test_pong() {
        let command = [
            b'p', b'o', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::Pong);
    }

    #[test]
    fn test_reject() {
        let command = [
            b'r', b'e', b'j', b'e', b'c', b't', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::RejectMessage);
    }

    #[test]
    fn test_version() {
        let command = [
            b'v', b'e', b'r', b's', b'i', b'o', b'n', 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::VersionMessage);
    }

    #[test]
    fn test_get_block_txn() {
        let command = [
            b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::GetBlockTxn);
    }

    #[test]
    fn test_get_headers() {
        let command = [
            b'g', b'e', b't', b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00,
        ];
        let msg_type = super::get_type(&command);

        assert_eq!(msg_type, MessageType::GetHeadersMessage);
    }
}
