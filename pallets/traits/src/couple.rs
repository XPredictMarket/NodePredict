use crate::{system::ProposalSystem, tokens::Tokens};
use frame_support::dispatch::DispatchError;

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;

pub trait LiquidityCouple<T>
where
    T: ProposalSystem<T::AccountId> + frame_system::Config,
{
    fn proposal_pair(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<(CurrencyIdOf<T>, CurrencyIdOf<T>), DispatchError>;

    fn set_proposal_result(
        proposal_id: ProposalIdOf<T>,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError>;
    
    fn set_proposal_result_when_end(
        proposal_id: ProposalIdOf<T>,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError>;

    fn get_proposal_result(proposal_id: ProposalIdOf<T>) -> Result<CurrencyIdOf<T>, DispatchError>;

    fn proposal_liquidate_currency_id(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<CurrencyIdOf<T>, DispatchError>;
}
