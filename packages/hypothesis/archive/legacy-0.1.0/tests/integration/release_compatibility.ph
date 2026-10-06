// Phase 12 release fixture: broad-v1 names remain direct authoritative aliases.
import { Check, CheckConfig, PropertyReporter, RuleBasedStateMachine, Invariant } from "hypothesis"

const settings = CheckConfig.standard.examples(10).seed(20260723)
const machineType = RuleBasedStateMachine
const invariantAttribute = Invariant
const reporter = PropertyReporter.console
System.print("PASS release compatibility")
