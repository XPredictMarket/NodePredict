use crate::{mock::*, MineInfo, Point};
use frame_support::assert_ok;

fn befor_test() -> BalanceOf<Test> {
    let numebr: BalanceOf<Test> = 100000000;
    assert_ok!(Proposals::new_couple_proposal(1, 1, numebr));
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
    numebr
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
