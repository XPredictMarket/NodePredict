use std::time::SystemTime;

use crate::{mock::*, MineInfo, Point};
use frame_support::assert_ok;
use xpmrl_couple::Proposal;

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
    let fee_rate: u32 = 200;
    let number = 100000000;
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
    let proposal_mine = MineInfo {
        perblock: 1000000,
        from: 200,
        to: 300,
    };
    assert_ok!(MiningModule::proposal_mine(
        Origin::root(),
        0,
        proposal_mine.perblock,
        proposal_mine.from,
        proposal_mine.to
    ));
    assert_eq!(MiningModule::proposal_mine_info(0), Some(proposal_mine));
    number
}

#[test]
fn test_stake() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        assert_ok!(MiningModule::stake(Origin::signed(1), 0, number));
        assert_eq!(
            MiningModule::proposal_checkpoint(0),
            Some(vec![Point { from: 1, number }])
        );
    });
}

#[test]
fn test_unstake() {
    new_test_ext().execute_with(|| {
        let number = befor_test();
        assert_ok!(MiningModule::stake(Origin::signed(1), 0, number));
        assert_ok!(MiningModule::unstake(Origin::signed(1), 0, 1000000));
        assert_ok!(MiningModule::unstake(
            Origin::signed(1),
            0,
            number - 1000000
        ));
    });
}

#[test]
fn test_proposal_mine() {
    new_test_ext().execute_with(|| {
        befor_test();
    });
}
