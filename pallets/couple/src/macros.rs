#[macro_export]
macro_rules! value_changed {
    ($value: ident, $old_amount: ident, $new_expr: expr) => {{
        let $old_amount = $value.unwrap_or_else(Zero::zero);
        let new_amount = $new_expr;
        *$value = Some(new_amount);
        ($old_amount, new_amount)
    }};
}

#[macro_export]
macro_rules! proposal_total_market_try_mutate {
    ($proposal_id: ident, $old_amount: ident, $new_expr: expr) => {
        storage_try_mutate!(ProposalTotalMarket, T, $proposal_id, |value| -> Result<
            BalanceOf<T>,
            DispatchError,
        > {
            let $old_amount = value.ok_or(Error::<T>::ProposalIdNotExist)?;
            let new_amount = $new_expr;
            *value = Some(new_amount);
            Ok(new_amount
                .checked_sub(&$old_amount)
                .unwrap_or_else(Zero::zero))
        })
    };
}

#[macro_export]
macro_rules! proposal_account_info_try_mutate {
    ($proposal_id: ident, $who: ident, $old_amount: ident, $new_expr: expr) => {
        storage_try_mutate!(
            ProposalAccountInfo,
            T,
            $proposal_id,
            &$who,
            |value| -> Result<BalanceOf<T>, DispatchError> {
                let ($old_amount, new_amount) = value_changed!(value, $old_amount, $new_expr);
                Ok(new_amount
                    .checked_sub(&$old_amount)
                    .unwrap_or_else(Zero::zero))
            },
        )
    };
}

#[macro_export]
macro_rules! proposal_total_optional_market_try_mutate {
    ($proposal_id: ident, $o1: ident, $o2: ident, $new_expr: expr) => {
        storage_try_mutate!(
            ProposalTotalOptionalMarket,
            T,
            $proposal_id,
            |item| -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
                let ($o1, $o2) = item.ok_or(Error::<T>::ProposalIdNotExist)?;
                let (new_o1, new_o2) = $new_expr;
                *item = Some((new_o1, new_o2));
                Ok((sub_abs!(new_o1, $o1), sub_abs!(new_o2, $o2)))
            }
        )
    };
}

#[macro_export]
macro_rules! proposal_total_market_fee_try_mutate {
    ($proposal_id: ident, $old_amount: ident, $new_expr: expr) => {
        storage_try_mutate!(ProposalTotalMarketFee, T, $proposal_id, |value| -> Result<
            BalanceOf<T>,
            DispatchError,
        > {
            let ($old_amount, _) = value_changed!(value, $old_amount, $new_expr);
            Ok($old_amount)
        })
    };
}

#[macro_export]
macro_rules! proposal_total_market_liquid_try_mutate {
    ($proposal_id: ident, $old_amount: ident, $new_expr: expr) => {
        storage_try_mutate!(
            ProposalTotalMarketLiquid,
            T,
            $proposal_id,
            |value| -> Result<BalanceOf<T>, DispatchError> {
                let ($old_amount, _) = value_changed!(value, $old_amount, $new_expr);
                Ok($old_amount)
            }
        )
    };
}
