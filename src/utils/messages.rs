use crate::message_structs::addr::Addr;
use crate::message_structs::addr2::Addr2;
use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::block_txn::BlockTxn;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::filter_load_message::FilterLoadMessage;
use crate::message_structs::get_block_message::GetBlockMessage;
use crate::message_structs::get_block_txn::GetBlockTxn;
use crate::message_structs::get_headers_message::GetHeadersMessage;
use crate::message_structs::headers_message::HeadersMessage;
use crate::message_structs::inv::Inv;
use crate::message_structs::inv_or_get_data_message::InvOrGetDataMessage;
use crate::message_structs::merkel_block::MerkleBlock;
use crate::message_structs::ping_or_pong::PingOrPong;
use crate::message_structs::reject_message::RejectMessage;
use crate::message_structs::tx_message::TXMessage;
use crate::message_structs::version_message::VersionMessage;

#[derive(Debug, PartialEq)]
pub enum Messages {
    Addr(Addr),
    Addr2(Addr2),
    BitcoinMessageHeader(BitcoinMessageHeader),
    BlockHeader(BlockHeader),
    BlockMessage(BlockMessage),
    BlockTxn(BlockTxn),
    FilterLoadMessage(FilterLoadMessage),
    GetBlockMessage(GetBlockMessage),
    GetBlockTxn(GetBlockTxn),
    GetHeadersMessage(GetHeadersMessage),
    HeadersMessage(HeadersMessage),
    InvOrGetDataMessage(InvOrGetDataMessage),
    MerkleBlock(MerkleBlock),
    PingOrPong(PingOrPong),
    RejectMessage(RejectMessage),
    Tx(TXMessage),
    VersionMessage(VersionMessage),
    Mempool(BitcoinMessageHeader),
    End,
    Unknown,
}

impl Messages {
    pub fn create_get_data(&self) -> InvOrGetDataMessage {
        let get_data;
        if let Messages::HeadersMessage(headers) = self {
            get_data = headers.create_get_data();
        } else {
            get_data = InvOrGetDataMessage::new(
                CompactSize {
                    prefix: 0,
                    number_vec: vec![1],
                    number: 1,
                },
                vec![Inv::new(1, [0u8; 32])],
            );
        };
        get_data
    }

    pub fn addr(msg: Addr) -> Self {
        Messages::Addr(msg)
    }

    pub fn addr2(msg: Addr2) -> Self {
        Messages::Addr2(msg)
    }

    pub fn bitcoin_message_header(msg: BitcoinMessageHeader) -> Self {
        Messages::BitcoinMessageHeader(msg)
    }

    pub fn block_header(msg: BlockHeader) -> Self {
        Messages::BlockHeader(msg)
    }

    pub fn block_message(msg: BlockMessage) -> Self {
        Messages::BlockMessage(msg)
    }

    pub fn block_txn(msg: BlockTxn) -> Self {
        Messages::BlockTxn(msg)
    }

    pub fn filter_load_message(msg: FilterLoadMessage) -> Self {
        Messages::FilterLoadMessage(msg)
    }

    pub fn get_block_message(msg: GetBlockMessage) -> Self {
        Messages::GetBlockMessage(msg)
    }

    pub fn get_block_txn(msg: GetBlockTxn) -> Self {
        Messages::GetBlockTxn(msg)
    }

    pub fn get_headers_message(msg: GetHeadersMessage) -> Self {
        Messages::GetHeadersMessage(msg)
    }

    pub fn headers_message(msg: HeadersMessage) -> Self {
        Messages::HeadersMessage(msg)
    }

    pub fn inv_or_get_data(msg: InvOrGetDataMessage) -> Self {
        Messages::InvOrGetDataMessage(msg)
    }

    pub fn merkel_block(msg: MerkleBlock) -> Self {
        Messages::MerkleBlock(msg)
    }

    pub fn ping_or_pong(msg: PingOrPong) -> Self {
        Messages::PingOrPong(msg)
    }

    pub fn reject_message(msg: RejectMessage) -> Self {
        Messages::RejectMessage(msg)
    }

    pub fn version_message(msg: VersionMessage) -> Self {
        Messages::VersionMessage(msg)
    }

    pub fn tx_message(msg: TXMessage) -> Self {
        Messages::Tx(msg)
    }

    pub fn mempool(msg: BitcoinMessageHeader)->Self{
        Messages::Mempool(msg)
    }

    pub fn unknown() -> Self {
        Messages::Unknown
    }
}
