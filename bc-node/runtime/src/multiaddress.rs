use codec::{Codec, Decode, Encode};
use sp_runtime::{
    traits::{LookupError, StaticLookup},
    RuntimeDebug,
};
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

/// A multi-format address wrapper for on-chain accounts.
#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum MultiAddress<AccountId, AccountIndex> {
    /// It's an account ID (pubkey).
    Id(AccountId),
    /// It's an account index.
    Index(#[codec(compact)] AccountIndex),
    /// It's some arbitrary raw bytes.
    Raw(Vec<u8>),
    /// It's a 32 byte representation.
    Address32([u8; 32]),
    /// Its a 20 byte representation.
    Address20([u8; 20]),
}

#[cfg(feature = "std")]
impl<AccountId, AccountIndex> std::fmt::Display for MultiAddress<AccountId, AccountIndex>
where
    AccountId: std::fmt::Debug,
    AccountIndex: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use sp_core::hexdisplay::HexDisplay;
        match self {
            MultiAddress::Raw(inner) => write!(f, "MultiAddress::Raw({})", HexDisplay::from(inner)),
            MultiAddress::Address32(inner) => {
                write!(f, "MultiAddress::Address32({})", HexDisplay::from(inner))
            }
            MultiAddress::Address20(inner) => {
                write!(f, "MultiAddress::Address20({})", HexDisplay::from(inner))
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl<AccountId, AccountIndex> From<AccountId> for MultiAddress<AccountId, AccountIndex> {
    fn from(a: AccountId) -> Self {
        MultiAddress::Id(a)
    }
}

impl<AccountId: Default, AccountIndex> Default for MultiAddress<AccountId, AccountIndex> {
    fn default() -> Self {
        MultiAddress::Id(Default::default())
    }
}

/// A lookup implementation returning the `AccountId` from a `MultiAddress`.
pub struct AccountIdLookup<AccountId, AccountIndex>(PhantomData<(AccountId, AccountIndex)>);
impl<AccountId, AccountIndex> StaticLookup for AccountIdLookup<AccountId, AccountIndex>
where
    AccountId: Codec + Clone + PartialEq + Debug,
    AccountIndex: Codec + Clone + PartialEq + Debug,
    MultiAddress<AccountId, AccountIndex>: Codec,
{
    type Source = MultiAddress<AccountId, AccountIndex>;
    type Target = AccountId;
    fn lookup(x: Self::Source) -> Result<Self::Target, LookupError> {
        match x {
            MultiAddress::Id(i) => Ok(i),
            _ => Err(LookupError),
        }
    }
    fn unlookup(x: Self::Target) -> Self::Source {
        MultiAddress::Id(x)
    }
}
