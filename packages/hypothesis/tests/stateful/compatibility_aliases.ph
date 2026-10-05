// Compatibility aliases delegate to the authoritative implementation.
import Assert from hypothesis
import Invariant from hypothesis
import RuleBasedStateMachine from hypothesis
import StateInvariant from hypothesis
import StateMachine from hypothesis

Assert.equal(StateMachine, RuleBasedStateMachine)
Assert.equal(StateInvariant, Invariant)
