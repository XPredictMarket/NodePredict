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
        assert_ok!(Proposals::new_couple_proposal(*account, 1));
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

        assert_eq!(AutonomyModule::temporary_results(0, *account), Some(4));
        assert_eq!(AutonomyModule::statistical_results(0, 4), Some(1));

        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::AccountHasAlreadyUploaded
        );
    })
}

#[test]
fn test_auto_merged_result() {
    new_test_ext(|public_key_array| {
        let interval: MomentOf<Test> = 5;
        assert_ok!(AutonomyModule::set_publicity_interval(
            Origin::root(),
            interval
        ));
        let own = public_key_array.get(0).unwrap();
        assert_ok!(Proposals::new_couple_proposal(*own, 1));
        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::WaitingForResults,
        ));

        let with_index = |index: usize, result: CurrencyIdOf<Test>| -> AccountId {
            let account = public_key_array.get(index).unwrap();
            assert_ok!(AutonomyModule::stake(Origin::signed(*account)));
            assert_ok!(AutonomyModule::tagging(Origin::root(), *account));
            let payload = Payload {
                proposal_id: 0,
                result,
                public: account.clone(),
            };
            assert_ok!(AutonomyModule::upload_result(
                Origin::none(),
                payload,
                Default::default()
            ),);
            assert_eq!(AutonomyModule::temporary_results(0, *account), Some(result));
            *account
        };

        let _node_1 = with_index(1, 4);
        let _node_2 = with_index(2, 4);
        let _node_3 = with_index(3, 5);
        assert_eq!(AutonomyModule::statistical_results(0, 4), Some(2));
        assert_eq!(AutonomyModule::statistical_results(0, 5), Some(1));

        let now = System::block_number();
        assert_ok!(Proposals::set_announcement_time(0, now));
        run_to_block::<AutonomyModule>(now + interval + 1);
        assert_eq!(
            <Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::ResultAnnouncement)
        );
        let now = System::block_number();
        assert_eq!(AutonomyModule::proposal_announcement(0), Some(now));

        run_to_block::<AutonomyModule>(now + interval + 1);
        assert_eq!(
            <Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::End)
        );
        assert_eq!(
            <Proposals as LiquidityCouple<Test>>::get_proposal_result(0),
            Ok(4)
        );
    })
}
