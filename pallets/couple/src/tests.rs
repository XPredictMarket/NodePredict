use std::time::SystemTime;

use crate::pallet::Proposal;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use xpmrl_traits::ProposalStatus;

fn befor_test() -> u128 {
    let proposal = Proposal {
        title: "how to test this module".as_bytes().to_vec(),
        category_id: 1,
        detail: "proposal detail".as_bytes().to_vec(),
    };
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let now = now.as_secs() as u32 + 1_000_000;
    let currency_id: u32 = 1;
    let fee_rate: u32 = 2000;
    let number = 100000;
    assert_ok!(XPMRLProposals::new_proposal(
        Origin::signed(1),
        proposal.title.clone(),
        [
            "the one".as_bytes().to_vec(),
            "other one".as_bytes().to_vec()
        ],
        now,
        proposal.category_id,
        currency_id,
        number,
        fee_rate,
        proposal.detail.clone()
    ));
    assert_eq!(CoupleModule::pool_pairs(0), Some((3, 4)));
    assert_eq!(CoupleModule::proposal_currency_id(0), Some(currency_id));
    assert_eq!(CoupleModule::proposal_liquidate_currency_id(0), Some(5));
    assert_eq!(
        CoupleModule::proposal_total_earn_trading_fee(0),
        Some(fee_rate)
    );
    assert_eq!(CoupleModule::proposal_result(0), None);
    assert_eq!(CoupleModule::proposal_account_info(0, 1), Some(number));
    assert_eq!(CoupleModule::proposal_total_market(0), Some(number));
    assert_eq!(
        CoupleModule::proposal_total_optional_market(0),
        Some((number, number))
    );
    assert_eq!(CoupleModule::proposal_total_market_fee(0), None);
    assert_eq!(CoupleModule::proposal_total_market_liquid(0), Some(number));

    number
}

#[test]
fn test_add_liquidity() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        let next_number = 100;
        assert_noop!(
            CoupleModule::add_liquidity(Origin::signed(2), 1, next_number),
            Error::<Test>::ProposalIdNotExist
        );
        assert_noop!(
            CoupleModule::add_liquidity(Origin::signed(2), 0, next_number),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::FormalPrediction
        ));
        assert_ok!(CoupleModule::add_liquidity(
            Origin::signed(2),
            0,
            next_number
        ));
        let add_liquidity_event = Event::couple(crate::Event::AddLiquidity(2, 0, 1, next_number));
        assert!(System::events()
            .iter()
            .any(|record| record.event == add_liquidity_event));
        assert_eq!(
            CoupleModule::proposal_total_market(0),
            Some(number + next_number)
        );
        assert_eq!(
            CoupleModule::proposal_total_market_liquid(0),
            Some(number + next_number)
        );
        assert_eq!(
            CoupleModule::proposal_total_market(0),
            Some(number + next_number)
        );
        assert_eq!(CoupleModule::proposal_account_info(0, 2), Some(next_number));
        assert_eq!(XPMRLTokens::balance_of(2, 5), Some(next_number));
    });
}

#[test]
fn test_remove_liquidity() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::FormalPrediction
        ));
        assert_ok!(CoupleModule::buy(Origin::signed(2), 0, 3, 31250));
        assert_noop!(
            CoupleModule::remove_liquidity(Origin::signed(1), 0, number),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), 0, 3));
        assert_noop!(
            CoupleModule::remove_liquidity(Origin::signed(1), 1, number),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(CoupleModule::remove_liquidity(Origin::signed(1), 0, number));

        let remove_liquidity_event = Event::couple(crate::Event::RemoveLiquidity(1, 0, 1, number));
        assert!(System::events()
            .iter()
            .any(|record| record.event == remove_liquidity_event));

        assert_eq!(XPMRLTokens::balance_of(1, 1), Some(86250));
        assert_eq!(XPMRLTokens::balance_of(1, 4), Some(45000));
        assert_eq!(XPMRLTokens::balance_of(2, 3), Some(45000));
    });
}

#[test]
fn test_buy() {
    new_test_ext().execute_with(|| {
        let _ = befor_test();
        assert_noop!(
            CoupleModule::buy(Origin::signed(2), 0, 3, 31250),
            Error::<Test>::ProposalAbnormalState
        );
        assert_noop!(
            CoupleModule::buy(Origin::signed(2), 1, 3, 31250),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::FormalPrediction
        ));
        assert_noop!(
            CoupleModule::buy(Origin::signed(2), 0, 5, 31250),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::buy(Origin::signed(2), 0, 3, 31250));

        let buy_event = Event::couple(crate::Event::Buy(2, 0, 3, 25000));
        assert!(System::events()
            .iter()
            .any(|record| record.event == buy_event));

        assert_eq!(XPMRLTokens::balance_of(2, 3), Some(45000));
        assert_eq!(
            CoupleModule::proposal_total_optional_market(0),
            Some((80000, 125000))
        );
        assert_eq!(CoupleModule::proposal_total_market_fee(0), Some(6250));
        assert_eq!(CoupleModule::proposal_account_info(0, 2), Some(25000));
    });
}

#[test]
fn test_sell() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        assert_noop!(
            CoupleModule::sell(Origin::signed(2), 0, 3, 255),
            Error::<Test>::ProposalAbnormalState
        );
        assert_noop!(
            CoupleModule::sell(Origin::signed(2), 1, 3, 255),
            Error::<Test>::ProposalIdNotExist
        );
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::FormalPrediction
        ));
        assert_noop!(
            CoupleModule::sell(Origin::signed(2), 0, 5, 255),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::buy(Origin::signed(2), 0, 3, 31250));
        assert_ok!(CoupleModule::sell(Origin::signed(2), 0, 3, 45000));

        let sell_event = Event::couple(crate::Event::Sell(2, 0, 3, 20000));
        assert!(System::events()
            .iter()
            .any(|record| record.event == sell_event));

        assert_eq!(XPMRLTokens::balance_of(2, 3), Some(0));
        assert_eq!(
            CoupleModule::proposal_total_optional_market(0),
            Some((number, number))
        );
        assert_eq!(CoupleModule::proposal_total_market_fee(0), Some(11250));
        assert_eq!(CoupleModule::proposal_account_info(0, 2), Some(0));
    });
}

#[test]
fn test_retrieval() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(1), 1),
            Error::<Test>::ProposalIdNotExist
        );
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(1), 0),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::End
        ));
        assert_noop!(
            CoupleModule::retrieval(Origin::signed(1), 0),
            Error::<Test>::ProposalNotResult
        );
        assert_ok!(XPMRLProposals::set_status(
            Origin::root(),
            0,
            ProposalStatus::FormalPrediction
        ));
        assert_ok!(CoupleModule::buy(Origin::signed(2), 0, 3, 31250));
        assert_ok!(CoupleModule::set_result(Origin::root(), 0, 3));
        assert_ok!(CoupleModule::remove_liquidity(Origin::signed(1), 0, number));
        assert_ok!(CoupleModule::retrieval(Origin::signed(1), 0));
        let retrieval_event = Event::couple(crate::Event::Retrieval(1, 0, 3, 0));
        assert!(System::events()
            .iter()
            .any(|record| record.event == retrieval_event));

        assert_ok!(CoupleModule::retrieval(Origin::signed(2), 0));
        let retrieval_event = Event::couple(crate::Event::Retrieval(2, 0, 3, 45000));
        assert!(System::events()
            .iter()
            .any(|record| record.event == retrieval_event));

        assert_eq!(XPMRLTokens::balance_of(1, 1), Some(86250));
        assert_eq!(XPMRLTokens::balance_of(2, 1), Some(45000));
    });
}

#[test]
fn test_set_result() {
    new_test_ext().execute_with(|| {
        let _ = befor_test();
        assert_noop!(
            CoupleModule::set_result(Origin::root(), 0, 5),
            Error::<Test>::CurrencyIdNotFound
        );
        assert_ok!(CoupleModule::set_result(Origin::root(), 0, 3));

        let set_result_event = Event::couple(crate::Event::SetResult(0, 3));
        assert!(System::events()
            .iter()
            .any(|record| record.event == set_result_event));
        assert_eq!(
            XPMRLProposals::proposal_status(0),
            Some(ProposalStatus::End)
        );
    });
}
