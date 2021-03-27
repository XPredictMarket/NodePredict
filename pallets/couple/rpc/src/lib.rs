use codec::Codec;
pub use couple_info_runtime_api::CoupleInfoApi as CoupleInfoRuntimeApi;
use couple_info_runtime_api::ProposalInfo;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use std::sync::Arc;
#[rpc]
pub trait CoupleInfoApi<BlockHash, ProposalId, CategoryId, Balance, Moment>
where
	Balance: MaybeDisplay + MaybeFromStr,
{
	#[rpc(name = "proposal_getProposalInfo")]
	fn get_proposal_info(
		&self,
		proposal_id: ProposalId,
		at: Option<BlockHash>,
	) -> Result<ProposalInfo<CategoryId, Balance, Moment>>;
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

impl<C, Block, ProposalId, CategoryId, Balance, Moment>
	CoupleInfoApi<<Block as BlockT>::Hash, ProposalId, CategoryId, Balance, Moment>
	for CoupleInfo<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: CoupleInfoRuntimeApi<Block, ProposalId, CategoryId, Balance, Moment>,
	ProposalId: Codec,
	CategoryId: Codec,
	Balance: Codec + MaybeDisplay + MaybeFromStr,
	Moment: Codec,
{
	fn get_proposal_info(
		&self,
		proposal_id: ProposalId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<ProposalInfo<CategoryId, Balance, Moment>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_proposal_info(&at, proposal_id);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
