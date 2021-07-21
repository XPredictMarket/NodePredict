use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};
use xpmrl_traits::RulerModule;

#[test]
fn test_transfer_ruler_address() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ruler::transfer_ruler_address(Origin::signed(1), RulerModule::NotUsed, 1),
            Error::<Test>::NotTransferSelf
        );
        assert_noop!(
            Ruler::transfer_ruler_address(Origin::signed(2), RulerModule::NotUsed, 3),
            Error::<Test>::ModuleNotAllowed
        );
        assert_noop!(
            Ruler::transfer_ruler_address(Origin::signed(2), RulerModule::PlatformDividend, 3),
            Error::<Test>::PermissionDenied
        );
        assert_ok!(Ruler::transfer_ruler_address(
            Origin::signed(1),
            RulerModule::PlatformDividend,
            2
        ));
        assert_eq!(
            Ruler::pending_ruler_address(RulerModule::PlatformDividend),
            Some(2)
        );

        let event = Event::ruler(crate::Event::PendingRulerAddress(
            RulerModule::PlatformDividend,
            1,
            2,
        ));
        assert!(System::events().iter().any(|record| record.event == event));
    })
}

#[test]
fn test_accept_ruler_address() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ruler::accept_ruler_address(Origin::signed(2), RulerModule::PlatformDividend),
            Error::<Test>::PermissionDenied
        );
        assert_ok!(Ruler::transfer_ruler_address(
            Origin::signed(1),
            RulerModule::PlatformDividend,
            2
        ));
        assert_noop!(
            Ruler::accept_ruler_address(Origin::signed(1), RulerModule::PlatformDividend),
            Error::<Test>::ModuleNotAllowed
        );
        assert_ok!(Ruler::accept_ruler_address(
            Origin::signed(2),
            RulerModule::PlatformDividend
        ));
        assert_eq!(
            Ruler::pending_ruler_address(RulerModule::PlatformDividend),
            None
        );
        assert_eq!(Ruler::ruler_address(RulerModule::PlatformDividend), Some(2));
        let event = Event::ruler(crate::Event::AcceptRulerAddress(
            RulerModule::PlatformDividend,
            2,
        ));
        assert!(System::events().iter().any(|record| record.event == event));
    })
}
