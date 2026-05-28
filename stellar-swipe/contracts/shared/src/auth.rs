//! Cross-contract call depth limit (Issue #433).
//!
//! Deep cross-contract call chains can exhaust the instruction budget or create
//! unexpected execution paths. This module provides a guard that returns
//! `CallDepthExceeded` when the call depth exceeds `MAX_CALL_DEPTH`.

use soroban_sdk::contracterror;

/// Maximum allowed cross-contract call depth.
pub const MAX_CALL_DEPTH: u32 = 5;

/// Error returned when the call depth limit is exceeded.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CallDepthError {
    CallDepthExceeded = 1,
}

/// Check that `call_depth` does not exceed `MAX_CALL_DEPTH`.
///
/// Returns `Ok(call_depth + 1)` (the depth to pass to the next callee) on
/// success, or `Err(CallDepthError::CallDepthExceeded)` if the limit is hit.
///
/// # Usage
/// ```ignore
/// let next_depth = check_call_depth(call_depth)?;
/// // pass next_depth to the downstream cross-contract call
/// ```
pub fn check_call_depth(call_depth: u32) -> Result<u32, CallDepthError> {
    if call_depth > MAX_CALL_DEPTH {
        return Err(CallDepthError::CallDepthExceeded);
    }
    Ok(call_depth.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_within_limit_succeeds() {
        // Depths 0..=5 should all succeed and return depth+1.
        for d in 0..=MAX_CALL_DEPTH {
            let result = check_call_depth(d);
            assert!(result.is_ok(), "expected Ok for depth {d}");
            assert_eq!(result.unwrap(), d + 1);
        }
    }

    #[test]
    fn depth_at_limit_succeeds() {
        assert!(check_call_depth(MAX_CALL_DEPTH).is_ok());
    }

    #[test]
    fn depth_exceeds_limit_returns_error() {
        let result = check_call_depth(MAX_CALL_DEPTH + 1);
        assert_eq!(result, Err(CallDepthError::CallDepthExceeded));
    }

    #[test]
    fn simulated_call_chain_depth_5_succeeds() {
        // Simulate a chain of 5 nested calls (depths 0→1→2→3→4→5).
        let mut depth = 0u32;
        for _ in 0..5 {
            depth = check_call_depth(depth).expect("should not exceed limit");
        }
        assert_eq!(depth, 5);
    }

    #[test]
    fn simulated_call_chain_depth_6_fails() {
        let mut depth = 0u32;
        for _ in 0..5 {
            depth = check_call_depth(depth).expect("should not exceed limit");
        }
        // 6th call should fail
        let result = check_call_depth(depth);
        assert_eq!(result, Err(CallDepthError::CallDepthExceeded));
    }
}
