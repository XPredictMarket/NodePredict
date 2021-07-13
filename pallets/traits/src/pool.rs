use crate::{tokens::Tokens, ProposalStatus};
use frame_support::{dispatch::DispatchError, traits::Time};

use crate::system::ProposalSystem;

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;

type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
type MomentOf<T> = <TimeOf<T> as Time>::Moment;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
type VersionIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::VersionId;

pub trait LiquidityPool<T>
where
    T: ProposalSystem<T::AccountId> + frame_system::Config,
{
    fn get_proposal_minimum_interval_time() -> MomentOf<T>;
    fn is_currency_id_used(currency_id: CurrencyIdOf<T>) -> bool;
    fn get_next_proposal_id() -> Result<ProposalIdOf<T>, DispatchError>;
    fn init_proposal(
        proposal_id: ProposalIdOf<T>,
        owner: &T::AccountId,
        state: ProposalStatus,
        version: VersionIdOf<T>,
    );
    fn append_used_currency(currency_id: CurrencyIdOf<T>);

    fn max_proposal_id() -> ProposalIdOf<T>;
    fn proposal_automatic_expiration_time() -> MomentOf<T>;
    fn get_proposal_state(proposal_id: ProposalIdOf<T>) -> Result<ProposalStatus, DispatchError>;
    fn set_proposal_state(
        proposal_id: ProposalIdOf<T>,
        new_state: ProposalStatus,
    ) -> Result<ProposalStatus, DispatchError>;

    fn get_earn_trading_fee_decimals() -> u8;
    fn proposal_liquidity_provider_fee_rate() -> u32;

    fn proposal_owner(proposal_id: ProposalIdOf<T>) -> Result<T::AccountId, DispatchError>;
}

pub trait LiquiditySubPool<T>
where
    T: ProposalSystem<T::AccountId> + frame_system::Config,
{
    fn finally_locked(proposal_id: ProposalIdOf<T>) -> Result<(), DispatchError>;
}
