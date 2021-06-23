#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{dispatch::DispatchError, ensure, traits::Get};
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedSub, Zero};
use sp_std::{mem, vec::Vec};
use xpmrl_couple::Pallet as CouplePallet;
use xpmrl_traits::tokens::Tokens;
use xpmrl_utils::storage_try_mutate;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::*, ModuleId};
    use sp_std::vec::Vec;
    use xpmrl_traits::tokens::Tokens;
    use xpmrl_utils::with_transaction_result;

    pub(crate) type BalanceOf<T> = <<T as xpmrl_couple::Config>::Tokens as Tokens<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type CurrencyIdOf<T> = <<T as xpmrl_couple::Config>::Tokens as Tokens<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
    pub struct Point<BlockNumber, Balance> {
        pub from: BlockNumber,
        pub number: Balance,
    }

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
    pub struct MineInfo<Balance, BlockNumber> {
        pub perblock: Balance,
        pub from: BlockNumber,
        pub to: BlockNumber,
    }

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: xpmrl_couple::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ModuleId: Get<ModuleId>;

        #[pallet::constant]
        type MineTokenCurrencyId: Get<CurrencyIdOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proposal_mine_info)]
    pub type ProposalMineInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::ProposalId,
        MineInfo<BalanceOf<T>, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn proposal_checkpoint)]
    pub type ProposalCheckpoint<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::ProposalId,
        Vec<Point<T::BlockNumber, BalanceOf<T>>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_checkpoint)]
    pub type AccountCheckpoint<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        T::ProposalId,
        Vec<Point<T::BlockNumber, BalanceOf<T>>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_claimed_blocknumber)]
    pub type AccountClaimedBlocknumber<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        T::ProposalId,
        T::BlockNumber,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Stake(T::AccountId, T::ProposalId, T::BlockNumber, BalanceOf<T>),
        UnStake(T::AccountId, T::ProposalId, T::BlockNumber, BalanceOf<T>),
        Claim(T::AccountId, T::ProposalId, T::BlockNumber, BalanceOf<T>),
        UnStakeAndClaim(
            T::AccountId,
            T::ProposalId,
            T::BlockNumber,
            BalanceOf<T>,
            BalanceOf<T>,
        ),
        ProposalMine(T::ProposalId, BalanceOf<T>, T::BlockNumber, T::BlockNumber),
        Deposit(T::AccountId, T::AccountId, BalanceOf<T>),
        Withdrtawal(T::AccountId, T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        BalanceOverflow,
        InsufficientBalance,
        AccountNotStake,
        ProposalNotExist,
        ProposalNotMined,
        ProposalIsMined,
        FromMustMoreThanNow,
        ToMustMoreThanFrom,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn stake(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                ProposalMineInfo::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotMined
            );
            let (now, number) =
                with_transaction_result(|| Self::inner_stake(&who, proposal_id, number))?;
            Self::deposit_event(Event::Stake(who, proposal_id, now, number));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unstake(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
            number: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                ProposalMineInfo::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotMined
            );
            let (now, number) =
                with_transaction_result(|| Self::inner_unstake(&who, proposal_id, number))?;
            Self::deposit_event(Event::UnStake(who, proposal_id, now, number));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn claim(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                ProposalMineInfo::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotMined
            );
            ensure!(
                AccountCheckpoint::<T>::contains_key(&who, proposal_id),
                Error::<T>::AccountNotStake
            );
            let (now, number) = with_transaction_result(|| Self::inner_claim(&who, proposal_id))?;
            Self::deposit_event(Event::Claim(who, proposal_id, now, number));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn claim_and_unstake(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                ProposalMineInfo::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotMined
            );
            let vec_account_checkpoints = AccountCheckpoint::<T>::get(&who, proposal_id)
                .ok_or(Error::<T>::AccountNotStake)?;
            let len = vec_account_checkpoints.len();
            ensure!(len > 0, Error::<T>::AccountNotStake);
            let (now, number_stake, number_claim) = with_transaction_result(|| {
                let number = vec_account_checkpoints[len - 1].clone().number;
                let (_, number_stake) = Self::inner_unstake(&who, proposal_id, number)?;
                let (now, number_claim) = Self::inner_claim(&who, proposal_id)?;
                Ok((now, number_stake, number_claim))
            })?;
            Self::deposit_event(Event::UnStakeAndClaim(
                who,
                proposal_id,
                now,
                number_stake,
                number_claim,
            ));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn deposit(origin: OriginFor<T>, number: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let currency_id = T::MineTokenCurrencyId::get();
            let module_account: T::AccountId = T::ModuleId::get().into_account();
            T::Tokens::transfer(currency_id, &who, &module_account, number)?;
            Self::deposit_event(Event::Deposit(who, module_account, number));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn withdrawal(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let currency_id = T::MineTokenCurrencyId::get();
            let module_account: T::AccountId = T::ModuleId::get().into_account();
            let balance = T::Tokens::balance(currency_id, &module_account);
            T::Tokens::transfer(currency_id, &module_account, &to, balance)?;
            Self::deposit_event(Event::Withdrtawal(module_account, to, balance));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn proposal_mine(
            origin: OriginFor<T>,
            proposal_id: T::ProposalId,
            perblock: BalanceOf<T>,
            from: T::BlockNumber,
            to: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let now = Self::block_number();
            ensure!(from > now, Error::<T>::FromMustMoreThanNow);
            ensure!(to > from, Error::<T>::ToMustMoreThanFrom);
            let _ = Self::currency_id(proposal_id)?;
            ProposalMineInfo::<T>::try_mutate(
                proposal_id,
                |option_mine_info| -> Result<(), DispatchError> {
                    let new_mine_info = MineInfo { perblock, from, to };
                    let mine_info = option_mine_info.clone().unwrap_or(MineInfo {
                        perblock: Zero::zero(),
                        from: Zero::zero(),
                        to: Zero::zero(),
                    });
                    if mine_info.from > now || mine_info.from == Zero::zero() {
                        *option_mine_info = Some(new_mine_info);
                        Ok(())
                    } else {
                        if new_mine_info.perblock == Zero::zero() {
                            *option_mine_info = None;
                            Ok(())
                        } else {
                            Err(Error::<T>::ProposalIsMined)?
                        }
                    }
                },
            )?;
            Self::deposit_event(Event::ProposalMine(proposal_id, perblock, from, to));
            Ok(().into())
        }
    }
}

macro_rules! account_checkpoint_try_mutate {
    ($who: ident, $proposal_id: ident, $now: ident, $vec_point: ident, $new_expr: expr) => {
        storage_try_mutate!(
            AccountCheckpoint,
            T,
            &$who,
            $proposal_id,
            |option_vec_point| -> Result<(), DispatchError> {
                let mut $vec_point = option_vec_point.clone().unwrap_or_default();
                let number = $new_expr;
                $vec_point.push(Point { from: $now, number });
                *option_vec_point = Some($vec_point);
                Ok(())
            },
        )
    };
}

macro_rules! total_checkpoint_try_mutate {
    ($proposal_id: ident, $now: ident, $vec_point: ident, $new_expr: expr) => {
        storage_try_mutate!(
            ProposalCheckpoint,
            T,
            $proposal_id,
            |option_vec_point| -> Result<(), DispatchError> {
                let mut $vec_point = option_vec_point.clone().unwrap_or_default();
                $vec_point.push(Point {
                    from: $now,
                    number: $new_expr,
                });
                *option_vec_point = Some($vec_point);
                Ok(())
            },
        )
    };
}

impl<T: Config> Pallet<T> {
    fn block_number() -> T::BlockNumber {
        frame_system::Module::<T>::block_number()
    }

    fn currency_id(proposal_id: T::ProposalId) -> Result<CurrencyIdOf<T>, Error<T>> {
        CouplePallet::<T>::proposal_liquidate_currency_id(proposal_id)
            .ok_or(Error::<T>::ProposalNotExist)
    }

    fn inner_stake(
        who: &T::AccountId,
        proposal_id: T::ProposalId,
        number: BalanceOf<T>,
    ) -> Result<(T::BlockNumber, BalanceOf<T>), DispatchError> {
        let now = Self::block_number();
        let currency_id = Self::currency_id(proposal_id)?;
        account_checkpoint_try_mutate!(
            who,
            proposal_id,
            now,
            vec_point,
            match vec_point.last() {
                Some(last) => last
                    .number
                    .checked_add(&number)
                    .ok_or(Error::<T>::BalanceOverflow)?,
                None => number,
            }
        )?;
        total_checkpoint_try_mutate!(
            proposal_id,
            now,
            vec_point,
            match vec_point.last() {
                Some(last) => last
                    .number
                    .checked_add(&number)
                    .ok_or(Error::<T>::BalanceOverflow)?,
                None => number,
            }
        )?;
        T::Tokens::reserve(currency_id, &who, number)?;
        Ok((now, number))
    }

    fn inner_unstake(
        who: &T::AccountId,
        proposal_id: T::ProposalId,
        number: BalanceOf<T>,
    ) -> Result<(T::BlockNumber, BalanceOf<T>), DispatchError> {
        let now = Self::block_number();
        let currency_id = Self::currency_id(proposal_id)?;
        ensure!(
            AccountCheckpoint::<T>::contains_key(&who, proposal_id),
            Error::<T>::AccountNotStake
        );
        let mut finally_number: BalanceOf<T> = Zero::zero();
        account_checkpoint_try_mutate!(
            who,
            proposal_id,
            now,
            vec_point,
            match vec_point.last() {
                Some(last) => {
                    let diff = last.number.checked_sub(&number).unwrap_or_else(Zero::zero);
                    finally_number = last.number.checked_sub(&diff).unwrap_or_else(Zero::zero);
                    diff
                }
                None => Zero::zero(),
            }
        )?;
        let number = finally_number;
        total_checkpoint_try_mutate!(
            proposal_id,
            now,
            vec_point,
            match vec_point.last() {
                Some(last) => last.number.checked_sub(&number).unwrap_or_else(Zero::zero),
                None => Zero::zero(),
            }
        )?;
        T::Tokens::unreserve(currency_id, &who, number)?;
        Ok((now, number))
    }

    fn get_range(
        checkpoints: &Vec<Point<T::BlockNumber, BalanceOf<T>>>,
        i: usize,
        now: T::BlockNumber,
        start: T::BlockNumber,
        end: T::BlockNumber,
    ) -> [Point<T::BlockNumber, BalanceOf<T>>; 2] {
        let len = checkpoints.len();
        let mut from = checkpoints[i].clone();
        let mut to;
        if i + 1 == len {
            to = Point {
                from: now,
                number: from.number,
            };
        } else {
            to = checkpoints[i + 1].clone();
        }
        if to.from >= start && from.from < start {
            from.from = start;
        }
        if to.from >= end && from.from < end {
            to.from = end;
        }
        if from.from >= end {
            from.from = end;
            to.from = end;
        }
        if to.from <= start {
            from.from = start;
            to.from = start;
        }
        [from, to]
    }

    fn get_sum(
        account_checkpoints: &Vec<Point<T::BlockNumber, BalanceOf<T>>>,
        total_checkpoints: &Vec<Point<T::BlockNumber, BalanceOf<T>>>,
        perblock: BalanceOf<T>,
        now: T::BlockNumber,
        start: T::BlockNumber,
        end: T::BlockNumber,
    ) -> BalanceOf<T> {
        let account_checkpoint_len = account_checkpoints.len();
        let total_checkpoint_len = total_checkpoints.len();

        let mut sum: BalanceOf<T> = Zero::zero();

        for i in 0..account_checkpoint_len {
            let account_range = Self::get_range(&account_checkpoints, i, now, start, end);
            if account_range[1].from <= start {
                continue;
            }
            if account_range[0].from > end {
                break;
            }

            let owner = account_range[0].number;
            let _100: BalanceOf<T> = 100u32.into();

            for j in 0..total_checkpoint_len {
                let total_range = Self::get_range(&total_checkpoints, j, now, start, end);
                let total = total_range[0].number;
                if total_range[1].from <= account_range[0].from {
                    continue;
                }
                if total_range[0].from >= account_range[1].from {
                    break;
                }
                let scale = (owner * _100)
                    .checked_div(&total)
                    .unwrap_or_else(Zero::zero);
                let mut diff: T::BlockNumber = Zero::zero();
                if total_range[0].from == account_range[0].from
                    && total_range[1].from == account_range[1].from
                {
                    diff = account_range[1].from - account_range[0].from;
                } else if total_range[0].from < account_range[0].from
                    && total_range[1].from < account_range[1].from
                    && total_range[1].from > account_range[0].from
                {
                    diff = account_range[1].from - total_range[0].from;
                } else if total_range[1].from <= account_range[1].from {
                    diff = total_range[1].from - total_range[0].from;
                } else if total_range[1].from > account_range[1].from {
                    diff = total_range[1].from - account_range[0].from;
                }
                unsafe {
                    let diff = mem::transmute::<&T::BlockNumber, &BalanceOf<T>>(&diff);
                    sum += (*diff) * scale * perblock / _100;
                }
            }
        }
        sum
    }

    fn inner_claim(
        who: &T::AccountId,
        proposal_id: T::ProposalId,
    ) -> Result<(T::BlockNumber, BalanceOf<T>), DispatchError> {
        let currency_id = T::MineTokenCurrencyId::get();
        let module_account: T::AccountId = T::ModuleId::get().into_account();
        let now = Self::block_number();

        let mine_info =
            ProposalMineInfo::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIsMined)?;
        let total_checkpoints =
            ProposalCheckpoint::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotMined)?;
        let account_checkpoints =
            AccountCheckpoint::<T>::get(&who, proposal_id).ok_or(Error::<T>::AccountNotStake)?;

        let [start, end]: [T::BlockNumber; 2] = [
            match AccountClaimedBlocknumber::<T>::get(&who, proposal_id) {
                Some(val) => {
                    if val < mine_info.from {
                        mine_info.from
                    } else {
                        val
                    }
                }
                None => mine_info.from,
            },
            mine_info.to,
        ];
        let mut sum = Self::get_sum(
            &account_checkpoints,
            &total_checkpoints,
            mine_info.perblock,
            now,
            start,
            end,
        );
        ensure!(
            T::Tokens::balance(currency_id, &module_account) >= sum,
            Error::<T>::InsufficientBalance,
        );
        AccountClaimedBlocknumber::<T>::try_mutate(
            &who,
            proposal_id,
            |option_block| -> Result<T::BlockNumber, DispatchError> {
                let old = option_block.unwrap_or(mine_info.from);
                *option_block = Some(now);
                Ok(old)
            },
        )?;
        sum = T::Tokens::transfer(currency_id, &module_account, &who, sum)?;
        Ok((now, sum))
    }
}
