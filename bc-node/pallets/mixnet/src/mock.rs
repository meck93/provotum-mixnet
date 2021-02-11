use crate as pallet_mixnet;
use codec::alloc::sync::Arc;
use codec::Decode;
use frame_support::{construct_runtime, parameter_types};
use hex_literal::hex;
use parking_lot::RwLock;
use sp_core::{
    offchain::{
        testing::{self, OffchainState, PoolState},
        OffchainExt, TransactionPoolExt,
    },
    sr25519::{self, Signature},
    H256,
};
use sp_io::TestExternalities;
use sp_keystore::{
    testing::KeyStore,
    {KeystoreExt, SyncCryptoStore},
};
use sp_runtime::{
    app_crypto::AppKey,
    testing::{Header, TestXt},
    traits::{BlakeTwo256, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        OffchainModule: pallet_mixnet::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

// The TestRuntime implements two pallet/frame traits: frame_system, and simple_event
impl frame_system::Config for TestRuntime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

// --- mocking offchain-worker trait

pub type TestExtrinsic = TestXt<Call, ()>;

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
    Call: From<LocalCall>,
{
    fn create_transaction<
        C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
    >(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: <TestRuntime as frame_system::Config>::AccountId,
        index: <TestRuntime as frame_system::Config>::Index,
    ) -> Option<(
        Call,
        <TestExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        Some((call, (index, ())))
    }
}

impl frame_system::offchain::SigningTypes for TestRuntime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = TestExtrinsic;
}

////////////////////////////////////////
/// Mock Implementation of pallet_mixnet
impl pallet_mixnet::Config for TestRuntime {
    type Call = Call;
    type Event = Event;
    type AuthorityId = pallet_mixnet::keys::TestAuthId;
}

pub struct ExternalityBuilder;

impl ExternalityBuilder {
    fn initialize_test_authorities() -> (
        Vec<<TestRuntime as frame_system::Config>::AccountId>,
        Vec<<TestRuntime as frame_system::Config>::AccountId>,
    ) {
        // use Alice as VotingAuthority
        let alice_account_id: [u8; 32] =
            hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
                .into();

        let voting_authority: <TestRuntime as frame_system::Config>::AccountId =
            <TestRuntime as frame_system::Config>::AccountId::decode(
                &mut &alice_account_id[..],
            )
            .unwrap();

        // Use Bob, Charlie, Dave as Sealers
        let bob_account_id: [u8; 32] =
            hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
                .into();

        let sealer1: <TestRuntime as frame_system::Config>::AccountId =
            <TestRuntime as frame_system::Config>::AccountId::decode(
                &mut &bob_account_id[..],
            )
            .unwrap();

        let charlie_account_id: [u8; 32] =
            hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22")
                .into();

        let sealer2: <TestRuntime as frame_system::Config>::AccountId =
            <TestRuntime as frame_system::Config>::AccountId::decode(
                &mut &charlie_account_id[..],
            )
            .unwrap();

        // let dave_account_id: [u8; 32] =
        //     hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eebbf2314649965fe22")
        //         .into();

        // let sealer3: <TestRuntime as frame_system::Config>::AccountId =
        //     <TestRuntime as frame_system::Config>::AccountId::decode(&mut &dave_account_id[..])
        //         .unwrap();

        let voting_authorities = vec![voting_authority];
        // let sealers = vec![sealer1, sealer2, sealer3];
        let sealers = vec![sealer1, sealer2];
        (voting_authorities, sealers)
    }

    pub fn build() -> (
        TestExternalities,
        Arc<RwLock<PoolState>>,
        Arc<RwLock<OffchainState>>,
    ) {
        const PHRASE: &str =
            "expire stage crawl shell boss any story swamp skull yellow bamboo copy";

        let (offchain, offchain_state) = testing::TestOffchainExt::new();
        let (pool, pool_state) = testing::TestTransactionPoolExt::new();
        let keystore = KeyStore::new();
        SyncCryptoStore::sr25519_generate_new(
            &keystore,
            crate::keys::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let (voting_authorities, sealers) = Self::initialize_test_authorities();

        super::GenesisConfig::<TestRuntime> {
            voting_authorities,
            sealers,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let mut t = TestExternalities::from(storage);
        t.register_extension(OffchainExt::new(offchain));
        t.register_extension(TransactionPoolExt::new(pool));
        t.register_extension(KeystoreExt(Arc::new(keystore)));
        t.execute_with(|| System::set_block_number(1));
        (t, pool_state, offchain_state)
    }
}
