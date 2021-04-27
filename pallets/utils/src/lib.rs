#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::storage::with_transaction;
use sp_runtime::{DispatchError, TransactionOutcome};

pub fn with_transaction_result<R>(
    f: impl FnOnce() -> Result<R, DispatchError>,
) -> Result<R, DispatchError> {
    with_transaction(|| {
        let res = f();
        if res.is_ok() {
            TransactionOutcome::Commit(res)
        } else {
            TransactionOutcome::Rollback(res)
        }
    })
}

#[macro_export]
macro_rules! runtime_format {
	($($args:tt)*) => {{
		#[cfg(feature = "std")]
		{
			format!($($args)*).as_bytes().to_vec()
		}
		#[cfg(not(feature = "std"))]
		{
			sp_std::alloc::format!($($args)*).as_bytes().to_vec()
		}
	}};
}

#[macro_export]
macro_rules! storage_try_mutate {
    ($storage_name: tt, $config: tt, $($args:tt)*) => {
        $storage_name::<$config>::try_mutate($($args)*)
    };
}

#[macro_export]
macro_rules! sub_abs {
    ($number_1: ident, $number_2: ident) => {
        if $number_1 < $number_2 {
            $number_2.checked_sub(&$number_1).unwrap_or(Zero::zero())
        } else {
            $number_1.checked_sub(&$number_2).unwrap_or(Zero::zero())
        }
    };
}
