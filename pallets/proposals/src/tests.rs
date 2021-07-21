use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok, traits::Time};
use xpmrl_traits::{pool::LiquidityPool, tokens::Tokens, ProposalStatus};

#[test]
fn test_set_status() {
    new_test_ext().execute_with(|| {
        assert_ok!(Couple::new_couple_proposal(1, 1, 10));
        assert_eq!(
            ProposalsModule::proposal_status(0),
            Some(ProposalStatus::OriginalPrediction),
        );
        assert_ok!(ProposalsModule::set_status(
            Origin::root(),
            0,
            ProposalStatus::End
        ));
        assert_noop!(
            ProposalsModule::set_status(Origin::root(), 0, ProposalStatus::End),
            Error::<Test>::ProposalAbnormalState
        );

        assert_ok!(Couple::new_couple_proposal(1, 1, 10));
        assert_noop!(
            ProposalsModule::set_status(Origin::root(), 1, ProposalStatus::WaitingForResults),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(ProposalsModule::set_status(
            Origin::root(),
            1,
            ProposalStatus::FormalPrediction
        ));
        let event = Event::proposals(crate::Event::ProposalStatusChanged(
            1,
            ProposalStatus::FormalPrediction,
        ));
        assert!(System::events().iter().any(|record| record.event == event));
    });
}

#[test]
fn test_stake_to() {
    new_test_ext().execute_with(|| {
        let account = 1;
        let other_account = 2;
        let number = 100;
        assert_ok!(Couple::new_couple_proposal(account, 1, 10));
        let id = 0;
        assert_eq!(ProposalsModule::proposal_owner(id), Some(account));
        assert_eq!(ProposalsModule::current_proposal_id(), Some(id + 1));
        assert_noop!(
            ProposalsModule::stake_to(Origin::signed(account), id, number, true),
            Error::<Test>::OwnerNotAllowedVote
        );
        let before = XPMRLTokens::balance(1, &other_account);
        assert_ok!(ProposalsModule::stake_to(
            Origin::signed(other_account),
            id,
            number,
            true
        ));
        let after = XPMRLTokens::balance(1, &other_account);
        assert_eq!(before - after, number);

        let event = Event::proposals(crate::Event::StakeTo(other_account, id, number));
        assert!(System::events().iter().any(|record| record.event == event));

        assert_noop!(
            ProposalsModule::stake_to(Origin::signed(other_account), id, number, true),
            Error::<Test>::NonRrepeatableStake
        );
    })
}

#[test]
fn test_unstake_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Couple::new_couple_proposal(1, 1, 10));
        let before = XPMRLTokens::balance(1, &2);
        assert_ok!(ProposalsModule::stake_to(Origin::signed(2), 0, 100, true));
        assert_noop!(
            ProposalsModule::unstake_from(Origin::signed(2), 0),
            Error::<Test>::ProposalAbnormalState
        );

        assert_ok!(
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                0,
                ProposalStatus::FormalPrediction
            )
        );
        assert_ok!(ProposalsModule::unstake_from(Origin::signed(2), 0));
        let event = Event::proposals(crate::Event::StakeTo(2, 0, 100));
        assert!(System::events().iter().any(|record| record.event == event));
        let after = XPMRLTokens::balance(1, &2);
        assert_eq!(before, after);
    })
}

#[test]
fn test_deposit_reward() {
    new_test_ext().execute_with(|| {
        let module_account = ProposalsModule::module_account();
        assert_ok!(ProposalsModule::deposit_reward(Origin::signed(1), 100));
        let event = Event::proposals(crate::Event::DepositReward(1, module_account, 100));
        assert!(System::events().iter().any(|record| record.event == event));

        let currency_id = <Test as crate::Config>::GovernanceCurrencyId::get();
        assert_eq!(
            <TokensOf<Test> as Tokens<AccountId>>::balance(currency_id, &module_account),
            100
        );
    })
}

#[test]
fn test_withdrawal_reward() {
    new_test_ext().execute_with(|| {
        assert_ok!(Couple::new_couple_proposal(1, 1, 102));
        assert_noop!(
            ProposalsModule::withdrawal_reward(Origin::signed(1), 0),
            Error::<Test>::ProposalAbnormalState
        );
        assert_ok!(
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                0,
                ProposalStatus::FormalPrediction
            )
        );
        assert_noop!(
            ProposalsModule::withdrawal_reward(Origin::signed(1), 0),
            Error::<Test>::ProposalAbnormalVote
        );
        assert_ok!(
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                0,
                ProposalStatus::OriginalPrediction
            )
        );
        assert_ok!(ProposalsModule::stake_to(Origin::signed(2), 0, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(3), 0, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(4), 0, 600, false));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + 101);
        assert_eq!(
            ProposalsModule::proposal_status(0),
            Some(ProposalStatus::FormalPrediction)
        );
        let default_reward = ProposalsModule::default_reward().unwrap();
        let currency_id = <Test as crate::Config>::GovernanceCurrencyId::get();
        assert_ok!(ProposalsModule::deposit_reward(
            Origin::signed(1),
            default_reward
        ));

        let before = <TokensOf<Test> as Tokens<AccountId>>::balance(currency_id, &2);
        assert_ok!(ProposalsModule::withdrawal_reward(Origin::signed(2), 0));
        let after = <TokensOf<Test> as Tokens<AccountId>>::balance(currency_id, &2);
        assert_eq!(after - before, default_reward / 2);
    })
}

#[test]
fn test_reclaim_reward() {
    new_test_ext().execute_with(|| {
        assert_ok!(Couple::new_couple_proposal(1, 1, 102));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(2), 0, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(3), 0, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(4), 0, 600, false));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + 101);
        let default_reward = ProposalsModule::default_reward().unwrap();
        assert_ok!(ProposalsModule::deposit_reward(
            Origin::signed(1),
            default_reward
        ));
        assert_ok!(ProposalsModule::withdrawal_reward(Origin::signed(2), 0));

        let currency_id = <Test as crate::Config>::GovernanceCurrencyId::get();
        let before = <TokensOf<Test> as Tokens<AccountId>>::balance(currency_id, &1);
        assert_ok!(ProposalsModule::reclaim_reward(Origin::root(), 1));
        let after = <TokensOf<Test> as Tokens<AccountId>>::balance(currency_id, &1);
        assert_eq!(after - before, default_reward / 2);
    })
}

#[test]
fn test_hooks() {
    new_test_ext().execute_with(|| {
        let step: MomentOf<Test> = 100;
        let id = 0;
        let interval_time =
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_minimum_interval_time();

        assert_ok!(Couple::new_couple_proposal(1, 1, step));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + step);
        assert_eq!(
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalStatus::End)
        );

        let now = <Timestamp as Time>::now();
        assert_ok!(Couple::new_couple_proposal(1, 1, now + step));
        let id = 1;
        assert_ok!(ProposalsModule::set_proposal_minimum_interval_time(
            Origin::root(),
            100
        ));
        assert_ok!(
            <ProposalsModule as LiquidityPool<Test>>::set_proposal_state(
                id,
                ProposalStatus::FormalPrediction,
            )
        );
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + step + 1);
        assert_eq!(
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalStatus::WaitingForResults)
        );

        let now = <Timestamp as Time>::now();
        assert_ok!(Couple::new_couple_proposal(1, 1, now + step));
        let id = 2;
        assert_ok!(ProposalsModule::set_proposal_minimum_interval_time(
            Origin::root(),
            step - 1
        ));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + step + 1);
        assert_eq!(
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalStatus::End)
        );

        let now = <Timestamp as Time>::now();
        assert_ok!(Couple::new_couple_proposal(1, 1, now + step * 2));
        let id = 3;
        assert_ok!(ProposalsModule::stake_to(Origin::signed(2), id, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(3), id, 600, true));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(4), id, 600, false));

        assert_eq!(ProposalsModule::proposal_count_vote(id, true), Some(1200));
        assert_eq!(ProposalsModule::proposal_count_vote(id, false), Some(600));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + step + 1);
        assert_eq!(
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalStatus::FormalPrediction)
        );

        let now = <Timestamp as Time>::now();
        assert_ok!(Couple::new_couple_proposal(1, 1, now + step * 2));
        let id = 4;
        assert_ok!(ProposalsModule::stake_to(Origin::signed(2), id, 600, false));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(3), id, 600, false));
        assert_ok!(ProposalsModule::stake_to(Origin::signed(4), id, 600, true));

        assert_eq!(ProposalsModule::proposal_count_vote(id, false), Some(1200));
        assert_eq!(ProposalsModule::proposal_count_vote(id, true), Some(600));
        let now = <Timestamp as Time>::now();
        run_to_block::<ProposalsModule>(now + step + 1);
        assert_eq!(
            <ProposalsModule as LiquidityPool<Test>>::get_proposal_state(id),
            Ok(ProposalStatus::End)
        );

        assert_ok!(ProposalsModule::set_proposal_minimum_interval_time(
            Origin::root(),
            interval_time
        ));
    });
}
