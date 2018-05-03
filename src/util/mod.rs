pub mod rtt;

pub fn no_err<T>(value: T) -> Result<T, !> {
    Ok(value)
}
