// Stable root façade for Hypothesis for Phalcom 0.1.0.
//
// Every export below delegates directly to an authoritative feature module.
// Compatibility names are aliases only; no compatibility implementation remains.

import coreSettings from "core/settings"
import corePhase from "core/phase"
import coreStatus from "core/status"
import coreFailure from "core/failure"
import coreStatistics from "core/statistics"
import coreErrors from "core/errors"
import choiceModel from "choices/choice"
import exampleModel from "choices/example"
import choiceRequest from "choices/request"
import choiceData from "choices/data"
import choiceProvider from "choices/provider"
import strategyModel from "strategies/strategy"
import strategyCombinators from "strategies/combinators"
import strategyGen from "strategies/gen"
import strategyRegistry from "strategies/registry"
import strategyAttributes from "strategies/attributes"
import engineShrinkPass from "engine/shrink_pass"
import engineShrinker from "engine/shrinker"
import propertyAttributes from "property/attributes"
import propertyAssertion from "property/assertion"
import propertyBuilder from "property/builder"
import propertyDiscovery from "property/discovery"
import propertyRunner from "property/runner"
import reportingEvent from "reporting/event"
import reportingReporter from "reporting/reporter"
import reportingConsole from "reporting/console"
import reportingJson from "reporting/json"
import reportingReproduction from "reporting/reproduction"
import databaseModel from "database/database"
import databaseKey from "database/key"
import databaseMemory from "database/memory"
import databaseDirectory from "database/directory"
import statefulAttributes from "stateful/attributes"
import statefulBundle from "stateful/bundle"
import statefulMachine from "stateful/machine"
import statefulRunner from "stateful/runner"

const Given = propertyAttributes.Given
const GivenArgs = propertyAttributes.GivenArgs
const Case = propertyAttributes.Case
const WithSettings = propertyAttributes.WithSettings
const Settings = coreSettings.Settings
const Phase = corePhase.Phase

const Choice = choiceModel.Choice
const ChoiceRequest = choiceRequest.ChoiceRequest
const Example = exampleModel.Example
const DrawData = choiceData.DrawData
const ChoiceProvider = choiceProvider.ChoiceProvider
const ChoiceProviderFactory = choiceProvider.ChoiceProviderFactory
const SystemRandomChoiceProvider = choiceProvider.SystemRandomChoiceProvider
const ScriptedChoiceProvider = choiceProvider.ScriptedChoiceProvider
const SystemRandomProviderFactory = choiceProvider.SystemRandomProviderFactory
const ScriptedProviderFactory = choiceProvider.ScriptedProviderFactory
const Strategy = strategyModel.Strategy
const StrategyBase = strategyCombinators.StrategyBase
const Gen = strategyGen.Gen
const StrategyRegistry = strategyRegistry.StrategyRegistry
const arbitrary = strategyAttributes.arbitrary
const strategy = strategyAttributes.strategy

const Property = propertyBuilder.Property
const PropertyBuilder = propertyBuilder.PropertyBuilder
const PropertySuite = propertyBuilder.PropertySuite
const Assert = propertyAssertion.Assert
const PropertyAssertionError = propertyAssertion.PropertyAssertionError
const PropertyRunner = propertyRunner.PropertyRunner
const PropertyRun = propertyRunner.PropertyRun
const PropertySuiteResult = propertyRunner.PropertySuiteResult

const PropertyResult = coreStatus.PropertyResult
const PropertyId = propertyDiscovery.PropertyId
const Failure = coreFailure.Failure
const Statistics = coreStatistics.Statistics
const StrategyResolutionError = coreErrors.StrategyResolutionError
const ReporterFailure = coreErrors.ReporterFailure
const PropertyDiscoveryError = coreErrors.PropertyDiscoveryError
const HealthCheckFailure = coreErrors._HealthCheckFailure
const FlakyFailure = coreErrors._FlakyFailure

const ShrinkPass = engineShrinkPass.ShrinkPass
const Shrinker = engineShrinker.Shrinker

const ReportEvent = reportingEvent.ReportEvent
const Reporter = reportingReporter.Reporter
const NullReporter = reportingReporter.NullReporter
const RecordingReporter = reportingReporter.RecordingReporter
const CompositeReporter = reportingReporter.CompositeReporter
const ConsoleReporter = reportingConsole.ConsoleReporter
const PropertyReporter = reportingConsole.PropertyReporter
const JsonReporter = reportingJson.JsonReporter
const ReproductionToken = reportingReproduction.ReproductionToken
const Reproduction = reportingReproduction.Reproduction

const DatabaseKey = databaseKey.DatabaseKey
const ExampleDatabase = databaseModel.ExampleDatabase
const MemoryDatabase = databaseMemory.MemoryDatabase
const DirectoryDatabase = databaseDirectory.DirectoryDatabase

const StateMachine = statefulMachine.StateMachine
const Stateful = statefulRunner.Stateful
const Rule = statefulAttributes.Rule
const Initialize = statefulAttributes.Initialize
const StateInvariant = statefulAttributes.StateInvariant
const When = statefulAttributes.When
const Teardown = statefulAttributes.Teardown
const Bundle = statefulBundle.Bundle

// Compatibility aliases retained as direct authoritative aliases.
const Check = propertyAttributes.WithSettings
const CheckConfig = coreSettings.Settings
const RuleBasedStateMachine = statefulMachine.StateMachine
const Invariant = statefulAttributes.StateInvariant
