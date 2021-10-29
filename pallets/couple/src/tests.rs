use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok, traits::Time};
use xpmrl_traits::{pool::LiquidityPool, tokens::Tokens, ProposalStatus as ProposalState};

fn create_proposal(
    account: AccountId,
    currency_id: CurrencyIdOf<Test>,
    number: BalanceOf<Test>,
    rate: u32,
    step: MomentOf<Test>,
) -> ProposalIdOf<Test> {
    let now = <Timestamp as Time>::now();
    let close_time = now + step;
    assert_ok!(CoupleModule::new_proposal(
        Origin::signed(account),
        "how to test this module".as_bytes().to_vec(),
        [
            "the one".as_bytes().to_vec(),
            "other one".as_bytes().to_vec()
        ],
        close_time,
        1,
        currency_id,
        number,
        rate,
        "proposal detail".as_bytes().to_vec(),
    ));
    <ProposalsWrapper as LiquidityPool<Test>>::max_proposal_id() - 1
}

#[test]
fn test_new_proposal() {
    new_test_ext().execute_with(|| {
        let number: BalanceOf<Test> = 100000;
        let step: MomentOf<Test> = 10;
        let fee_rate: u32 = 2000;
        let currency_id: CurrencyIdOf<Test> = 1;
        let account: AccountId = 1;
        let id = create_proposal(account, currency_id, number, fee_rate, step);

        assert_eq!(CoupleModule::pool_pairs(id), Some((3, 4)));
        assert_eq!(CoupleModule::proposal_currency_id(id), Some(currency_id));
        assert_eq!(CoupleModule::proposal_liquidate_currency_id(id), Some(5));
        assert_eq!(
            CoupleModule::proposal_total_earn_trading_fee(id),
            Some(fee_rate)
        );
        assert_eq!(CoupleModule::proposal_result(id), None);
        assert_eq!(
            CoupleModule::proposal_account_info(id, account),
            Some(number)
        );
        assert_eq!(CoupleModule::proposal_total_market(id), Some(number));
        assert_eq!(
            CoupleModule::proposal_total_optional_market(id),
            Some((number, number))
        );
        assert_eq!(CoupleModule::proposal_total_market_fee(id), None);
        assert_eq!(CoupleModule::proposal_total_market_liquid(id), Some(number));
    });
}

#[test]
fn test_add_liquidity() {
    new_test_ext().execute_with(|| {
        let number: BalanceOf<Test> = 100000;
        let other_account: AccountId = 2;
        let id = create_proposal(1, 1, number, 2000, 10);

        let next_number: BalanceOf<Test> = 100;
        assert_noop!(
            CoupleModule::add_liquidity(Origin::signed(other_account), id + 1, next_number),
            Error::<Test>::ProposalIdNotExist
        );
        assert_noop!(
            CoupleModule::add_liquidity(Origin::signed(other_account), id, next_number),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_ok!(CoupleModule::add_liquidity(
            Origin::signed(other_account),
            id,
            next_number
        ));
        let add_liquidity_event = Event::couple(crate::Event::AddLiquidity(
            other_account,
            id,
            1,
            next_number,
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == add_liquidity_event));
        assert_eq!(
            CoupleModule::proposal_total_market(id),
            Some(number + next_number)
        );
        assert_eq!(
            CoupleModule::proposal_total_market_liquid(id),
            Some(number + next_number)
        );
        assert_eq!(
            CoupleModule::proposal_total_market(id),
            Some(number + next_number)
        );
        assert_eq!(
            CoupleModule::proposal_account_info(id, other_account),
            Some(next_number)
        );
        assert_eq!(
            XPMRLTokens::free_balance_of(other_account, 5),
            Some(next_number)
        );
    });
}

#[test]
fn test_remove_liquidity() {
    new_test_ext().execute_with(|| {
        let account: AccountId = 1;
        let other_account: AccountId = 2;
        let number: BalanceOf<Test> = 100000;
        let id = create_proposal(account, 1, number, 2000, 10);
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_ok!(CoupleModule::buy(
            Origin::signed(other_account),
            id,
            3,
            31250
        ));
        assert_noop!(
            CoupleModule::remove_liquidity(Origin::signed(account), id, number),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::WaitingForResults
            )
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), id, 3));
        assert_noop!(
            CoupleModule::remove_liquidity(Origin::signed(account), 1, number),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(CoupleModule::remove_liquidity(
            Origin::signed(account),
            id,
            number
        ));

        let remove_liquidity_event =
            Event::couple(crate::Event::RemoveLiquidity(account, id, 1, number));
        assert!(System::events()
            .iter()
            .any(|record| record.event == remove_liquidity_event));

        assert_eq!(XPMRLTokens::free_balance_of(1, 1), Some(86250));
        assert_eq!(XPMRLTokens::free_balance_of(1, 4), Some(45000));
        assert_eq!(XPMRLTokens::free_balance_of(2, 3), Some(45000));
    });
}

#[test]
fn test_buy() {
    new_test_ext().execute_with(|| {
        let other_account: AccountId = 2;
        let id = create_proposal(1, 1, 100000, 2000, 10);

        assert_noop!(
            CoupleModule::buy(Origin::signed(other_account), id, 3, 31250),
            Error::<Test>::ProposalAbnormalState
        );
        assert_noop!(
            CoupleModule::buy(Origin::signed(other_account), id + 1, 3, 31250),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_noop!(
            CoupleModule::buy(Origin::signed(other_account), id, 5, 31250),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::buy(
            Origin::signed(other_account),
            id,
            3,
            31250
        ));

        let buy_event = Event::couple(crate::Event::Buy(other_account, id, 3, 25000));
        assert!(System::events()
            .iter()
            .any(|record| record.event == buy_event));

        assert_eq!(XPMRLTokens::free_balance_of(other_account, 3), Some(45000));
        assert_eq!(
            CoupleModule::proposal_total_optional_market(id),
            Some((80000, 125000))
        );
        assert_eq!(CoupleModule::proposal_total_market_fee(id), Some(6250));
        assert_eq!(
            CoupleModule::proposal_account_info(id, other_account),
            Some(25000)
        );
    });
}

#[test]
fn test_sell() {
    new_test_ext().execute_with(|| {
        let number: BalanceOf<Test> = 100000;
        let other_account: AccountId = 2;
        let id = create_proposal(1, 1, number, 2000, 10);

        assert_noop!(
            CoupleModule::sell(Origin::signed(other_account), id, 3, 255),
            Error::<Test>::ProposalAbnormalState
        );
        assert_noop!(
            CoupleModule::sell(Origin::signed(other_account), id + 1, 3, 255),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_noop!(
            CoupleModule::sell(Origin::signed(other_account), id, 5, 255),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::buy(
            Origin::signed(other_account),
            id,
            3,
            31250
        ));
        assert_ok!(CoupleModule::sell(
            Origin::signed(other_account),
            id,
            3,
            45000
        ));

        let sell_event = Event::couple(crate::Event::Sell(other_account, id, 3, 20000));
        assert!(System::events()
            .iter()
            .any(|record| record.event == sell_event));

        assert_eq!(XPMRLTokens::free_balance_of(other_account, 3), Some(0));
        assert_eq!(
            CoupleModule::proposal_total_optional_market(id),
            Some((number, number))
        );
        assert_eq!(CoupleModule::proposal_total_market_fee(id), Some(11250));
        assert_eq!(
            CoupleModule::proposal_account_info(id, other_account),
            Some(0)
        );
    });
}

#[test]
fn test_retrieval() {
    new_test_ext().execute_with(|| {
        let number: BalanceOf<Test> = 100000;
        let account: AccountId = 1;
        let other_account: AccountId = 2;
        let currency_id: CurrencyIdOf<Test> = 1;
        let id = create_proposal(account, currency_id, number, 2000, 10);
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(account), id + 1, 3, number),
            Error::<Test>::ProposalIdNotExist
        );
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(account), id, 3, number),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(id, ProposalState::End)
        );
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(account), id, 3, number),
            Error::<Test>::ProposalNotResult
        );
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_ok!(CoupleModule::buy(
            Origin::signed(other_account),
            id,
            3,
            31250
        ));
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::WaitingForResults
            )
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), id, 3));
        assert_ok!(CoupleModule::remove_liquidity(
            Origin::signed(account),
            id,
            number
        ));
        assert_ok!(CoupleModule::retrieval(
            Origin::signed(account),
            id,
            3,
            number
        ));
        let retrieval_event = Event::couple(crate::Event::Retrieval(account, id, 3, 0));
        assert!(System::events()
            .iter()
            .any(|record| record.event == retrieval_event));

        assert_ok!(CoupleModule::retrieval(
            Origin::signed(other_account),
            id,
            3,
            number
        ));
        let retrieval_event = Event::couple(crate::Event::Retrieval(other_account, id, 3, 44775));
        assert!(System::events()
            .iter()
            .any(|record| record.event == retrieval_event));

        assert_eq!(XPMRLTokens::free_balance_of(account, 1), Some(86250));
        assert_eq!(XPMRLTokens::free_balance_of(other_account, 1), Some(44775));
        assert_eq!(CoupleModule::proposal_total_market(0), Some(0));
    });
}

#[test]
fn test_withdrawal_reward() {
    new_test_ext().execute_with(|| {
        let number: BalanceOf<Test> = 100000;
        let account: AccountId = 1;
        let other_account: AccountId = 2;
        let currency_id: CurrencyIdOf<Test> = 1;
        let id = create_proposal(account, currency_id, number, 2000, 10);
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::FormalPrediction
            )
        );
        assert_ok!(CoupleModule::buy(
            Origin::signed(other_account),
            id,
            3,
            31250
        ));
        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::WaitingForResults
            )
        );
        AutonomyWrapper::set_temporary_results(id, &4, 3);
        AutonomyWrapper::set_temporary_results(id, &5, 3);
        AutonomyWrapper::set_temporary_results(id, &6, 4);
        assert_noop!(
            CoupleModule::withdrawal_reward(Origin::signed(4), id),
            Error::<Test>::ProposalNotResult
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), id, 3));
        assert_noop!(
            CoupleModule::withdrawal_reward(Origin::signed(6), id),
            Error::<Test>::UploadedNotResult
        );
        let number = <TokensOf<Test> as Tokens<AccountId>>::balance(3, &2);
        assert_ok!(CoupleModule::retrieval(
            Origin::signed(other_account),
            id,
            3,
            number
        ));
        let fee = number * 5 / 1000 / 2;
        assert_eq!(CoupleModule::proposal_total_autonomy_reward(id), Some(fee));
        assert_ok!(CoupleModule::withdrawal_reward(Origin::signed(4), id));
        let event = Event::couple(crate::Event::WithdrawalReward(4, id, 56));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(
            CoupleModule::proposal_account_reward_start(id, 4),
            Some(112)
        );
        assert_eq!(CoupleModule::proposal_current_autonomy_reward(id), Some(56));
        assert_ok!(CoupleModule::withdrawal_reward(Origin::signed(5), id));
        let event = Event::couple(crate::Event::WithdrawalReward(5, id, 56));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(
            CoupleModule::proposal_account_reward_start(id, 5),
            Some(112)
        );
        assert_eq!(CoupleModule::proposal_current_autonomy_reward(id), Some(0));
    })
}

#[test]
fn test_set_result() {
    new_test_ext().execute_with(|| {
        let id = create_proposal(1, 1, 100000, 200, 10);

        assert_ok!(
            <ProposalsWrapper as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalState::WaitingForResults
            )
        );
        assert_noop!(
            CoupleModule::set_result(Origin::root(), id, 5),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), id, 3));

        let set_result_event = Event::couple(crate::Event::SetResult(id, 3));
        assert!(System::events()
            .iter()
            .any(|record| record.event == set_result_event));
        assert_eq!(
            <ProposalsWrapper as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalState::End)
        );
    });
}
