use codec::Encode;
use pallet_mixnet::types::{
    Ballot, PublicKey as SubstratePK, PublicParameters, Title, Topic, VoteId, VotePhase,
};
use substrate_subxt::{Call, EventsDecoder, NodeTemplateRuntime};

#[derive(Encode)]
pub struct CreateVote {
    pub vote_id: VoteId,
    pub title: Title,
    pub params: PublicParameters,
    pub topics: Vec<Topic>,
    pub batch_size: u64,
}

impl Call<NodeTemplateRuntime> for CreateVote {
    const MODULE: &'static str = "PalletMixnet";
    const FUNCTION: &'static str = "create_vote";
    fn events_decoder(_decoder: &mut EventsDecoder<NodeTemplateRuntime>) {
        _decoder.register_type_size::<VoteId>("VoteId");
        _decoder.register_type_size::<Title>("Title");
        _decoder.register_type_size::<PublicParameters>("PublicParameters");
        _decoder.register_type_size::<Vec<Topic>>("Vec<Topic>");
        _decoder.register_type_size::<u64>("batch_size");
    }
}

#[derive(Encode)]
pub struct StorePublicKey {
    pub vote_id: VoteId,
    pub pk: SubstratePK,
}

impl Call<NodeTemplateRuntime> for StorePublicKey {
    const MODULE: &'static str = "PalletMixnet";
    const FUNCTION: &'static str = "store_public_key";
    fn events_decoder(_decoder: &mut EventsDecoder<NodeTemplateRuntime>) {
        _decoder.register_type_size::<VoteId>("VoteId");
        _decoder.register_type_size::<SubstratePK>("SubstratePK");
    }
}

#[derive(Encode)]
pub struct SetVotePhase {
    pub vote_id: VoteId,
    pub vote_phase: VotePhase,
}

impl Call<NodeTemplateRuntime> for SetVotePhase {
    const MODULE: &'static str = "PalletMixnet";
    const FUNCTION: &'static str = "set_vote_phase";
    fn events_decoder(_decoder: &mut EventsDecoder<NodeTemplateRuntime>) {
        _decoder.register_type_size::<VoteId>("VoteId");
        _decoder.register_type_size::<VotePhase>("VotePhase");
    }
}

#[derive(Encode)]
pub struct CastBallot {
    pub vote_id: VoteId,
    pub ballot: Ballot,
}

impl Call<NodeTemplateRuntime> for CastBallot {
    const MODULE: &'static str = "PalletMixnet";
    const FUNCTION: &'static str = "cast_ballot";
    fn events_decoder(_decoder: &mut EventsDecoder<NodeTemplateRuntime>) {
        _decoder.register_type_size::<VoteId>("VoteId");
        _decoder.register_type_size::<Ballot>("Ballot");
    }
}
