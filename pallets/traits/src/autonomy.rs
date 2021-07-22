use crate::{system::ProposalSystem, tokens::Tokens};
use sp_runtime::DispatchError;

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
type BalanceOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;

pub trait Autonomy<T>
where
    T: ProposalSystem<T::AccountId> + frame_system::Config,
{
    fn temporary_results(
        proposal_id: ProposalIdOf<T>,
        who: &T::AccountId,
    ) -> Result<CurrencyIdOf<T>, DispatchError>;

    fn statistical_results(
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
    ) -> BalanceOf<T>;
}
