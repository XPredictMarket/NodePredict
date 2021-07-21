use crate::RulerModule;
use frame_support::dispatch::DispatchError;

pub trait RulerAccounts<T>
where
    T: frame_system::Config,
{
    fn get_account(module: RulerModule) -> Result<T::AccountId, DispatchError>;
}
