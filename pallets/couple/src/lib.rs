#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	ensure,
	storage::{with_transaction, TransactionOutcome},
	traits::{Get, Time},
};
use num_traits::pow::pow;
use pallet::Pallet;
use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot, One, Zero},
	DispatchError,
};
use sp_std::vec::Vec;
use xpmrl_proposals::Pallet as ProposalPallet;
use xpmrl_traits::pool::LiquidityPool;
use xpmrl_traits::{tokens::Tokens, ProposalStatus};

macro_rules! runtime_format {
	($($args:tt)*) => {{
		#[cfg(feature = "std")]
		{
			format!($($args)*).as_bytes().to_vec()
		}
		#[cfg(not(feature = "std"))]
		{
			sp_std::alloc::format!($($args)*).as_bytes().to_vec()
		}
	}};
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero};
	use sp_std::{cmp, vec::Vec};
	use xpmrl_proposals::Pallet as ProposalPallet;
	use xpmrl_traits::{tokens::Tokens, ProposalStatus};

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Tokens as Tokens<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::Tokens as Tokens<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type CategoryIdOf<T> = <T as xpmrl_proposals::Config>::CategoryId;
	pub(crate) type ProposalIdOf<T> = <T as xpmrl_proposals::Config>::ProposalId;
	pub(crate) type MomentOf<T> = <<T as xpmrl_proposals::Config>::Time as Time>::Moment;

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
	pub struct Proposal<CategoryId> {
		pub title: Vec<u8>,
		pub category_id: CategoryId,
		pub detail: Vec<u8>,
	}

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: xpmrl_proposals::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Tokens: Tokens<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, Proposal<T::CategoryId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_close_time)]
	pub type ProposalCloseTime<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_create_time)]
	pub type ProposalCreateTime<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_pairs)]
	pub type PoolPairs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalIdOf<T>,
		(CurrencyIdOf<T>, CurrencyIdOf<T>),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_currency_id)]
	pub type ProposalCurrencyId<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_liquidate_currency_id)]
	pub type ProposalLiquidateCurrencyId<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_total_earn_trading_fee)]
	pub type ProposalTotalEarnTradingFee<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, u32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_result)]
	pub type ProposalResult<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalIdOf<T>, CurrencyIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_account_info)]
	pub type ProposalAccountInfo<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::ProposalId,
		Twox64Concat,
		T::AccountId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_total_market)]
	pub type ProposalTotalMarket<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_total_optional_market)]
	pub type ProposalTotalOptionalMarket<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, (BalanceOf<T>, BalanceOf<T>), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_total_market_fee)]
	pub type ProposalTotalMarketFee<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_total_market_liquid)]
	pub type ProposalTotalMarketLiquid<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, BalanceOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AddLiquidity(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		RemoveLiquidity(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		Buy(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		Sell(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		Retrieval(T::AccountId, ProposalIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		SetResult(ProposalIdOf<T>, CurrencyIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		CurrencyIdNotFound,
		ProposalMustFormalPrediction,
		ProposalAbnormalState,
		ProposalIdNotExist,
		ProposalNotResult,
		ProposalOptionNotCorrect,
		BalanceOverflow,
		NoRealNumber,
		InsufficientBalance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
			number: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status == ProposalStatus::FormalPrediction,
				Error::<T>::ProposalAbnormalState
			);
			let currency_id =
				ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			let (asset_id_1, asset_id_2) =
				PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			Self::with_transaction_result(|| {
				T::Tokens::donate(currency_id, &who, number)?;
				T::Tokens::mint_donate(asset_id_1, number)?;
				T::Tokens::mint_donate(asset_id_2, number)?;
				ProposalTotalOptionalMarket::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(), DispatchError> {
						let (o1, o2) = item.ok_or(Error::<T>::ProposalIdNotExist)?;
						let new_o1 = o1.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?;
						let new_o2 = o2.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some((new_o1, new_o2));
						Ok(())
					},
				)?;
				ProposalTotalMarketLiquid::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(), DispatchError> {
						let old_value = item.ok_or(Error::<T>::ProposalIdNotExist)?;
						let new_value = old_value
							.checked_add(&number)
							.ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some(new_value);
						Ok(())
					},
				)?;
				Self::total_and_account_add(proposal_id, &who, number)?;
				Ok(())
			})?;
			Self::deposit_event(Event::AddLiquidity(who, proposal_id, currency_id, number));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
			number: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status == ProposalStatus::End,
				Error::<T>::ProposalAbnormalState
			);
			let currency_id =
				ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			let liquidate_currency_id = ProposalLiquidateCurrencyId::<T>::get(proposal_id)
				.ok_or(Error::<T>::ProposalIdNotExist)?;
			let (asset_id_1, asset_id_2) =
				PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			Self::with_transaction_result(|| {
				T::Tokens::burn(liquidate_currency_id, &who, number)?;
				let total_liquid = ProposalTotalMarketLiquid::<T>::try_mutate(
					proposal_id,
					|item| -> Result<BalanceOf<T>, DispatchError> {
						let old_value = item.ok_or(Error::<T>::ProposalIdNotExist)?;
						let new_value = old_value
							.checked_sub(&number)
							.ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some(new_value);
						Ok(old_value)
					},
				)?;
				let fee = Self::get_fee_of_liquid(proposal_id, number, total_liquid)?;
				let creater_fee = Self::get_fee_of_creator(&who, proposal_id)?;
				let fee = fee
					.checked_add(&creater_fee)
					.ok_or(Error::<T>::BalanceOverflow)?;
				ProposalTotalMarketFee::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(), DispatchError> {
						let old_value = item.unwrap_or(Zero::zero());
						let new_value = old_value
							.checked_sub(&fee)
							.ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some(new_value);
						Ok(())
					},
				)?;
				let (o1, o2) = ProposalTotalOptionalMarket::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
						let (o1, o2) = item.ok_or(Error::<T>::ProposalIdNotExist)?;

						let new_o1 = o1.checked_mul(&number).ok_or(Error::<T>::BalanceOverflow)?;
						let new_o1 = new_o1
							.checked_div(&total_liquid.into())
							.ok_or(Error::<T>::BalanceOverflow)?;
						let new_o1 = o1.checked_sub(&new_o1).ok_or(Error::<T>::BalanceOverflow)?;

						let new_o2 = o2.checked_mul(&number).ok_or(Error::<T>::BalanceOverflow)?;
						let new_o2 = new_o2
							.checked_div(&total_liquid.into())
							.ok_or(Error::<T>::BalanceOverflow)?;
						let new_o2 = o2.checked_sub(&new_o2).ok_or(Error::<T>::BalanceOverflow)?;

						*item = Some((new_o1, new_o2));
						Ok((
							o1.checked_sub(&new_o1).unwrap_or(Zero::zero()),
							o2.checked_sub(&new_o2).unwrap_or(Zero::zero()),
						))
					},
				)?;
				let min = cmp::min(o1, o2);
				T::Tokens::burn_donate(asset_id_1, min)?;
				T::Tokens::burn_donate(asset_id_2, min)?;
				Self::total_and_account_sub(proposal_id, &who, min)?;
				let actual_amount = min.checked_add(&fee).ok_or(Error::<T>::BalanceOverflow)?;
				T::Tokens::appropriation(currency_id, &who, actual_amount)?;
				T::Tokens::appropriation(
					asset_id_1,
					&who,
					o1.checked_sub(&min).unwrap_or(Zero::zero()),
				)?;
				T::Tokens::appropriation(
					asset_id_2,
					&who,
					o2.checked_sub(&min).unwrap_or(Zero::zero()),
				)?;
				Ok(())
			})?;
			Self::deposit_event(Event::RemoveLiquidity(
				who,
				proposal_id,
				currency_id,
				number,
			));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn buy(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
			optional_currency_id: CurrencyIdOf<T>,
			number: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status == ProposalStatus::FormalPrediction,
				Error::<T>::ProposalAbnormalState
			);
			let currency_id =
				ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			let (asset_id_1, asset_id_2) =
				PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			ensure!(
				optional_currency_id == asset_id_1 || optional_currency_id == asset_id_2,
				Error::<T>::CurrencyIdNotFound
			);
			let other_currency = Self::get_other_optional_id(proposal_id, optional_currency_id)?;
			let actual_number = Self::with_transaction_result(|| {
				let (actual_number, fee) = Self::get_fee(proposal_id, number)?;
				T::Tokens::donate(currency_id, &who, number)?;
				T::Tokens::mint(optional_currency_id, &who, actual_number)?;
				T::Tokens::mint_donate(other_currency.1, actual_number)?;
				let diff = ProposalTotalOptionalMarket::<T>::try_mutate(
					proposal_id,
					|item| -> Result<BalanceOf<T>, DispatchError> {
						let old_pair = item.ok_or(Error::<T>::ProposalIdNotExist)?;
						let old_pair = [old_pair.0, old_pair.1];
						let new_pair =
							Self::add_and_adjust_pool(other_currency.0, actual_number, &old_pair)?;
						let diff = old_pair[1 - other_currency.0]
							.checked_sub(&new_pair[1 - other_currency.0])
							.ok_or(Error::<T>::BalanceOverflow)?;

						*item = Some((new_pair[0], new_pair[1]));
						Ok(diff)
					},
				)?;
				Self::total_and_account_add(proposal_id, &who, actual_number)?;
				ProposalTotalMarketFee::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(), DispatchError> {
						let old_value = item.unwrap_or(Zero::zero());
						let new_value = old_value
							.checked_add(&fee)
							.ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some(new_value);
						Ok(())
					},
				)?;
				T::Tokens::appropriation(optional_currency_id, &who, diff)?;
				Ok(actual_number)
			})?;
			Self::deposit_event(Event::Buy(
				who,
				proposal_id,
				optional_currency_id,
				actual_number,
			));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn sell(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
			optional_currency_id: CurrencyIdOf<T>,
			number: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status == ProposalStatus::FormalPrediction,
				Error::<T>::ProposalAbnormalState
			);
			let currency_id =
				ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			let (asset_id_1, asset_id_2) =
				PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			ensure!(
				optional_currency_id == asset_id_1 || optional_currency_id == asset_id_2,
				Error::<T>::CurrencyIdNotFound
			);
			let other_currency = Self::get_other_optional_id(proposal_id, optional_currency_id)?;
			let actual_number = Self::with_transaction_result(|| {
				T::Tokens::donate(optional_currency_id, &who, number)?;
				let diff = ProposalTotalOptionalMarket::<T>::try_mutate(
					proposal_id,
					|item| -> Result<[BalanceOf<T>; 2], DispatchError> {
						let old_pair = item.ok_or(Error::<T>::ProposalIdNotExist)?;
						let old_pair = [old_pair.0, old_pair.1];
						let actual_number = Self::get_sell_result(
							proposal_id,
							&old_pair,
							number,
							optional_currency_id,
						)?;
						let new_pair = Self::add_and_adjust_pool(
							1 - other_currency.0,
							actual_number,
							&old_pair,
						)?;
						*item = Some((new_pair[0], new_pair[1]));
						let mut diff = new_pair.clone();
						diff[other_currency.0] = old_pair[other_currency.0]
							.checked_sub(&diff[other_currency.0])
							.ok_or(Error::<T>::BalanceOverflow)?;
						diff[1 - other_currency.0] = diff[1 - other_currency.0]
							.checked_sub(&old_pair[1 - other_currency.0])
							.ok_or(Error::<T>::BalanceOverflow)?;
						Ok(diff)
					},
				)?;
				let last_select_currency = number
					.checked_sub(&diff[1 - other_currency.0])
					.ok_or(Error::<T>::BalanceOverflow)?;
				let acquired_currency = diff[other_currency.0];
				let min = cmp::min(last_select_currency, acquired_currency);
				let (actual_number, fee) = Self::get_fee(proposal_id, min)?;
				ProposalTotalMarketFee::<T>::try_mutate(
					proposal_id,
					|item| -> Result<(), DispatchError> {
						let old_value = item.unwrap_or(Zero::zero());
						let new_value = old_value
							.checked_add(&fee)
							.ok_or(Error::<T>::BalanceOverflow)?;
						*item = Some(new_value);
						Ok(())
					},
				)?;
				Self::total_and_account_sub(proposal_id, &who, min)?;
				T::Tokens::appropriation(currency_id, &who, actual_number)?;
				T::Tokens::appropriation(
					optional_currency_id,
					&who,
					last_select_currency
						.checked_sub(&min)
						.ok_or(Error::<T>::BalanceOverflow)?,
				)?;
				T::Tokens::appropriation(
					other_currency.1,
					&who,
					acquired_currency
						.checked_sub(&min)
						.ok_or(Error::<T>::BalanceOverflow)?,
				)?;
				Ok(actual_number)
			})?;
			Self::deposit_event(Event::Sell(
				who,
				proposal_id,
				optional_currency_id,
				actual_number,
			));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn retrieval(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status == ProposalStatus::End,
				Error::<T>::ProposalAbnormalState
			);
			let result_id =
				ProposalResult::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotResult)?;
			let currency_id =
				ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			let balance = T::Tokens::balance(result_id, &who);
			ensure!(balance >= Zero::zero(), Error::<T>::InsufficientBalance);
			Self::with_transaction_result(|| {
				T::Tokens::burn(result_id, &who, balance)?;
				T::Tokens::appropriation(currency_id, &who, balance)?;
				Ok(())
			})?;
			Self::deposit_event(Event::Retrieval(who, proposal_id, result_id, balance));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_result(
			origin: OriginFor<T>,
			proposal_id: ProposalIdOf<T>,
			currency_id: CurrencyIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;
			let status = Self::get_proposal_status(proposal_id)?;
			ensure!(
				status != ProposalStatus::OriginalPrediction || status != ProposalStatus::End,
				Error::<T>::ProposalAbnormalState
			);
			let (asset_id_1, asset_id_2) =
				PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
			ensure!(
				currency_id == asset_id_1 || currency_id == asset_id_2,
				Error::<T>::CurrencyIdNotFound
			);
			Self::with_transaction_result(|| {
				ProposalPallet::<T>::set_new_status(proposal_id, ProposalStatus::End)?;
				ProposalResult::<T>::insert(proposal_id, currency_id);
				Ok(())
			})?;
			Self::deposit_event(Event::SetResult(proposal_id, currency_id));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn with_transaction_result<R>(
		f: impl FnOnce() -> Result<R, DispatchError>,
	) -> Result<R, DispatchError> {
		with_transaction(|| {
			let res = f();
			if res.is_ok() {
				TransactionOutcome::Commit(res)
			} else {
				TransactionOutcome::Rollback(res)
			}
		})
	}

	pub fn get_other_optional_id(
		proposal_id: T::ProposalId,
		optional_currency_id: CurrencyIdOf<T>,
	) -> Result<(usize, CurrencyIdOf<T>), DispatchError> {
		let (asset_id_1, asset_id_2) =
			PoolPairs::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
		let other_currency_id = if optional_currency_id == asset_id_1 {
			(1, asset_id_2)
		} else {
			(0, asset_id_1)
		};
		Ok(other_currency_id)
	}

	pub fn get_fee_of_liquid(
		proposal_id: ProposalIdOf<T>,
		number: BalanceOf<T>,
		total_liquid: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let market_fee = ProposalTotalMarketFee::<T>::get(proposal_id).unwrap_or(Zero::zero());
		let mul_market_fee = market_fee
			.checked_mul(&number)
			.ok_or(Error::<T>::BalanceOverflow)?;
		let mul_market_fee = mul_market_fee
			.checked_mul(&90u32.into())
			.ok_or(Error::<T>::BalanceOverflow)?;
		let fee = mul_market_fee
			.checked_div(&total_liquid)
			.ok_or(Error::<T>::BalanceOverflow)?;
		let fee = fee
			.checked_div(&100u32.into())
			.ok_or(Error::<T>::BalanceOverflow)?;
		Ok(fee)
	}

	pub fn get_fee_of_creator(
		who: &T::AccountId,
		proposal_id: ProposalIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let owner = ProposalPallet::<T>::proposal_owner(proposal_id)
			.ok_or(Error::<T>::ProposalIdNotExist)?;
		if owner == *who {
			let market_fee = ProposalTotalMarketFee::<T>::get(proposal_id).unwrap_or(Zero::zero());
			let mul_market_fee = market_fee
				.checked_mul(&90u32.into())
				.ok_or(Error::<T>::BalanceOverflow)?;
			let fee = mul_market_fee
				.checked_div(&100u32.into())
				.ok_or(Error::<T>::BalanceOverflow)?;
			let fee = market_fee
				.checked_sub(&fee)
				.ok_or(Error::<T>::BalanceOverflow)?;
			Ok(fee)
		} else {
			Ok(Zero::zero())
		}
	}

	pub fn init_pool(
		who: &T::AccountId,
		proposal_id: T::ProposalId,
		title: Vec<u8>,
		close_time: MomentOf<T>,
		category_id: T::CategoryId,
		currency_id: CurrencyIdOf<T>,
		optional: [Vec<u8>; 2],
		number: BalanceOf<T>,
		earn_fee: u32,
		detail: Vec<u8>,
	) -> Result<(), DispatchError> {
		Self::with_transaction_result(|| -> Result<(), DispatchError> {
			Proposals::<T>::insert(
				proposal_id,
				Proposal {
					title,
					category_id,
					detail,
				},
			);
			ProposalCloseTime::<T>::insert(proposal_id, close_time);
			ProposalCreateTime::<T>::insert(proposal_id, T::Time::now());
			ProposalCurrencyId::<T>::insert(proposal_id, currency_id);
			T::Tokens::donate(currency_id, &who, number)?;
			let decimals = T::Tokens::decimals(currency_id)?;
			let asset_id_1 = T::Tokens::new_asset(
				optional[0].clone(),
				runtime_format!("{:?}-yes", proposal_id),
				decimals,
			)?;
			let asset_id_2 = T::Tokens::new_asset(
				optional[1].clone(),
				runtime_format!("{:?}-no", proposal_id),
				decimals,
			)?;
			let asset_id_lp = T::Tokens::new_asset(
				runtime_format!("{:?}-lp", proposal_id),
				runtime_format!("{:?}-lp", proposal_id),
				decimals,
			)?;

			T::Tokens::mint_donate(asset_id_1, number)?;
			T::Tokens::mint_donate(asset_id_2, number)?;
			ProposalTotalOptionalMarket::<T>::insert(proposal_id, (number, number));

			ProposalLiquidateCurrencyId::<T>::insert(proposal_id, asset_id_lp);
			T::Tokens::mint(asset_id_lp, &who, number)?;

			PoolPairs::<T>::insert(proposal_id, (asset_id_1, asset_id_2));
			ProposalTotalEarnTradingFee::<T>::insert(proposal_id, earn_fee);
			ProposalAccountInfo::<T>::insert(proposal_id, who.clone(), number);
			ProposalTotalMarket::<T>::insert(proposal_id, number);
			ProposalTotalMarketLiquid::<T>::insert(proposal_id, number);
			Ok(())
		})
	}

	fn get_proposal_status(proposal_id: ProposalIdOf<T>) -> Result<ProposalStatus, DispatchError> {
		Ok(ProposalPallet::<T>::proposal_status(proposal_id)
			.ok_or(Error::<T>::ProposalIdNotExist)?)
	}

	fn get_fee(
		proposal_id: ProposalIdOf<T>,
		number: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let fee_decimals = <T as xpmrl_proposals::Config>::EarnTradingFeeDecimals::get();
		let one = pow(10u32, fee_decimals.into());
		let fee_rate = ProposalTotalEarnTradingFee::<T>::get(proposal_id)
			.ok_or(Error::<T>::ProposalIdNotExist)?;
		let mut rate = number
			.checked_mul(&(fee_rate.into()))
			.ok_or(Error::<T>::BalanceOverflow)?;
		rate = rate
			.checked_div(&(one.into()))
			.ok_or(Error::<T>::BalanceOverflow)?;
		let actual_number = number
			.checked_sub(&rate)
			.ok_or(Error::<T>::BalanceOverflow)?;
		Ok((actual_number, rate))
	}

	fn add_and_adjust_pool(
		to_add: usize,
		number: BalanceOf<T>,
		old_pair: &[BalanceOf<T>; 2],
	) -> Result<[BalanceOf<T>; 2], DispatchError> {
		let base = old_pair[0]
			.checked_mul(&old_pair[1])
			.ok_or(Error::<T>::BalanceOverflow)?;
		let mut new_pair = old_pair.clone();
		new_pair[to_add] = new_pair[to_add]
			.checked_add(&number)
			.ok_or(Error::<T>::BalanceOverflow)?;
		new_pair[1 - to_add] = base
			.checked_div(&new_pair[to_add])
			.ok_or(Error::<T>::BalanceOverflow)?;
		Ok(new_pair)
	}

	fn get_sell_result(
		proposal_id: ProposalIdOf<T>,
		pair: &[BalanceOf<T>; 2],
		number: BalanceOf<T>,
		current_currency: CurrencyIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let a: BalanceOf<T> = One::one();
		let b: BalanceOf<T> = pair[0]
			.checked_add(&pair[1])
			.ok_or(Error::<T>::BalanceOverflow)?;
		let b: BalanceOf<T> = b.checked_sub(&number).ok_or(Error::<T>::BalanceOverflow)?;
		let other_currency = Self::get_other_optional_id(proposal_id, current_currency)?;
		let c: BalanceOf<T> = number
			.checked_mul(&pair[1 - other_currency.0])
			.ok_or(Error::<T>::BalanceOverflow)?;
		let _4ac = a.checked_mul(&c).ok_or(Error::<T>::BalanceOverflow)?;
		let _4ac = _4ac
			.checked_mul(&4u32.into())
			.ok_or(Error::<T>::BalanceOverflow)?;
		let _2a = a
			.checked_mul(&2u32.into())
			.ok_or(Error::<T>::BalanceOverflow)?;
		let delta = pow(b, 2)
			.checked_add(&_4ac)
			.ok_or(Error::<T>::BalanceOverflow)?;
		let sqrt_delta = delta.integer_sqrt();
		ensure!(sqrt_delta >= b, Error::<T>::NoRealNumber);
		let tmp = sqrt_delta
			.checked_sub(&b)
			.ok_or(Error::<T>::BalanceOverflow)?;
		Ok(tmp
			.checked_div(&2u32.into())
			.ok_or(Error::<T>::BalanceOverflow)?)
	}

	fn total_and_account_add(
		proposal_id: ProposalIdOf<T>,
		who: &T::AccountId,
		diff: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		ProposalTotalMarket::<T>::try_mutate(
			proposal_id,
			|value| -> Result<BalanceOf<T>, DispatchError> {
				let old_amount = value.ok_or(Error::<T>::ProposalIdNotExist)?;
				let new_amount = old_amount
					.checked_add(&diff)
					.ok_or(Error::<T>::BalanceOverflow)?;
				*value = Some(new_amount);
				Ok(new_amount.checked_sub(&old_amount).unwrap_or(Zero::zero()))
			},
		)?;
		ProposalAccountInfo::<T>::try_mutate(
			proposal_id,
			&who,
			|value| -> Result<BalanceOf<T>, DispatchError> {
				let old_amount = value.unwrap_or(Zero::zero());
				let new_amount = old_amount
					.checked_add(&diff)
					.ok_or(Error::<T>::BalanceOverflow)?;
				*value = Some(new_amount);
				Ok(new_amount.checked_sub(&old_amount).unwrap_or(Zero::zero()))
			},
		)?;
		Ok(())
	}

	fn total_and_account_sub(
		proposal_id: ProposalIdOf<T>,
		who: &T::AccountId,
		diff: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		ProposalTotalMarket::<T>::try_mutate(
			proposal_id,
			|value| -> Result<BalanceOf<T>, DispatchError> {
				let old_amount = value.ok_or(Error::<T>::ProposalIdNotExist)?;
				let new_amount = old_amount
					.checked_sub(&diff)
					.ok_or(Error::<T>::BalanceOverflow)?;
				*value = Some(new_amount);
				Ok(old_amount.checked_sub(&new_amount).unwrap_or(Zero::zero()))
			},
		)?;
		ProposalAccountInfo::<T>::try_mutate(
			proposal_id,
			&who,
			|value| -> Result<BalanceOf<T>, DispatchError> {
				let old_amount = value.unwrap_or(Zero::zero());
				let new_amount = old_amount
					.checked_sub(&diff)
					.ok_or(Error::<T>::BalanceOverflow)?;
				*value = Some(new_amount);
				Ok(new_amount.checked_sub(&old_amount).unwrap_or(Zero::zero()))
			},
		)?;
		Ok(())
	}
}

impl<T: Config> LiquidityPool<T::AccountId, ProposalIdOf<T>, MomentOf<T>, CategoryIdOf<T>>
	for Pallet<T>
{
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;

	fn new_liquidity_pool(
		who: &T::AccountId,
		proposal_id: T::ProposalId,
		title: Vec<u8>,
		close_time: MomentOf<T>,
		category_id: T::CategoryId,
		currency_id: CurrencyIdOf<T>,
		optional: [Vec<u8>; 2],
		number: BalanceOf<T>,
		earn_fee: u32,
		detail: Vec<u8>,
	) -> Result<(), DispatchError> {
		Self::init_pool(
			&who,
			proposal_id,
			title,
			close_time,
			category_id,
			currency_id,
			optional,
			number,
			earn_fee,
			detail,
		)
	}

	fn time(proposal_id: T::ProposalId) -> Result<(MomentOf<T>, MomentOf<T>), DispatchError> {
		let start =
			ProposalCreateTime::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
		let end = ProposalCloseTime::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
		Ok((start, end))
	}
}
