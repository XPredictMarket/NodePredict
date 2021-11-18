use crate::{mock::*, Error, Payload};

use frame_support::{assert_noop, assert_ok};
use sp_std::collections::btree_map::BTreeMap;
use xpmrl_traits::{couple::LiquidityCouple, pool::LiquidityPool, ProposalStatus, tokens::Tokens,};

#[test]
fn test_set_minimal_number() {
    new_test_ext(|_| {
        let minimal_stake_number: BalanceOf<Test> = 10000;
        assert_ok!(AutonomyModule::set_minimal_stake_number(Origin::root(), minimal_stake_number));
        let event = Event::autonomy(crate::Event::SetMinimalStakeNumber(minimal_stake_number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::minimal_stake_number(), Some(minimal_stake_number));
        let minimal_review_number: BalanceOf<Test> = 1000;
        assert_ok!(AutonomyModule::set_minimal_review_number(Origin::root(), minimal_review_number));
        let event = Event::autonomy(crate::Event::SetMinimalReviewNumber(minimal_review_number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::minimal_review_number(), Some(minimal_review_number));
        let minimal_report_number: BalanceOf<Test> = 100000;
        assert_ok!(AutonomyModule::set_minimal_report_number(Origin::root(), minimal_report_number));
        let event = Event::autonomy(crate::Event::SetMinimalReportNumber(minimal_report_number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::minimal_report_number(), Some(minimal_report_number));

        let lock_ratio: BalanceOf<Test> = 100;
        let lock_ratio1: BalanceOf<Test> = 10;
        assert_noop!(
            AutonomyModule::set_lock_ratio(Origin::root(), lock_ratio),
            Error::<Test>::InputRatioIsTooLarge
        );
        assert_ok!(AutonomyModule::set_lock_ratio(Origin::root(), lock_ratio1));
        let event = Event::autonomy(crate::Event::SetLockRatio(lock_ratio1));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::lock_ratio(), Some(lock_ratio1));
    })
}

#[test]
fn test_set_time_period() {
    new_test_ext(|_| {
        let review_cycle: MomentOf<Test> = 100;
        assert_ok!(AutonomyModule::set_review_cycle(
            Origin::root(),
            review_cycle
        ));
        let event = Event::autonomy(crate::Event::SetReviewCycle(review_cycle));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::review_cycle(), Some(review_cycle));
        let upload_cycle: MomentOf<Test> = 100;
        assert_ok!(AutonomyModule::set_upload_cycle(
            Origin::root(),
            upload_cycle
        ));
        let event = Event::autonomy(crate::Event::SetUploadCycle(upload_cycle));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::result_upload_cycle(), Some(upload_cycle));
        let publicity_period: MomentOf<Test> = 100;
        assert_ok!(AutonomyModule::set_publicity_period(
            Origin::root(),
            publicity_period
        ));
        let event = Event::autonomy(crate::Event::SetPublicityPeriod(publicity_period));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::publicity_period(), Some(publicity_period));
    })
}

#[test]
fn test_stake() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let now = System::block_number();
        let stake_number1: BalanceOf<Test> = 100;
        assert_eq!(AutonomyModule::staked_node(*account), None);
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number1));
        let event = Event::autonomy(crate::Event::Stake(*account, stake_number1));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::staked_node(*account), Some((stake_number1, false)));
        let snapshot_number = AutonomyModule::snap_shot_num(*account).unwrap();
        assert_eq!(AutonomyModule::snap_shot(*account, snapshot_number), Some((now , 100)));
        let stake_number2: BalanceOf<Test> = 900;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number2));
        let event = Event::autonomy(crate::Event::Stake(*account, stake_number2));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::staked_node(*account), Some((stake_number1 + stake_number2, true)));
        assert_eq!(AutonomyModule::snap_shot(*account, snapshot_number + 1), Some((now , 1000)));
    })
}

#[test]
fn test_unstake() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let now = System::block_number();
        let stake_number: BalanceOf<Test> = 2000;
        let unstake_number: BalanceOf<Test> = 600;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account),stake_number));
        let snapshot_number = AutonomyModule::snap_shot_num(*account).unwrap();
        assert_eq!(AutonomyModule::snap_shot(*account, snapshot_number), Some((now , 2000)));
        assert_ok!(AutonomyModule::unstake(Origin::signed(*account), unstake_number));
        let event = Event::autonomy(crate::Event::UnStake(*account, unstake_number));
        assert_eq!(AutonomyModule::snap_shot(*account, snapshot_number + 1), Some((now , 1400)));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::staked_node(*account), 
            Some((stake_number - unstake_number, true)));
        assert_ok!(AutonomyModule::unstake(Origin::signed(*account), unstake_number));
        assert_eq!(AutonomyModule::snap_shot(*account, snapshot_number + 2), Some((now , 800)));
        let event = Event::autonomy(crate::Event::UnStake(*account, unstake_number));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(AutonomyModule::staked_node(*account), 
            Some((stake_number - unstake_number - unstake_number, false)));
    })
}

#[test]
fn test_review() {
    new_test_ext(|public_key_array| {
        let review_time = 5;
        let account = public_key_array.get(0).unwrap();
        let other = public_key_array.get(1).unwrap();
        let other1 = public_key_array.get(2).unwrap();
        assert_ok!(Proposals::new_couple_proposal(*account, 1));
        
        let now = System::block_number();
        assert_ok!(Proposals::set_create_time(0, now));
        assert_ok!(Proposals::set_close_time(0, 1000));
        let stake_number: BalanceOf<Test> = 1000;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other1), stake_number));
        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::FormalPrediction,
        ));
        assert_noop!(
            AutonomyModule::review(Origin::signed(*other), 30, 0, true),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::OriginalPrediction,
        ));
        assert_noop!(
            AutonomyModule::review(Origin::signed(*other), 0, 0, true),
            Error::<Test>::ReviewStakedNumberZero
        );
        assert_ok!( AutonomyModule::review(Origin::signed(*other), 30, 0, true) );
        assert_noop!(
            AutonomyModule::review(Origin::signed(*other), 30, 0, true),
            Error::<Test>::NodeHasAlreadyReview
        );
        let mut map = BTreeMap::new();
        map.insert(true, 30);
        assert_eq!(AutonomyModule::node_review_voting_status(0, *other), Some(map));
        assert_eq!(AutonomyModule::review_voting_status(0, true), Some(30));
        assert_eq!(AutonomyModule::review_flag(0), None);
        assert_ok!( AutonomyModule::review(Origin::signed(*other1), 70, 0, true) );
        let mut map = BTreeMap::new();
        map.insert(true, 70);
        assert_eq!(AutonomyModule::node_review_voting_status(0, *other1), Some(map));
        assert_eq!(AutonomyModule::review_voting_status(0, true), Some(100));
        assert_eq!(AutonomyModule::review_flag(0), Some(()));
        run_to_block::<AutonomyModule>(now + review_time - 1);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::OriginalPrediction));
        run_to_block::<AutonomyModule>(now + review_time + 1);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::FormalPrediction));

        assert_ok!(Proposals::new_couple_proposal(*account, 1));
        let now = System::block_number();
        assert_ok!(Proposals::set_create_time(1, now));
        assert_ok!(Proposals::set_close_time(1, 1000));
        assert_ok!( AutonomyModule::review(Origin::signed(*other), 200, 1, false) );
        assert_ok!( AutonomyModule::review(Origin::signed(*other1), 150, 1, true) );
        run_to_block::<AutonomyModule>(now + review_time - 1);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(1),
            Ok(ProposalStatus::OriginalPrediction));
        run_to_block::<AutonomyModule>(now + review_time + 1);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(1),
            Ok(ProposalStatus::End));

        assert_ok!(Proposals::new_couple_proposal(*account, 2));
        let now = System::block_number();
        assert_ok!(Proposals::set_create_time(2, now));
        assert_ok!(Proposals::set_close_time(2, 1000));
        assert_ok!( AutonomyModule::review(Origin::signed(*other), 200, 2, false) );
        assert_ok!( AutonomyModule::review(Origin::signed(*other1), 200, 2, true) );
        run_to_block::<AutonomyModule>(now + review_time - 1);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(2),
            Ok(ProposalStatus::OriginalPrediction));
        assert_eq!(AutonomyModule::review_delay(2), None);
        run_to_block::<AutonomyModule>(now + review_time);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(2),
            Ok(ProposalStatus::OriginalPrediction));    
        assert_eq!(AutonomyModule::review_delay(2), Some(1));
        assert_ok!( AutonomyModule::review(Origin::signed(*account), 200, 2, true) );
        run_to_block::<AutonomyModule>(now + review_time + review_time);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(2),
            Ok(ProposalStatus::FormalPrediction));    
    })
}

#[test]
fn test_upload_result() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let other = public_key_array.get(1).unwrap();
        let other1 = public_key_array.get(2).unwrap();
        let upload_cycle: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_upload_cycle(
            Origin::root(),
            upload_cycle
        ));
        let mut payload = Payload {
            proposal_id: 0,
            result: 3,
            public: *account,
            vote_num: 100
        };
        let payload1 = Payload {
            proposal_id: 0,
            result: 5,
            public: *other,
            vote_num: 100
        };
        let payload2 = Payload {
            proposal_id: 0,
            result: 5,
            public: *other1,
            vote_num: 100
        };
        assert_ok!(Proposals::new_couple_proposal(*account, 1));
        let now = System::block_number();
        let close_time: MomentOf<Test> = 10;
        assert_ok!(Proposals::set_create_time(0, now));
        assert_ok!(Proposals::set_close_time(0, close_time));
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
        let stake_number: BalanceOf<Test> = 2000;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other1), stake_number));
        assert_eq!(AutonomyModule::staked_node_lock_total_num(*account), None);
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::ProposalOptionNotCorrect
        );
        run_to_block::<AutonomyModule>(close_time - 1);
        assert_eq!(AutonomyModule::upload_delay(0), None);
        payload.result = 4;
        assert_eq!(AutonomyModule::node_result_voting_status(0, *account), None);
        assert_eq!(AutonomyModule::result_voting_status(0, 4), None);
        let lock_ratio: BalanceOf<Test> = 10;
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload.clone(), Default::default()),
            Error::<Test>::LockRatioNotSet
        );
        assert_ok!(AutonomyModule::set_lock_ratio(Origin::root(), lock_ratio) );
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload.clone(),
            Default::default()
        ));
        assert_eq!(AutonomyModule::staked_node_lock_total_num(*account), Some(10));
        assert_eq!(AutonomyModule::staked_node_lock_num(0, *account), Some(10));
        let event = Event::autonomy(crate::Event::UploadResult(*account, 0, 4, 100));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_noop!(
            AutonomyModule::upload_result(Origin::none(), payload, Default::default()),
            Error::<Test>::AccountHasAlreadyUploaded
        );
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload1.clone(),
            Default::default()
        ));
        
        assert_eq!(AutonomyModule::node_result_voting_status(0, *account), Some((4, 100)));
        assert_eq!(AutonomyModule::result_voting_status(0, 4), Some(100));
        assert_eq!(AutonomyModule::node_result_voting_status(0, *other), Some((5, 100)));
        assert_eq!(AutonomyModule::result_voting_status(0, 5), Some(100));
        run_to_block::<AutonomyModule>(close_time + upload_cycle + 5);
        assert_eq!(AutonomyModule::upload_delay(0), Some(1));
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::WaitingForResults));
        
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload2.clone(),
            Default::default()
        ));
        run_to_block::<AutonomyModule>(close_time + upload_cycle + upload_cycle);
        assert_eq!(<Proposals as LiquidityCouple<Test>>::get_proposal_result(0),
            Ok(5)
        );
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::ResultAnnouncement));
    })
}

#[test]
fn test_report() {
    new_test_ext(|public_key_array| {
        let account = public_key_array.get(0).unwrap();
        let other = public_key_array.get(1).unwrap();
        let other2 = public_key_array.get(2).unwrap();
        let other3 = public_key_array.get(3).unwrap();
        assert_ok!(Proposals::new_couple_proposal(*account, 1));
        let publicity_period: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_publicity_period(
            Origin::root(),
            publicity_period
        ));
        let now = System::block_number();
        let close_time: MomentOf<Test> = 10;
        assert_ok!(Proposals::set_create_time(0, now));
        assert_ok!(Proposals::set_close_time(0, close_time));
        let upload_cycle: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_upload_cycle(
            Origin::root(),
            upload_cycle
        ));
        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::WaitingForResults,
        ));
        let stake_number: BalanceOf<Test> = 1000;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other2), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other3), stake_number));
        let payload = Payload {
            proposal_id: 0,
            result: 4,
            public: *account,
            vote_num: 500,
        };
        let payload2 = Payload {
            proposal_id: 0,
            result: 5,
            public: *other,
            vote_num: 300,
        };
        let lock_ratio: BalanceOf<Test> = 10;
        assert_ok!(AutonomyModule::set_lock_ratio(Origin::root(), lock_ratio) );
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload,
            Default::default()
        ));
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload2,
            Default::default()
        ));
        assert_noop!(
            AutonomyModule::report(Origin::signed(*other), 0, 10),
            Error::<Test>::ProposalAbnormalState
        );
        run_to_block::<AutonomyModule>(close_time + upload_cycle);
        assert_eq!(<Proposals as LiquidityCouple<Test>>::get_proposal_result(0),
            Ok(4),
        );
        assert_noop!(AutonomyModule::report(Origin::signed(*other), 0, 0),
            Error::<Test>::ReportStakedNumberZero
        );
        let minimal_report_number: BalanceOf<Test> = 10000;
        assert_ok!(AutonomyModule::set_minimal_report_number(Origin::root(), minimal_report_number));
        assert_ok!(AutonomyModule::report(Origin::signed(*other), 0, 5000));
        assert_noop!(AutonomyModule::report(Origin::signed(*other), 0, 5000),
            Error::<Test>::AccountHasAlreadyReport
        );
        let event = Event::autonomy(crate::Event::Report(*other, 0, 5000));
        assert!(System::events().iter().any(|record| record.event == event));
        assert_eq!(
            AutonomyModule::account_report_number(0, *other),
            Some(5000)
        );
        assert_eq!(
            AutonomyModule::report_success_flag(0),
            None
        );
        assert_eq!(
            AutonomyModule::report_voting_status(0),
            Some(5000)
        );
        assert_ok!(
            AutonomyModule::report(Origin::signed(*other2), 0, 6000)
        );
        assert_eq!(
            AutonomyModule::account_report_number(0, *other2),
            Some(6000)
        );
        assert_eq!(
            AutonomyModule::report_success_flag(0),
            Some(())
        );
        assert_eq!(
            AutonomyModule::report_voting_status(0),
            Some(11000)
        );
        run_to_block::<AutonomyModule>(close_time + upload_cycle + publicity_period);
        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::End));
    })
}

#[test]
fn test_slash_and_take_out_unlock() {
    new_test_ext(|public_key_array| {
        let module_account = AutonomyModule::module_account();
        let account = public_key_array.get(0).unwrap();
        let other = public_key_array.get(1).unwrap();
        let other2 = public_key_array.get(2).unwrap();
        let other3 = public_key_array.get(3).unwrap();
        assert_ok!(Proposals::new_couple_proposal(*account, 1));
        let publicity_period: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_publicity_period(
            Origin::root(),
            publicity_period
        ));
        let upload_cycle: MomentOf<Test> = 10;
        assert_ok!(AutonomyModule::set_upload_cycle(
            Origin::root(),
            upload_cycle
        ));
        let now = System::block_number();
        let close_time: MomentOf<Test> = 10;
        assert_ok!(Proposals::set_create_time(0, now));
        assert_ok!(Proposals::set_close_time(0, close_time));
        assert_ok!(<Proposals as LiquidityPool<Test>>::set_proposal_state(
            0,
            ProposalStatus::WaitingForResults,
        ));
        let minimal_report_number: BalanceOf<Test> = 10000;
        assert_ok!(AutonomyModule::set_minimal_report_number(Origin::root(), minimal_report_number));
        let stake_number: BalanceOf<Test> = 1000;
        assert_ok!(AutonomyModule::stake(Origin::signed(*account), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other2), stake_number));
        assert_ok!(AutonomyModule::stake(Origin::signed(*other3), stake_number));
        let payload = Payload {
            proposal_id: 0,
            result: 4,
            public: *account,
            vote_num: 500,
        };
        let payload2 = Payload {
            proposal_id: 0,
            result: 5,
            public: *other,
            vote_num: 300,
        };
        let payload3 = Payload {
            proposal_id: 0,
            result: 4,
            public: *other2,
            vote_num: 1000,
        };
        let lock_ratio: BalanceOf<Test> = 10;
        assert_ok!(AutonomyModule::set_lock_ratio(Origin::root(), lock_ratio) );
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload,
            Default::default()
        ));
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload2,
            Default::default()
        ));
        assert_ok!(AutonomyModule::upload_result(
            Origin::none(),
            payload3,
            Default::default()  
        ));
        run_to_block::<AutonomyModule>(close_time + upload_cycle);
        let minimal_report_number: BalanceOf<Test> = 10000;
        assert_ok!(AutonomyModule::set_minimal_report_number(Origin::root(), minimal_report_number));

        assert_ok!(AutonomyModule::report(Origin::signed(*other), 0, 5000));
        assert_ok!(AutonomyModule::report(Origin::signed(*other3), 0, 10000));
        assert_eq!(AutonomyModule::report_success_flag(0), Some(()));
        run_to_block::<AutonomyModule>(close_time + publicity_period + upload_cycle);

        assert_eq!(<Proposals as LiquidityPool<Test>>::get_proposal_state(0),
            Ok(ProposalStatus::End));
        assert_eq!(AutonomyModule::report_asset_pool(0), None);
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, &module_account), 0);
        assert_eq!(AutonomyModule::account_slash_number(0, *account), None);
        assert_ok!(AutonomyModule::slash(Origin::root(), *account, 0));
        assert_eq!(AutonomyModule::report_asset_pool(0), Some(50));
        assert_eq!(AutonomyModule::account_slash_number(0, *account), Some(50));
        assert_eq!(AutonomyModule::slash_finish_flag(0), None);
        assert_ok!(AutonomyModule::slash(Origin::root(), *other2, 0));
        assert_eq!(AutonomyModule::report_asset_pool(0), Some(150));
        assert_eq!(AutonomyModule::account_slash_number(0, *other2), Some(100));
        assert_ok!(AutonomyModule::slash_finish(Origin::root(), 0));
        assert_eq!(AutonomyModule::slash_finish_flag(0), Some(()));
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, &module_account), 150);
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, other), 94000);
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, other3), 89000);
        assert_ok!(AutonomyModule::take_out(Origin::signed(*other), 0));
        assert_ok!(AutonomyModule::take_out(Origin::signed(*other3), 0));
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, &module_account), 2);
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, other), 99049);
        assert_eq!(<TokensOf<Test> as Tokens<AccountId>>::balance(1, other3), 99099);
        assert_noop!(AutonomyModule::unlock(Origin::signed(*account), 0),
            Error::<Test>::UploadResultWasReported
        );
    })
}

