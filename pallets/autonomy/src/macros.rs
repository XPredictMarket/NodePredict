#[macro_export]
macro_rules! flag_try_mutate {
    ($change_flag: ident, $proposal_id: ident, $new_value: ident) => {
        storage_try_mutate!(
            $change_flag,
            T,
            $proposal_id,
            |value| -> Result<(), DispatchError> {
                match $new_value{
                    Some(_) => {
                        *value = Some(());
                        Ok(())
                    }
                    None => {
                        *value = None;
                        Ok(())
                    }
                }
            }
        )
    };
}