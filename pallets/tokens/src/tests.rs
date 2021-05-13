use crate::pallet::PRC20;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_new_asset() {
    new_test_ext().execute_with(|| {
        // native asset is none
        assert_eq!(TokensModule::currencies(0), None);
        assert_eq!(TokensModule::current_currency_id(), Some(1));

        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };

        // new a asset
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        let new_asset_event = Event::tokens(crate::Event::NewAsset(1));
        assert!(System::events()
            .iter()
            .any(|record| record.event == new_asset_event));
        assert_eq!(TokensModule::currencies(0), None);
        // this is new asset, will equal last one
        assert_eq!(TokensModule::currencies(1), Some(asset));
        // 2 asset is none
        assert_eq!(TokensModule::currencies(2), None);
        assert_eq!(TokensModule::current_currency_id(), Some(2));
    });
}

#[test]
fn test_mint() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TokensModule::mint(Origin::root(), 1, 1, 100),
            Error::<Test>::CurrencyIdNotExist
        );
        assert_ok!(TokensModule::mint(Origin::root(), 0, 1, 100));
        assert_eq!(PalletBalances::free_balance(1), 200);

        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 100));
        let mint_event = Event::tokens(crate::Event::Mint(1, 1, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == mint_event));
        assert_eq!(TokensModule::total_supply(1), Some(100));
        assert_eq!(TokensModule::free_balance_of(1, 1), Some(100));

        assert_ok!(TokensModule::mint(Origin::root(), 1, 2, 100));
        let mint_event = Event::tokens(crate::Event::Mint(1, 2, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == mint_event));
        assert_eq!(TokensModule::total_supply(1), Some(200));
        assert_eq!(TokensModule::free_balance_of(2, 1), Some(100));
    });
}

#[test]
fn test_burn() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TokensModule::burn(Origin::signed(1), 1, 100),
            Error::<Test>::CurrencyIdNotExist
        );
        assert_ok!(TokensModule::burn(Origin::signed(1), 0, 100));
        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_noop!(
            TokensModule::burn(Origin::signed(1), 1, 100),
            Error::<Test>::CurrencyIdNotExist
        );
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 100));
        assert_ok!(TokensModule::burn(Origin::signed(1), 1, 100));
        let burn_event = Event::tokens(crate::Event::Burn(1, 1, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == burn_event));
        assert_eq!(TokensModule::total_supply(1), Some(0));
        assert_eq!(TokensModule::free_balance_of(1, 1), Some(0));

        assert_ok!(TokensModule::burn(Origin::signed(1), 1, 100));
        let burn_event = Event::tokens(crate::Event::Burn(1, 1, 0));
        assert!(System::events()
            .iter()
            .any(|record| record.event == burn_event));
    });
}

#[test]
fn test_transfer() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokensModule::inner_balance_of(0, &1), 100);
        assert_ok!(TokensModule::transfer(Origin::signed(1), 0, 3, 50));
        assert_eq!(TokensModule::inner_balance_of(0, &3), 50);
        assert_ok!(TokensModule::transfer(Origin::signed(3), 0, 1, 50));
        assert_eq!(TokensModule::inner_balance_of(0, &1), 100);

        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 100));
        assert_noop!(
            TokensModule::transfer(Origin::signed(1), 1, 1, 100),
            Error::<Test>::TransferFromSelf
        );
        assert_noop!(
            TokensModule::transfer(Origin::signed(1), 1, 2, 200),
            Error::<Test>::InsufficientBalance
        );
        assert_ok!(TokensModule::transfer(Origin::signed(1), 1, 2, 50));
        let transfer_event = Event::tokens(crate::Event::Transfer(1, 1, 2, 50));
        assert!(System::events()
            .iter()
            .any(|record| record.event == transfer_event));

        assert_eq!(TokensModule::inner_balance_of(1, &1), 50);
        assert_eq!(TokensModule::inner_balance_of(1, &2), 50);
        assert_eq!(TokensModule::total_supply(1), Some(100));
    });
}

#[test]
fn test_approve() {
    new_test_ext().execute_with(|| {
        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 200));

        assert_noop!(
            TokensModule::approve(Origin::signed(1), 1, 1, 50),
            Error::<Test>::ApproveSelf
        );
        assert_ok!(TokensModule::approve(Origin::signed(1), 1, 2, 100));
        let approval_event = Event::tokens(crate::Event::Approval(1, 1, 2, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == approval_event));

        let allowed = TokensModule::allowance(1, 1);
        assert_ne!(allowed, None);
        assert_eq!(allowed.unwrap().get(&2u64), Some(&100u128));
    });
}

#[test]
fn test_burn_from() {
    new_test_ext().execute_with(|| {
        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 200));
        assert_ok!(TokensModule::approve(Origin::signed(1), 1, 2, 100));

        assert_noop!(
            TokensModule::burn_from(Origin::signed(1), 1, 1, 200),
            Error::<Test>::BurnFromSelf
        );
        assert_noop!(
            TokensModule::burn_from(Origin::signed(2), 1, 1, 200),
            Error::<Test>::OriginNotAllowed
        );
        assert_ok!(TokensModule::burn_from(Origin::signed(2), 1, 1, 100));
        let approval_event = Event::tokens(crate::Event::Approval(1, 1, 2, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == approval_event));
        assert_eq!(TokensModule::total_supply(1), Some(100));
        assert_eq!(TokensModule::free_balance_of(1, 1), Some(100));
    });
}

#[test]
fn test_transfer_from() {
    new_test_ext().execute_with(|| {
        let asset = PRC20 {
            name: "Tether USD".as_bytes().to_vec(),
            symbol: "USD".as_bytes().to_vec(),
            decimals: 6,
        };
        assert_ok!(TokensModule::new_asset(
            Origin::root(),
            asset.name.clone(),
            asset.symbol.clone(),
            asset.decimals
        ));
        assert_ok!(TokensModule::mint(Origin::root(), 1, 1, 200));
        assert_ok!(TokensModule::approve(Origin::signed(1), 1, 2, 100));

        assert_noop!(
            TokensModule::transfer_from(Origin::signed(2), 1, 2, 1, 100),
            Error::<Test>::TransferFromSelf
        );

        assert_noop!(
            TokensModule::transfer_from(Origin::signed(2), 1, 1, 1, 100),
            Error::<Test>::TransferFromSelf
        );
        assert_noop!(
            TokensModule::transfer_from(Origin::signed(2), 1, 1, 2, 200),
            Error::<Test>::OriginNotAllowed
        );
        assert_ok!(TokensModule::transfer_from(Origin::signed(2), 1, 1, 2, 100));
        let transfer_event = Event::tokens(crate::Event::TransferFrom(1, 2, 1, 2, 100));
        assert!(System::events()
            .iter()
            .any(|record| record.event == transfer_event));
    });
}
