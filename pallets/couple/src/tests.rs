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
    let currency_id = 1;
    let fee_rate = 300;
    let number = 100;
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
    });
}

#[test]
fn test_remove_liquidity() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
    });
}

#[test]
fn test_buy() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
    });
}

#[test]
fn test_sell() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
    });
}

#[test]
fn test_retrieval() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
    });
}
