use serde::{Deserialize, Serialize};

/// Multi-denomination currency purse.
///
/// Exchange rates (all expressed in copper):
///   1 platinum = 1000 cp  (= 10 gp)
///   1 gold     =  100 cp  (= 10 sp)
///   1 silver   =   10 cp
///   1 copper   =    1 cp
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Purse {
    pub platinum: u32,
    pub gold: u32,
    pub silver: u32,
    pub copper: u32,
}

impl Purse {
    /// Total value expressed in copper pieces.
    pub fn total_copper(&self) -> u64 {
        self.platinum as u64 * 1000
            + self.gold as u64 * 100
            + self.silver as u64 * 10
            + self.copper as u64
    }

    /// Add an amount of the given denomination.
    /// Returns Err for unknown denominations.
    pub fn add(&mut self, denomination: &str, amount: u32) -> Result<(), String> {
        match denomination.to_lowercase().as_str() {
            "platinum" | "pp" | "plat" => self.platinum += amount,
            "gold"     | "gp"          => self.gold     += amount,
            "silver"   | "sp"          => self.silver   += amount,
            "copper"   | "cp"          => self.copper   += amount,
            other => return Err(format!("Unknown denomination: {}", other)),
        }
        Ok(())
    }

    /// Remove an amount of the given denomination, making change from larger
    /// denominations as needed. Returns Err if the purse has insufficient funds.
    pub fn remove(&mut self, denomination: &str, amount: u32) -> Result<(), String> {
        let cost_cp: u64 = match denomination.to_lowercase().as_str() {
            "platinum" | "pp" | "plat" => amount as u64 * 1000,
            "gold"     | "gp"          => amount as u64 * 100,
            "silver"   | "sp"          => amount as u64 * 10,
            "copper"   | "cp"          => amount as u64,
            other => return Err(format!("Unknown denomination: {}", other)),
        };

        let total = self.total_copper();
        if cost_cp > total {
            return Err(format!(
                "Insufficient funds: have {}, need {}{}",
                self.to_context_string(), amount, denomination
            ));
        }

        // Rebuild purse from remaining copper total (normalises automatically)
        let remaining = total - cost_cp;
        self.platinum = (remaining / 1000) as u32;
        let remaining = remaining % 1000;
        self.gold     = (remaining / 100) as u32;
        let remaining = remaining % 100;
        self.silver   = (remaining / 10) as u32;
        self.copper   = (remaining % 10) as u32;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.total_copper() == 0
    }

    /// Compact string for the AI context header, e.g. "12gp 3sp 7cp".
    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();
        if self.platinum > 0 { parts.push(format!("{}pp", self.platinum)); }
        if self.gold     > 0 { parts.push(format!("{}gp", self.gold)); }
        if self.silver   > 0 { parts.push(format!("{}sp", self.silver)); }
        if self.copper   > 0 { parts.push(format!("{}cp", self.copper)); }
        if parts.is_empty()  { "0cp".to_string() }
        else                 { parts.join(" ") }
    }
}
