use rust_decimal::Decimal;
use solana_program::pubkey::Pubkey;
use solana_program::clock::Clock;
use std::cell::Ref;
use bytemuck;

pub const PRECISION: u32 = 18;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CurrentResult {
    /// The median value of the submissions needed for quorom size
    pub value: i128,
    /// The standard deviation of the submissions needed for quorom size
    pub std_dev: i128,
    /// The mean of the submissions needed for quorom size
    pub mean: i128,
    /// The range of the submissions needed for quorom size
    pub range: i128,
    /// The minimum value of the submissions needed for quorom size
    pub min_value: i128,
    /// The maximum value of the submissions needed for quorom size
    pub max_value: i128,
    pub padding1: [u8; 8],
    /// The slot at which this value was signed.
    pub slot: u64,
    /// The slot at which the first considered submission was made
    pub min_slot: u64,
    /// The slot at which the last considered submission was made
    pub max_slot: u64,
}
impl CurrentResult {
    /// The median value of the submissions needed for quorom size
    pub fn value(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.value, PRECISION))
    }

    /// The standard deviation of the submissions needed for quorom size
    pub fn std_dev(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.std_dev, PRECISION))
    }

    /// The mean of the submissions needed for quorom size
    pub fn mean(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.mean, PRECISION))
    }

    /// The range of the submissions needed for quorom size
    pub fn range(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.range, PRECISION))
    }

    /// The minimum value of the submissions needed for quorom size
    pub fn min_value(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.min_value, PRECISION))
    }

    /// The maximum value of the submissions needed for quorom size
    pub fn max_value(&self) -> Option<Decimal> {
        if self.slot == 0 {
            return None;
        }
        Some(Decimal::from_i128_with_scale(self.max_value, PRECISION))
    }

    pub fn result_slot(&self) -> Option<u64> {
        if self.slot == 0 {
            return None;
        }
        Some(self.slot)
    }

    pub fn min_slot(&self) -> Option<u64> {
        if self.slot == 0 {
            return None;
        }
        Some(self.min_slot)
    }

    pub fn max_slot(&self) -> Option<u64> {
        if self.slot == 0 {
            return None;
        }
        Some(self.max_slot)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OracleSubmission {
    /// The public key of the oracle that submitted this value.
    pub oracle: Pubkey,
    /// The slot at which this value was signed.
    pub slot: u64,
    padding1: [u8; 8],
    /// The value that was submitted.
    pub value: i128,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompactResult {
    /// The standard deviation of the submissions needed for quorom size
    pub std_dev: f32,
    /// The mean of the submissions needed for quorom size
    pub mean: f32,
    /// The slot at which this value was signed.
    pub slot: u64,
}

/// A representation of the data in a pull feed account.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PullFeedAccountData {
    /// The oracle submissions for this feed.
    pub submissions: [OracleSubmission; 32],
    /// The public key of the authority that can update the feed hash that
    /// this account will use for registering updates.
    pub authority: Pubkey,
    /// The public key of the queue which oracles must be bound to in order to
    /// submit data to this feed.
    pub queue: Pubkey,
    /// SHA-256 hash of the job schema oracles will execute to produce data
    /// for this feed.
    pub feed_hash: [u8; 32],
    /// The slot at which this account was initialized.
    pub initialized_at: i64,
    pub permissions: u64,
    pub max_variance: u64,
    pub min_responses: u32,
    pub name: [u8; 32],
    padding1: [u8; 2],
    pub historical_result_idx: u8,
    pub min_sample_size: u8,
    pub last_update_timestamp: i64,
    pub lut_slot: u64,
    _reserved1: [u8; 32],
    pub result: CurrentResult,
    pub max_staleness: u32,
    padding2: [u8; 12],
    pub historical_results: [CompactResult; 32],
    _ebuf4: [u8; 8],
    _ebuf3: [u8; 24],
    _ebuf2: [u8; 256],
}

impl OracleSubmission {
    pub fn is_empty(&self) -> bool {
        self.slot == 0
    }

    pub fn value(&self) -> Decimal {
        Decimal::from_i128_with_scale(self.value, PRECISION)
    }
}

impl PullFeedAccountData {

    pub fn discriminator() -> [u8; 8] {
        [196, 27, 108, 196, 10, 215, 219, 40]
    }

    pub fn parse<'info>(
        data: Ref<'info, &mut [u8]>,
    ) -> Result<Ref<'info, Self>, OnDemandError> {
        if data.len() < Self::discriminator().len() {
            return Err(OnDemandError::InvalidDiscriminator);
        }

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != Self::discriminator() {
            return Err(OnDemandError::InvalidDiscriminator);
        }

        Ok(Ref::map(data, |data: &&mut [u8]| {
            bytemuck::from_bytes(&data[8..std::mem::size_of::<Self>() + 8])
        }))
    }

    /// **method**
    /// get_value
    /// Returns the median value of the submissions in the last `max_staleness` slots.
    /// If there are fewer than `min_samples` submissions, returns an error.
    /// **arguments**
    /// * `clock` - the clock to use for the current slot
    /// * `max_staleness` - the maximum number of slots to consider
    /// * `min_samples` - the minimum number of samples required to return a value
    /// **returns**
    /// * `Ok(Decimal)` - the median value of the submissions in the last `max_staleness` slots
    pub fn get_value(
        &self,
        clock: &Clock,
        max_staleness: u64,
        min_samples: u32,
        only_positive: bool,
    ) -> Result<Decimal, OnDemandError> {
        let submissions = self
            .submissions
            .iter()
            .take_while(|s| !s.is_empty())
            .filter(|s| s.slot > clock.slot - max_staleness)
            .collect::<Vec<_>>();
        if submissions.len() < min_samples as usize {
            return Err(OnDemandError::NotEnoughSamples);
        }
        let median =
            lower_bound_median(&mut submissions.iter().map(|s| s.value).collect::<Vec<_>>())
            .ok_or(OnDemandError::NotEnoughSamples)?;
        if only_positive && median <= 0 {
            return Err(OnDemandError::IllegalFeedValue);
        }

        Ok(Decimal::from_i128_with_scale(median, PRECISION))
    }

    /// The median value of the submissions needed for quorom size
    pub fn value(&self) -> Option<Decimal> {
        self.result.value()
    }

    /// The standard deviation of the submissions needed for quorom size
    pub fn std_dev(&self) -> Option<Decimal> {
        self.result.std_dev()
    }

    /// The mean of the submissions needed for quorom size
    pub fn mean(&self) -> Option<Decimal> {
        self.result.mean()
    }

    /// The range of the submissions needed for quorom size
    pub fn range(&self) -> Option<Decimal> {
        self.result.range()
    }

    /// The minimum value of the submissions needed for quorom size
    pub fn min_value(&self) -> Option<Decimal> {
        self.result.min_value()
    }

    /// The maximum value of the submissions needed for quorom size
    pub fn max_value(&self) -> Option<Decimal> {
        self.result.max_value()
    }
}

// takes the rounded down median of a list of numbers
pub fn lower_bound_median(numbers: &mut Vec<i128>) -> Option<i128> {
    numbers.sort(); // Sort the numbers in ascending order.

    let len = numbers.len();
    if len == 0 {
        return None; // Return None for an empty list.
    }
    Some(numbers[len / 2])
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum OnDemandError {
    Generic,
    AccountBorrowError,
    AccountNotFound,
    AnchorParse,
    AnchorParseError,
    CheckSizeError,
    DecimalConversionError,
    DecryptError,
    EventListenerRoutineFailure,
    EvmError,
    FunctionResultIxIncorrectTargetChain,
    HeartbeatRoutineFailure,
    IntegerOverflowError,
    InvalidChain,
    InvalidData,
    InvalidDiscriminator,
    InvalidInstructionError,
    InvalidKeypairFile,
    InvalidNativeMint,
    InvalidQuote,
    InvalidQuoteError,
    InvalidSignature,
    IpfsNetworkError,
    IpfsParseError,
    KeyParseError,
    MrEnclaveMismatch,
    NetworkError,
    ParseError,
    PdaDerivationError,
    QuoteParseError,
    QvnTxSendFailure,
    SgxError,
    SgxWriteError,
    SolanaBlockhashError,
    SolanaMissingSigner,
    SolanaPayerSignerMissing,
    SolanaPayerMismatch,
    SolanaInstructionOverflow,
    SolanaInstructionsEmpty,
    TxCompileErr,
    TxDeserializationError,
    TxFailure,
    Unexpected,
    SolanaSignError,
    IoError,
    KeyDerivationFailed,
    InvalidSecretKey,
    EnvVariableMissing,
    AccountDeserializeError,
    NotEnoughSamples,
    IllegalFeedValue,
    CustomMessage(String),
    SwitchboardRandomnessTooOld,
    AddressLookupTableFetchError,
    AddressLookupTableDeserializeError,
}