use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use std::time::SystemTime;
use xpmrl_traits::ProposalStatus;

#[test]
fn test_new_proposal() {
    new_test_ext().execute_with(|| {
        assert_eq!(ProposalsModule::current_proposal_id(), None);
        let proposal = Proposal {
            title: "how to test this module".as_bytes().to_vec(),
            category_id: 1,
            detail: "proposal detail".as_bytes().to_vec(),
        };
        if let Ok(ts) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let now = ts.as_secs() as u32 + 1_000_000;
            assert_ok!(ProposalsModule::new_proposal(
                Origin::signed(1),
                proposal.title.clone(),
                [
                    "the one".as_bytes().to_vec(),
                    "other one".as_bytes().to_vec()
                ],
                now,
                proposal.category_id,
                1,
                100,
                200,
                proposal.detail.clone()
            ));
            let new_proposal_event = Event::proposals(crate::Event::NewProposal(1, 0, 1));
            assert!(System::events()
                .iter()
                .any(|record| record.event == new_proposal_event));
            assert_eq!(ProposalsModule::current_proposal_id(), Some(1));
            assert_eq!(
                ProposalsModule::proposal_status(0),
                Some(ProposalStatus::OriginalPrediction)
            );
            assert_eq!(ProposalsModule::proposal_owner(0), Some(1));
        }
    });
}

#[test]
fn test_set_status() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ProposalsModule::set_status(Origin::root(), 0, ProposalStatus::FormalPrediction),
            Error::<Test>::ProposalIdNotExist
        );
        let proposal = Proposal {
            title: "how to test this module".as_bytes().to_vec(),
            category_id: 1,
            detail: "proposal detail".as_bytes().to_vec(),
        };
        if let Ok(ts) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let now = ts.as_secs() as u32 + 1_000_000;
            assert_ok!(ProposalsModule::new_proposal(
                Origin::signed(1),
                proposal.title.clone(),
                [
                    "the one".as_bytes().to_vec(),
                    "other one".as_bytes().to_vec()
                ],
                now,
                proposal.category_id,
                1,
                100,
                200,
                proposal.detail.clone()
            ));
            assert_ok!(ProposalsModule::set_status(
                Origin::root(),
                0,
                ProposalStatus::FormalPrediction
            ));
            let proposal_status_changed_event = Event::proposals(crate::Event::ProposalStatusChanged(
                0,
                ProposalStatus::FormalPrediction,
            ));
            assert!(System::events()
                .iter()
                .any(|record| record.event == proposal_status_changed_event));
            assert_eq!(
                ProposalsModule::proposal_status(0),
                Some(ProposalStatus::FormalPrediction)
            );
        }
    });
}
