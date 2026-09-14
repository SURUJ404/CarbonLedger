use soroban_sdk::contracterror;

/// Shared error codes used across all CarbonLedger-Lite contracts.
/// Kept identical across contracts so a client only needs one error table.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CarbonError {
    ProjectNotFound = 1,
    ProjectNotVerified = 2,
    ProjectSuspended = 3,
    InsufficientCredits = 4,
    AlreadyRetired = 5,
    SerialNumberConflict = 6,
    UnauthorizedVerifier = 7,
    UnauthorizedOracle = 8,
    InvalidVintageYear = 9,
    ListingNotFound = 10,
    PriceNotSet = 11,
    MonitoringDataStale = 12,
    ZeroAmountNotAllowed = 13,
    ProjectAlreadyExists = 14,
    InvalidSerialRange = 15,
    NotOwner = 16,
    AlreadyListed = 17,
}
