use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionInput {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_optional_amount")]
    pub amount: Option<Decimal>,
}

fn deserialize_optional_amount<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AmountField {
        Value(Decimal),
        Empty(String),
    }

    match Option::<AmountField>::deserialize(deserializer)? {
        Some(AmountField::Value(v)) => Ok(Some(v)),
        Some(AmountField::Empty(s)) if s.trim().is_empty() => Ok(None),
        None => Ok(None),
        Some(AmountField::Empty(s)) => {
            // Try to parse as decimal
            s.trim()
                .parse::<Decimal>()
                .map(Some)
                .map_err(|_| Error::custom(format!("Invalid amount: {}", s)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Normal,
    UnderDispute,
    ChargedBack,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: u32,
    pub client_id: u16,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub state: TransactionState,
}

impl Transaction {
    pub fn new(
        tx_id: u32,
        client_id: u16,
        transaction_type: TransactionType,
        amount: Decimal,
    ) -> Self {
        Transaction {
            tx_id,
            client_id,
            transaction_type,
            amount,
            state: TransactionState::Normal,
        }
    }
}