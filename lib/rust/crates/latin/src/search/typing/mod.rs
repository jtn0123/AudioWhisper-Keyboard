//! `suggest/policyimpl/typing`: the policy of tap typing: [`traversal`] (when to correct and
//! cache), [`weighting`] (what each correction costs) and [`scoring`] (the final score). The
//! constants, `scoring_params`, are in the parent module. `TypingSuggestPolicy` and its factory
//! only choose between these and the gesture policy, which is not ported, so they are not either.

pub mod scoring;
pub mod traversal;
pub mod weighting;
