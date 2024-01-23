//! Source: https://github.com/facebook/winterfell/tree/main/examples/src/fibonacci/fib2
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod winter {
    use core::marker::PhantomData;
    use ink::prelude::{vec, vec::Vec};
    use winterfell::{
        crypto::{hashers::Blake3_256, DefaultRandomCoin, ElementHasher},
        math::{fields::f128::BaseElement, FieldElement},
        AcceptableOptions, Air, AirContext, Assertion, EvaluationFrame, ProofOptions, StarkProof,
        TraceInfo, TransitionConstraintDegree,
    };

    // FIBONACCI AIR
    // ================================================================================================

    pub struct FibAir {
        context: AirContext<BaseElement>,
        result: BaseElement,
    }

    impl Air for FibAir {
        type BaseField = BaseElement;
        type PublicInputs = BaseElement;

        fn new(trace_info: TraceInfo, pub_inputs: Self::BaseField, options: ProofOptions) -> Self {
            let degrees = vec![
                TransitionConstraintDegree::new(1),
                TransitionConstraintDegree::new(1),
            ];
            FibAir {
                context: AirContext::new(trace_info, degrees, 3, options),
                result: pub_inputs,
            }
        }

        fn context(&self) -> &AirContext<Self::BaseField> {
            &self.context
        }

        #[allow(clippy::arithmetic_side_effects)]
        fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
            &self,
            frame: &EvaluationFrame<E>,
            _periodic_values: &[E],
            result: &mut [E],
        ) {
            let current = frame.current();
            let next = frame.next();
            // expected state width is 2 field elements

            // constraints of Fibonacci sequence (2 terms per step):
            // s_{0, i+1} = s_{0, i} + s_{1, i}
            // s_{1, i+1} = s_{1, i} + s_{0, i+1}
            result[0] = are_equal(next[0], current[0] + current[1]);
            result[1] = are_equal(next[1], current[1] + next[0]);
        }

        fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
            // a valid Fibonacci sequence should start with two ones and terminate with
            // the expected result
            let last_step = self.trace_length().checked_sub(1).unwrap();
            vec![
                Assertion::single(0, 0, Self::BaseField::ONE),
                Assertion::single(1, 0, Self::BaseField::ONE),
                Assertion::single(1, last_step, self.result),
            ]
        }
    }

    /// Returns zero only when a == b.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn are_equal<E: FieldElement>(a: E, b: E) -> E {
        a - b
    }

    struct FibExample<H: ElementHasher>(PhantomData<H>);

    impl<H: ElementHasher<BaseField = BaseElement>> FibExample<H> {
        pub fn verify(proof: StarkProof, pub_inputs: BaseElement) -> bool {
            let acceptable_options = AcceptableOptions::OptionSet(vec![proof.options().clone()]);
            winterfell::verify::<FibAir, H, DefaultRandomCoin<H>>(
                proof,
                pub_inputs,
                &acceptable_options,
            )
            .is_ok()
        }
    }

    #[ink(storage)]
    pub struct Winter {}

    impl Winter {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn verify(&self, value: u128, proof: Vec<u8>) -> bool {
            FibExample::<Blake3_256<BaseElement>>::verify(
                StarkProof::from_bytes(&proof).unwrap(),
                BaseElement::new(value),
            )
        }
    }
}
