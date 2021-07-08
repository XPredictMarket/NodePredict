use crate::tokens::Tokens;
use frame_support::{
    dispatch::{DispatchError, DispatchResultWithPostInfo},
    traits::Time,
};
use frame_system::pallet_prelude::OriginFor;
use sp_std::vec::Vec;

use crate::system::ProposalSystem;

type TokensOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Tokens;
type CurrencyIdOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
type BalanceOf<T> = <TokensOf<T> as Tokens<<T as frame_system::Config>::AccountId>>::Balance;

type TimeOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::Time;
type MomentOf<T> = <TimeOf<T> as Time>::Moment;

type ProposalIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::ProposalId;
type CategoryIdOf<T> = <T as ProposalSystem<<T as frame_system::Config>::AccountId>>::CategoryId;

pub trait LiquidityCouple<T>
where
    T: ProposalSystem<T::AccountId> + frame_system::Config,
{
    fn proposal_announcement_time(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<MomentOf<T>, DispatchError>;

    fn proposal_pair(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<(CurrencyIdOf<T>, CurrencyIdOf<T>), DispatchError>;

    fn set_proposal_result(
        proposal_id: ProposalIdOf<T>,
        result: CurrencyIdOf<T>,
    ) -> Result<(), DispatchError>;

    fn proposal_liquidate_currency_id(
        proposal_id: ProposalIdOf<T>,
    ) -> Result<CurrencyIdOf<T>, DispatchError>;

    fn new_couple_proposal(
        origin: OriginFor<T>,
        title: Vec<u8>,
        optional: [Vec<u8>; 2],
        close_time: MomentOf<T>,
        category_id: CategoryIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        earn_fee: u32,
        detail: Vec<u8>,
    ) -> DispatchResultWithPostInfo;
}
