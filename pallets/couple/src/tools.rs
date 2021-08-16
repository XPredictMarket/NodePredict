use crate::*;

use frame_support::{
    ensure,
    traits::{Get, Time},
};
use num_traits::pow::pow;
use sp_runtime::{
    traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot, One, Zero},
    DispatchError,
};
use sp_std::{cmp, vec::Vec};
use xpmrl_traits::{
    pool::LiquidityPool, ruler::RulerAccounts, tokens::Tokens, ProposalStatus, RulerModule,
};
use xpmrl_utils::{runtime_format, storage_try_mutate, sub_abs};

impl<T: Config> Pallet<T> {
    pub(crate) fn quadratic_equation(
        a: BalanceOf<T>,
        b: BalanceOf<T>,
        c: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
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
        let tmp = sqrt_delta.checked_sub(&b).unwrap_or_else(Zero::zero);
        Ok(tmp
            .checked_div(&2u32.into())
            .ok_or(Error::<T>::BalanceOverflow)?)
    }

    pub(crate) fn adjust_pool(
        to_add: usize,
        number: BalanceOf<T>,
        old_pair: &[BalanceOf<T>; 2],
    ) -> Result<[BalanceOf<T>; 2], DispatchError> {
        let base = old_pair[0]
            .checked_mul(&old_pair[1])
            .ok_or(Error::<T>::BalanceOverflow)?;
        let mut new_pair = *old_pair;
        new_pair[to_add] = new_pair[to_add]
            .checked_add(&number)
            .ok_or(Error::<T>::BalanceOverflow)?;
        new_pair[1 - to_add] = base
            .checked_div(&new_pair[to_add])
            .ok_or(Error::<T>::BalanceOverflow)?;
        Ok(new_pair)
    }

    pub(crate) fn get_other_optional_id(
        proposal_id: ProposalIdOf<T>,
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

    pub(crate) fn get_fee_from_total(
        proposal_id: ProposalIdOf<T>,
        number: BalanceOf<T>,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
        let fee_decimals: u8 = T::EarnTradingFeeDecimals::get();
        let one = pow(10u32, fee_decimals.into());
        let fee_rate = ProposalTotalEarnTradingFee::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalIdNotExist)?;
        let mut rate = number
            .checked_mul(&(fee_rate.into()))
            .ok_or(Error::<T>::BalanceOverflow)?;
        rate = rate
            .checked_div(&(one.into()))
            .ok_or(Error::<T>::BalanceOverflow)?;
        let actual_number = number.checked_sub(&rate).unwrap_or_else(Zero::zero);
        Ok((actual_number, rate))
    }

    pub(crate) fn get_fee_of_liquid(
        proposal_id: ProposalIdOf<T>,
        number: BalanceOf<T>,
        total_liquid: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let market_fee = ProposalFinallyMarketFee::<T>::get(proposal_id).unwrap_or_else(Zero::zero);

        let decimals = T::EarnTradingFeeDecimals::get();
        let one = pow(10u32, decimals.into());
        let liquidity_provider_fee_rate: u32 =
            ProposalLiquidityProviderFeeRate::<T>::get().unwrap_or_else(Zero::zero);

        let mul_market_fee = market_fee
            .checked_mul(&number)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let mul_market_fee = mul_market_fee
            .checked_mul(&liquidity_provider_fee_rate.into())
            .ok_or(Error::<T>::BalanceOverflow)?;
        let fee = mul_market_fee
            .checked_div(&total_liquid)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let fee = fee
            .checked_div(&one.into())
            .ok_or(Error::<T>::BalanceOverflow)?;
        Ok(fee)
    }

    pub(crate) fn get_fee_of_creator(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let owner = T::Pool::proposal_owner(proposal_id)?;
        if owner == *who && !ProposalOwnerAlreadyWithdrawnFee::<T>::contains_key(proposal_id, &who)
        {
            let market_fee =
                ProposalFinallyMarketFee::<T>::get(proposal_id).unwrap_or_else(Zero::zero);

            let decimals = T::EarnTradingFeeDecimals::get();
            let one = pow(10u32, decimals.into());
            let liquidity_provider_fee_rate: u32 =
                ProposalLiquidityProviderFeeRate::<T>::get().unwrap_or_else(Zero::zero);

            let mul_market_fee = market_fee
                .checked_mul(&liquidity_provider_fee_rate.into())
                .ok_or(Error::<T>::BalanceOverflow)?;
            let fee = mul_market_fee
                .checked_div(&one.into())
                .ok_or(Error::<T>::BalanceOverflow)?;
            let fee = market_fee.checked_sub(&fee).unwrap_or_else(Zero::zero);
            ProposalOwnerAlreadyWithdrawnFee::<T>::insert(proposal_id, &who, fee);
            Ok(fee)
        } else {
            Ok(Zero::zero())
        }
    }

    pub(crate) fn get_withdrawal_fee(
        number: BalanceOf<T>,
    ) -> (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>) {
        let rate = ProposalWithdrawalFeeRate::<T>::get().unwrap_or_else(Zero::zero);
        let decimals: u8 = T::EarnTradingFeeDecimals::get();
        let scale = pow(10u32, decimals.into());
        let fee = number.checked_mul(&rate.into()).unwrap_or_else(Zero::zero);
        let fee = fee.checked_div(&scale.into()).unwrap_or_else(Zero::zero);
        let number = number.checked_sub(&fee).unwrap_or_else(Zero::zero);
        let reward = fee.checked_div(&2u32.into()).unwrap_or_else(Zero::zero);
        let dividends = fee.checked_sub(&reward).unwrap_or_else(Zero::zero);
        (number, reward, dividends)
    }

    pub(crate) fn finally_locked(proposal_id: ProposalIdOf<T>) -> Result<(), DispatchError> {
        let finally_liquid =
            ProposalTotalMarketLiquid::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
        let finally_fee = ProposalTotalMarketFee::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
        let finally_optional = ProposalTotalOptionalMarket::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalIdNotExist)?;
        ProposalFinallyMarketFee::<T>::insert(proposal_id, finally_fee);
        ProposalFinallyMarketLiquid::<T>::insert(proposal_id, finally_liquid);
        ProposalFinallyTotalOptionalMarket::<T>::insert(proposal_id, finally_optional);
        Ok(())
    }

    pub(crate) fn appropriation(
        currency_id: CurrencyIdOf<T>,
        who: &T::AccountId,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        <TokensOf<T> as Tokens<T::AccountId>>::appropriation(currency_id, &who, number)
    }

    pub(crate) fn new_asset(
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> Result<CurrencyIdOf<T>, DispatchError> {
        <TokensOf<T> as Tokens<T::AccountId>>::new_asset(name, symbol, decimals)
    }

    pub(crate) fn init_pool(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        title: Vec<u8>,
        close_time: MomentOf<T>,
        category_id: T::CategoryId,
        earn_fee: u32,
        detail: Vec<u8>,
    ) -> Result<ProposalIdOf<T>, DispatchError> {
        let version: VersionIdOf<T> = T::CurrentLiquidateVersionId::get();
        Proposals::<T>::insert(
            proposal_id,
            Proposal {
                title,
                category_id,
                detail,
            },
        );
        T::Pool::init_proposal(
            proposal_id,
            &who,
            ProposalStatus::OriginalPrediction,
            T::Time::now(),
            close_time,
            version,
        );
        ProposalTotalEarnTradingFee::<T>::insert(proposal_id, earn_fee);
        Ok(proposal_id)
    }

    pub(crate) fn new_currency(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        optional: [Vec<u8>; 2],
    ) -> Result<ProposalIdOf<T>, DispatchError> {
        ProposalCurrencyId::<T>::insert(proposal_id, currency_id);
        <TokensOf<T> as Tokens<T::AccountId>>::donate(currency_id, &who, number)?;
        let decimals = <TokensOf<T> as Tokens<T::AccountId>>::decimals(currency_id)?;
        let yes_symbol = runtime_format!("{:?}-YES", proposal_id);
        let asset_id_1 = Self::new_asset(optional[0].clone(), yes_symbol, decimals)?;
        let no_symbol = runtime_format!("{:?}-NO", proposal_id);
        let asset_id_2 = Self::new_asset(optional[1].clone(), no_symbol, decimals)?;
        let lp_name = runtime_format!("LP-{:?}", proposal_id);
        let asset_id_lp = Self::new_asset(lp_name.clone(), lp_name, decimals)?;

        T::Pool::append_used_currency(asset_id_1);
        T::Pool::append_used_currency(asset_id_2);
        T::Pool::append_used_currency(asset_id_lp);

        <TokensOf<T> as Tokens<T::AccountId>>::mint_donate(asset_id_1, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint_donate(asset_id_2, number)?;
        ProposalTotalOptionalMarket::<T>::insert(proposal_id, (number, number));

        ProposalLiquidateCurrencyId::<T>::insert(proposal_id, asset_id_lp);
        <TokensOf<T> as Tokens<T::AccountId>>::mint(asset_id_lp, &who, number)?;

        PoolPairs::<T>::insert(proposal_id, (asset_id_1, asset_id_2));
        ProposalAccountInfo::<T>::insert(proposal_id, who.clone(), number);
        ProposalTotalMarket::<T>::insert(proposal_id, number);
        ProposalTotalMarketLiquid::<T>::insert(proposal_id, number);
        Ok(proposal_id)
    }

    pub(crate) fn total_and_account_add(
        proposal_id: ProposalIdOf<T>,
        who: &T::AccountId,
        diff: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        proposal_total_market_try_mutate!(
            proposal_id,
            old_amount,
            old_amount
                .checked_add(&diff)
                .ok_or(Error::<T>::BalanceOverflow)?
        )?;
        proposal_account_info_try_mutate!(
            proposal_id,
            who,
            old_amount,
            old_amount
                .checked_add(&diff)
                .ok_or(Error::<T>::BalanceOverflow)?
        )?;
        Ok(())
    }

    pub(crate) fn total_and_account_sub(
        proposal_id: ProposalIdOf<T>,
        who: &T::AccountId,
        diff: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        proposal_total_market_try_mutate!(
            proposal_id,
            old_amount,
            old_amount.checked_sub(&diff).unwrap_or_else(Zero::zero)
        )?;
        proposal_account_info_try_mutate!(
            proposal_id,
            who,
            old_amount,
            old_amount.checked_sub(&diff).unwrap_or_else(Zero::zero)
        )?;
        Ok(())
    }

    pub(crate) fn inner_add_liquidity(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        asset_id_1: CurrencyIdOf<T>,
        asset_id_2: CurrencyIdOf<T>,
        liquidate_currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        <TokensOf<T> as Tokens<T::AccountId>>::donate(currency_id, &who, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint_donate(asset_id_1, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint_donate(asset_id_2, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint(liquidate_currency_id, &who, number)?;
        proposal_total_optional_market_try_mutate!(proposal_id, o1, o2, {
            let new_o1 = o1.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?;
            let new_o2 = o2.checked_add(&number).ok_or(Error::<T>::BalanceOverflow)?;
            (new_o1, new_o2)
        })?;
        proposal_total_market_liquid_try_mutate!(
            proposal_id,
            old_value,
            old_value
                .checked_add(&number)
                .ok_or(Error::<T>::BalanceOverflow)?
        )?;
        Self::total_and_account_add(proposal_id, &who, number)
    }

    pub(crate) fn inner_remove_liquidity(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        liquidate_currency_id: CurrencyIdOf<T>,
        asset_id_1: CurrencyIdOf<T>,
        asset_id_2: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        finally_o1: BalanceOf<T>,
        finally_o2: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        <TokensOf<T> as Tokens<T::AccountId>>::burn(liquidate_currency_id, &who, number)?;
        proposal_total_market_liquid_try_mutate!(
            proposal_id,
            old_value,
            old_value.checked_sub(&number).unwrap_or_else(Zero::zero)
        )?;
        let total_liquid =
            ProposalFinallyMarketLiquid::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
        let fee = Self::get_fee_of_liquid(proposal_id, number, total_liquid)?;
        let creater_fee = Self::get_fee_of_creator(&who, proposal_id)?;
        let fee = fee
            .checked_add(&creater_fee)
            .ok_or(Error::<T>::BalanceOverflow)?;
        proposal_total_market_fee_try_mutate!(
            proposal_id,
            old_value,
            old_value.checked_sub(&fee).unwrap_or_else(Zero::zero)
        )?;
        let (o1, o2) = proposal_total_optional_market_try_mutate!(proposal_id, o1, o2, {
            let new_o1 = finally_o1
                .checked_mul(&number)
                .ok_or(Error::<T>::BalanceOverflow)?;
            let new_o1 = new_o1
                .checked_div(&total_liquid)
                .ok_or(Error::<T>::BalanceOverflow)?;
            let new_o1 = o1.checked_sub(&new_o1).unwrap_or_else(Zero::zero);

            let new_o2 = finally_o2
                .checked_mul(&number)
                .ok_or(Error::<T>::BalanceOverflow)?;
            let new_o2 = new_o2
                .checked_div(&total_liquid)
                .ok_or(Error::<T>::BalanceOverflow)?;
            let new_o2 = o2.checked_sub(&new_o2).unwrap_or_else(Zero::zero);
            (new_o1, new_o2)
        })?;
        let min = cmp::min(o1, o2);
        <TokensOf<T> as Tokens<T::AccountId>>::burn_donate(asset_id_1, min)?;
        <TokensOf<T> as Tokens<T::AccountId>>::burn_donate(asset_id_2, min)?;
        Self::total_and_account_sub(proposal_id, &who, min)?;
        let actual_amount = min.checked_add(&fee).ok_or(Error::<T>::BalanceOverflow)?;
        Self::appropriation(currency_id, &who, actual_amount)?;
        let yes_amount = o1.checked_sub(&min).unwrap_or_else(Zero::zero);
        Self::appropriation(asset_id_1, &who, yes_amount)?;
        let no_amount = o2.checked_sub(&min).unwrap_or_else(Zero::zero);
        Self::appropriation(asset_id_2, &who, no_amount)
    }

    pub(crate) fn inner_buy(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        optional_currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        other_currency: (usize, CurrencyIdOf<T>),
    ) -> Result<BalanceOf<T>, DispatchError> {
        let (actual_number, fee) = Self::get_fee_from_total(proposal_id, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::donate(currency_id, &who, number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint(optional_currency_id, &who, actual_number)?;
        <TokensOf<T> as Tokens<T::AccountId>>::mint_donate(other_currency.1, actual_number)?;
        let (d1, d2) = proposal_total_optional_market_try_mutate!(proposal_id, o1, o2, {
            let old_pair = [o1, o2];
            let new_pair = Self::adjust_pool(other_currency.0, actual_number, &old_pair)?;
            (new_pair[0], new_pair[1])
        })?;
        let diff = [d1, d2][1 - other_currency.0];
        Self::total_and_account_add(proposal_id, &who, actual_number)?;
        proposal_total_market_fee_try_mutate!(
            proposal_id,
            old_value,
            old_value
                .checked_add(&fee)
                .ok_or(Error::<T>::BalanceOverflow)?
        )?;
        Self::appropriation(optional_currency_id, &who, diff)?;
        Ok(actual_number)
    }

    pub(crate) fn inner_sell(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        currency_id: CurrencyIdOf<T>,
        optional_currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
        other_currency: (usize, CurrencyIdOf<T>),
    ) -> Result<BalanceOf<T>, DispatchError> {
        <TokensOf<T> as Tokens<T::AccountId>>::donate(optional_currency_id, &who, number)?;
        let (d1, d2) = proposal_total_optional_market_try_mutate!(proposal_id, o1, o2, {
            let old_pair = [o1, o2];
            let other_currency = Self::get_other_optional_id(proposal_id, optional_currency_id)?;
            let current_index = 1 - other_currency.0;

            let b: BalanceOf<T> = o1.checked_add(&o2).ok_or(Error::<T>::BalanceOverflow)?;
            let b: BalanceOf<T> = b.checked_sub(&number).unwrap_or_else(Zero::zero);
            let c: BalanceOf<T> = number
                .checked_mul(&old_pair[current_index])
                .ok_or(Error::<T>::BalanceOverflow)?;
            let actual_number = Self::quadratic_equation(One::one(), b, c)?;

            let new_pair = Self::adjust_pool(current_index, actual_number, &old_pair)?;
            (new_pair[0], new_pair[1])
        })?;
        let diff = [d1, d2];
        let last_select_currency = number
            .checked_sub(&diff[1 - other_currency.0])
            .unwrap_or_else(Zero::zero);
        let acquired_currency = diff[other_currency.0];
        let min = cmp::min(last_select_currency, acquired_currency);
        <TokensOf<T> as Tokens<T::AccountId>>::burn_donate(other_currency.1, min)?;
        let (actual_number, fee) = Self::get_fee_from_total(proposal_id, min)?;
        proposal_total_market_fee_try_mutate!(
            proposal_id,
            old_value,
            old_value
                .checked_add(&fee)
                .ok_or(Error::<T>::BalanceOverflow)?
        )?;
        Self::total_and_account_sub(proposal_id, &who, min)?;
        Self::appropriation(currency_id, &who, actual_number)?;
        let last = last_select_currency
            .checked_sub(&min)
            .unwrap_or_else(Zero::zero);
        Self::appropriation(optional_currency_id, &who, last)?;
        let acquired = acquired_currency
            .checked_sub(&min)
            .unwrap_or_else(Zero::zero);
        Self::appropriation(other_currency.1, &who, acquired)?;
        Ok(actual_number)
    }

    pub(crate) fn inner_retrieval(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        result_id: CurrencyIdOf<T>,
        optional_currency_id: CurrencyIdOf<T>,
        number: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if optional_currency_id == result_id {
            let currency_id =
                ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
            proposal_total_market_try_mutate!(
                proposal_id,
                old_amount,
                old_amount.checked_sub(&number).unwrap_or_else(Zero::zero)
            )?;
            <TokensOf<T> as Tokens<T::AccountId>>::burn(result_id, &who, number)?;
            let (number, reward, dividends) = Self::get_withdrawal_fee(number);
            ProposalTotalAutonomyReward::<T>::try_mutate(
                proposal_id,
                |optional| -> Result<(), DispatchError> {
                    let old = optional.unwrap_or_else(Zero::zero);
                    *optional = Some(old.checked_add(&reward).unwrap_or_else(Zero::zero));
                    Ok(())
                },
            )?;
            ProposalCurrentAutonomyReward::<T>::try_mutate(
                proposal_id,
                |optional| -> Result<(), DispatchError> {
                    let old = optional.unwrap_or_else(Zero::zero);
                    *optional = Some(old.checked_add(&reward).unwrap_or_else(Zero::zero));
                    Ok(())
                },
            )?;
            let dividends_account = T::Ruler::get_account(RulerModule::PlatformDividend)?;
            Self::appropriation(currency_id, &dividends_account, dividends)?;
            Self::appropriation(currency_id, &who, number)
        } else {
            <TokensOf<T> as Tokens<T::AccountId>>::burn(optional_currency_id, &who, number)
        }
    }

    pub(crate) fn inner_withdrawal_reward(
        who: &T::AccountId,
        proposal_id: ProposalIdOf<T>,
        total: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let total_reward =
            ProposalTotalAutonomyReward::<T>::get(proposal_id).unwrap_or_else(Zero::zero);
        let currency_id =
            ProposalCurrencyId::<T>::get(proposal_id).ok_or(Error::<T>::ProposalIdNotExist)?;
        let start_reward = ProposalAccountRewardStart::<T>::try_mutate(
            proposal_id,
            &who,
            |optional| -> Result<BalanceOf<T>, DispatchError> {
                let old = optional.unwrap_or_else(Zero::zero);
                *optional = Some(total_reward);
                Ok(old)
            },
        )?;
        let number = ProposalCurrentAutonomyReward::<T>::try_mutate_exists(
            proposal_id,
            |optional| -> Result<BalanceOf<T>, DispatchError> {
                let old = optional.unwrap_or_else(Zero::zero);
                let diff = total_reward
                    .checked_sub(&start_reward)
                    .unwrap_or_else(Zero::zero);
                let number = diff.checked_div(&total).unwrap_or_else(Zero::zero);
                *optional = old.checked_sub(&number);
                Ok(number)
            },
        )?;
        Self::appropriation(currency_id, &who, number)
    }
}
