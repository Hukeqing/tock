use rust_decimal::Decimal;
use time::OffsetDateTime;

/// for every stock status in event or pull
#[derive(Debug)]
pub struct Stock {
    /// Security code
    pub symbol: String,
    /// Time of latest price
    pub timestamp: OffsetDateTime,
    /// Latest price
    pub last_done: Decimal,
    /// Open
    pub open: Decimal,
    /// High
    pub high: Decimal,
    /// Low
    pub low: Decimal,
}
