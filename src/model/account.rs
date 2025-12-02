use rust_decimal::Decimal;
use serde::Serialize;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Account {
    pub client_id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
    pub ordering_lock: Arc<Mutex<()>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AccountOutput {
    pub client: u16,
    #[serde(serialize_with = "serialize_decimal")]
    pub available: Decimal,
    #[serde(serialize_with = "serialize_decimal")]
    pub held: Decimal,
    #[serde(serialize_with = "serialize_decimal")]
    pub total: Decimal,
    pub locked: bool,
}

fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let rounded = value.round_dp(4);
    serializer.serialize_str(&rounded.to_string())
}


impl Account {

    pub fn new(client_id: u16) -> Self {
        Account {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            ordering_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// Returns true if successful, false if account is locked
    pub fn deposit(&mut self, amount: Decimal) -> bool {
        if self.locked {
            return false;
        }

        self.available += amount;
        true
    }

    /// Returns true if successful, false if insufficient funds or account locked
    pub fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.locked || self.available < amount {
            return false;
        }

        self.available -= amount;
        true
    }

    /// Returns true if successful, false if insufficient available funds
    pub fn hold_funds(&mut self, amount: Decimal) -> bool {
        if self.available < amount {
            return false;
        }

        self.available -= amount;
        self.held += amount;
        true
    }

    /// Returns true if successful, false if insufficient held funds
    pub fn release_funds(&mut self, amount: Decimal) -> bool {
        if self.held < amount {
            return false;
        }

        self.held -= amount;
        self.available += amount;
        true
    }

    /// Returns true if successful, false if insufficient held funds
    pub fn chargeback(&mut self, amount: Decimal) -> bool {
        if self.held < amount {
            return false;
        }

        self.held -= amount;
        self.locked = true;
        true
    }

    pub fn to_output(&self) -> AccountOutput {
        AccountOutput {
            client: self.client_id,
            available: self.available,
            held: self.held,
            total: self.total(),
            locked: self.locked,
        }
    }
}