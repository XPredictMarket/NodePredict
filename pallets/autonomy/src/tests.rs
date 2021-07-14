use crate::{mock::*, Error, Payload};

use frame_support::{assert_noop, assert_ok};
use xpmrl_traits::{couple::LiquidityCouple, pool::LiquidityPool, ProposalStatus};

#[test]
fn test_set_minimal_number() {
    new_test_ext(|_| {
        let number: BalanceOf<Test> = 200;
        assert_ok!(AutonomyModule::set_minimal_number(Origin::root(), number));
        let event = Event::autonomy(crate::Event::SetMinimalNumber(number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::minimal_stake_number(), Some(number));
    })
}

#[test]
fn test_set_publicity_interval() {
    new_test_ext(|_| {
        let interval: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_publicity_interval(
            Origin::root(),
            interval
        ));
        let event = Event::autonomy(crate::Event::SetPublicityInterval(interval));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::publicity_interval(), Some(interval));
    })
}

#[test]
fn test_stake() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let number = AutonomyModule::minimal_stake_number().unwrap();

        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        let event = Event::autonomy(crate::Event::Stake(*account, number));
        assert!(System::events().iter().any(|record| record.event == event));

        assert_eq!(AutonomyModule::staked_account(*account), Some(number));
    })
}

#[test]
fn test_tagging() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        assert_noop!(
            AutonomyModule::tagging(Origin::root(), *account),
            Error::<Test>::AccountNotStaked
        );
        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
        let event = Event::autonomy(crate::Event::Tagging(*account));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_noop!(
            AutonomyModule::tagging(Origin::root(), *account),
            Error::<Test>::AccountHasTagged
        );
        assert_eq!(AutonomyModule::autonomy_account(*account), Some(()));
    })
}

#[test]
fn test_unstake() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let number = AutonomyModule::minimal_stake_number().unwrap();

        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::unstake(Origin::signed(*account)));
        let event = Event::autonomy(crate::Event::UnStake(*account, number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::staked_account(*account), None);

        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
        assert_ok!(AutonomyModule::unstake(Origin::signed(*account)));
        assert_eq!(AutonomyModule::autonomy_account(*account), None);
        assert_eq!(AutonomyModule::staked_account(*account), None);
    })
}

#[test]
fn test_untagging() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
        assert_ok!(AutonomyModule::untagging(Origin::root(), *account));
        let event = Event::autonomy(crate::Event::Untagging(*account));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::autonomy_account(*account), None);
    })
}

#[test]
fn test_slash() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let number = AutonomyModule::minimal_stake_number().unwrap();

        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
        assert_ok!(AutonomyModule::slash(Origin::root(), *account));
        let event = Event::autonomy(crate::Event::Slash(*account, number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::autonomy_account(*account), None);
        assert_eq!(AutonomyModule::staked_account(*account), None);
    })
}

#[test]
fn test_upload_result() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let mut payload = Payload {
            proposal_id: 0,
            result: 3,
            public: account.clone(),
        };
        let xx = "".as_bytes().to_vec();
        assert_ok!(<Proposals as LiquidityCouple<Test>>::new_couple_proposal(
            Origin::signed(*account),
            xx.clone(),
            [xx.clone(), xx.clone()],
            0,
            0,
            1,
            0,
            0,
            xx,
        ));
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::ProposalAbnormalState
        );

        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::WaitingForResults,
        ));
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::AccountNotStaked
        );

        assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
        assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::ProposalOptionNotCorrect
        );

        payload.result = 4;

        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload.clone(),
            Default::default()
        ));
        let event = Event::autonomy(crate::Event::UploadResult(*account, 0, 4));
        assert!(System::events().iter().any(|record| record.event == event));

        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::AccountHasAlreadyUploaded
        );
    })
}

#[test]
fn test_auto_merged_result() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        // TODO test begin_block function
    })
}
