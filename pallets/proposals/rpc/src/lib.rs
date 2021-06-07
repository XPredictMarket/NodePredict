use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use proposals_info_runtime_api::types::{PersonalProposalInfo, ProposalInfo};
pub use proposals_info_runtime_api::CoupleInfoApi as CoupleInfoRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use std::sync::Arc;
#[rpc]
pub trait CoupleInfoApi<
    BlockHash,
    VersionId,
    ProposalId,
    CategoryId,
    Balance,
    Moment,
    CurrencyId,
    AccountId,
> where
    Balance: MaybeDisplay + MaybeFromStr,
    AccountId: Codec + Clone,
{
    #[rpc(name = "proposal_getProposalInfo")]
    fn get_proposal_info(
        &self,
        version_id: VersionId,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> Result<ProposalInfo<CategoryId, Balance, Moment, CurrencyId>>;

    #[rpc(name = "proposal_getPersonalProposalInfo")]
    fn get_personal_proposal_info(
        &self,
        version_id: VersionId,
        proposal_id: ProposalId,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<PersonalProposalInfo<Balance, Moment, CurrencyId>>;
}

pub struct CoupleInfo<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> CoupleInfo<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, VersionId, ProposalId, CategoryId, Balance, Moment, CurrencyId, AccountId>
    CoupleInfoApi<
        <Block as BlockT>::Hash,
        VersionId,
        ProposalId,
        CategoryId,
        Balance,
        Moment,
        CurrencyId,
        AccountId,
    > for CoupleInfo<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: CoupleInfoRuntimeApi<
        Block,
        VersionId,
        ProposalId,
        CategoryId,
        Balance,
        Moment,
        CurrencyId,
        AccountId,
    >,
    VersionId: Codec,
    ProposalId: Codec,
    CategoryId: Codec,
    Balance: Codec + MaybeDisplay + MaybeFromStr,
    Moment: Codec,
    CurrencyId: Codec,
    AccountId: Codec + Clone,
{
    fn get_proposal_info(
        &self,
        version_id: VersionId,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ProposalInfo<CategoryId, Balance, Moment, CurrencyId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let runtime_api_result = api.get_proposal_info(&at, version_id, proposal_id);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_personal_proposal_info(
        &self,
        version_id: VersionId,
        proposal_id: ProposalId,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<PersonalProposalInfo<Balance, Moment, CurrencyId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let runtime_api_result =
            api.get_personal_proposal_info(&at, version_id, proposal_id, account_id);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
