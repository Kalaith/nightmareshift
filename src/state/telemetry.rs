//! Balance-measurement records retained by the live simulation.

/// One fare's contribution to the shift total, retained for balance reports.
///
/// The outcome card only needs the most recent ride, but campaign measurement
/// needs the whole distribution so a single generous passenger cannot hide an
/// otherwise impossible quota.
#[derive(Debug, Clone)]
pub struct FareContribution {
    pub passenger_id: u32,
    pub passenger_name: String,
    pub fare: u32,
}
