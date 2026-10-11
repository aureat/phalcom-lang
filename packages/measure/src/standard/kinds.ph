//! Standard semantic quantity kinds and coherent unambiguous algebraic laws.
//!
//! Equal dimensions do not imply equal kinds: Energy/Torque, Frequency/
//! Radioactivity, and AbsorbedDose/EquivalentDose intentionally remain distinct.

from ..algebra.kind import QuantityKind, ProductKind, QuotientKind, PointKind
from ..dimension.dimension import Dimension

let lengthD = Dimension.base("measure.si.length")
let massD = Dimension.base("measure.si.mass")
let timeD = Dimension.base("measure.si.time")
let currentD = Dimension.base("measure.si.current")
let temperatureD = Dimension.base("measure.si.temperature")
let substanceD = Dimension.base("measure.si.substance")
let luminousD = Dimension.base("measure.si.luminous")
let informationD = Dimension.base("measure.information")

data Scalar()
data Length()
data Area()
data Volume()
data Mass()
data ElapsedTime()
data Velocity()
data Acceleration()
data Force()
data Pressure()
data Energy()
data Torque()
data Power()
data Frequency()
data Temperature()
data TemperatureDifference()
data TemperatureRate()
data ElectricCurrent()
data ElectricCharge()
data Voltage()
data Resistance()
data Conductance()
data Capacitance()
data MagneticFlux()
data MagneticFluxDensity()
data Inductance()
data AmountOfSubstance()
data CatalyticActivity()
data LuminousIntensity()
data LuminousFlux()
data Illuminance()
data Angle()
data SolidAngle()
data Radioactivity()
data AbsorbedDose()
data EquivalentDose()
data Information()
data DataRate()

impl QuantityKind for Scalar {
  name -> String { "Scalar" }
  dimension -> Dimension { Dimension.one }
}

impl QuantityKind for Length {
  name -> String { "Length" }
  dimension -> Dimension { lengthD }
}

impl QuantityKind for Area {
  name -> String { "Area" }
  dimension -> Dimension { lengthD ** 2 }
}

impl QuantityKind for Volume {
  name -> String { "Volume" }
  dimension -> Dimension { lengthD ** 3 }
}

impl QuantityKind for Mass {
  name -> String { "Mass" }
  dimension -> Dimension { massD }
}

impl QuantityKind for ElapsedTime {
  name -> String { "ElapsedTime" }
  dimension -> Dimension { timeD }
}

impl QuantityKind for Velocity {
  name -> String { "Velocity" }
  dimension -> Dimension { lengthD / timeD }
}

impl QuantityKind for Acceleration {
  name -> String { "Acceleration" }
  dimension -> Dimension { lengthD / (timeD ** 2) }
}

impl QuantityKind for Force {
  name -> String { "Force" }
  dimension -> Dimension { massD * lengthD / (timeD ** 2) }
}

impl QuantityKind for Pressure {
  name -> String { "Pressure" }
  dimension -> Dimension { Force().dimension / (lengthD ** 2) }
}

impl QuantityKind for Energy {
  name -> String { "Energy" }
  dimension -> Dimension { Force().dimension * lengthD }
}

impl QuantityKind for Torque {
  name -> String { "Torque" }
  dimension -> Dimension { Energy().dimension }
}

impl QuantityKind for Power {
  name -> String { "Power" }
  dimension -> Dimension { Energy().dimension / timeD }
}

impl QuantityKind for Frequency {
  name -> String { "Frequency" }
  dimension -> Dimension { Dimension.one / timeD }
}

impl QuantityKind for TemperatureDifference {
  name -> String { "TemperatureDifference" }
  dimension -> Dimension { temperatureD }
}

impl QuantityKind for TemperatureRate {
  name -> String { "TemperatureRate" }
  dimension -> Dimension { temperatureD / timeD }
}

impl PointKind for Temperature {
  type Difference = TemperatureDifference

  name -> String { "Temperature" }
  dimension -> Dimension { temperatureD }
  differenceKind -> TemperatureDifference { TemperatureDifference() }
}

impl QuantityKind for ElectricCurrent {
  name -> String { "ElectricCurrent" }
  dimension -> Dimension { currentD }
}

impl QuantityKind for ElectricCharge {
  name -> String { "ElectricCharge" }
  dimension -> Dimension { currentD * timeD }
}

impl QuantityKind for Voltage {
  name -> String { "Voltage" }
  dimension -> Dimension { Power().dimension / currentD }
}

impl QuantityKind for Resistance {
  name -> String { "Resistance" }
  dimension -> Dimension { Voltage().dimension / currentD }
}

impl QuantityKind for Conductance {
  name -> String { "Conductance" }
  dimension -> Dimension { currentD / Voltage().dimension }
}

impl QuantityKind for Capacitance {
  name -> String { "Capacitance" }
  dimension -> Dimension { ElectricCharge().dimension / Voltage().dimension }
}

impl QuantityKind for MagneticFlux {
  name -> String { "MagneticFlux" }
  dimension -> Dimension { Voltage().dimension * timeD }
}

impl QuantityKind for MagneticFluxDensity {
  name -> String { "MagneticFluxDensity" }
  dimension -> Dimension { MagneticFlux().dimension / (lengthD ** 2) }
}

impl QuantityKind for Inductance {
  name -> String { "Inductance" }
  dimension -> Dimension { MagneticFlux().dimension / currentD }
}

impl QuantityKind for AmountOfSubstance {
  name -> String { "AmountOfSubstance" }
  dimension -> Dimension { substanceD }
}

impl QuantityKind for CatalyticActivity {
  name -> String { "CatalyticActivity" }
  dimension -> Dimension { substanceD / timeD }
}

impl QuantityKind for LuminousIntensity {
  name -> String { "LuminousIntensity" }
  dimension -> Dimension { luminousD }
}

impl QuantityKind for LuminousFlux {
  name -> String { "LuminousFlux" }
  dimension -> Dimension { luminousD }
}

impl QuantityKind for Illuminance {
  name -> String { "Illuminance" }
  dimension -> Dimension { luminousD / (lengthD ** 2) }
}

impl QuantityKind for Angle {
  name -> String { "Angle" }
  dimension -> Dimension { Dimension.one }
}

impl QuantityKind for SolidAngle {
  name -> String { "SolidAngle" }
  dimension -> Dimension { Dimension.one }
}

impl QuantityKind for Radioactivity {
  name -> String { "Radioactivity" }
  dimension -> Dimension { Frequency().dimension }
}

impl QuantityKind for AbsorbedDose {
  name -> String { "AbsorbedDose" }
  dimension -> Dimension { (lengthD ** 2) / (timeD ** 2) }
}

impl QuantityKind for EquivalentDose {
  name -> String { "EquivalentDose" }
  dimension -> Dimension { AbsorbedDose().dimension }
}

impl QuantityKind for Information {
  name -> String { "Information" }
  dimension -> Dimension { informationD }
}

impl QuantityKind for DataRate {
  name -> String { "DataRate" }
  dimension -> Dimension { informationD / timeD }
}

impl ProductKind<Length> for Length {
  type Output = Area
  productKind(_ other: Length) -> Area { Area() }
}

impl ProductKind<Length> for Area {
  type Output = Volume
  productKind(_ other: Length) -> Volume { Volume() }
}

impl QuotientKind<ElapsedTime> for Length {
  type Output = Velocity
  quotientKind(_ other: ElapsedTime) -> Velocity { Velocity() }
}

impl QuotientKind<Velocity> for Length {
  type Output = ElapsedTime
  quotientKind(_ other: Velocity) -> ElapsedTime { ElapsedTime() }
}

impl ProductKind<ElapsedTime> for Velocity {
  type Output = Length
  productKind(_ other: ElapsedTime) -> Length { Length() }
}

impl ProductKind<Velocity> for ElapsedTime {
  type Output = Length
  productKind(_ other: Velocity) -> Length { Length() }
}

impl QuotientKind<ElapsedTime> for Velocity {
  type Output = Acceleration
  quotientKind(_ other: ElapsedTime) -> Acceleration { Acceleration() }
}

impl ProductKind<Acceleration> for Mass {
  type Output = Force
  productKind(_ other: Acceleration) -> Force { Force() }
}

impl ProductKind<Mass> for Acceleration {
  type Output = Force
  productKind(_ other: Mass) -> Force { Force() }
}

impl QuotientKind<Area> for Force {
  type Output = Pressure
  quotientKind(_ other: Area) -> Pressure { Pressure() }
}

impl QuotientKind<ElapsedTime> for Energy {
  type Output = Power
  quotientKind(_ other: ElapsedTime) -> Power { Power() }
}

impl QuotientKind<Power> for Energy {
  type Output = ElapsedTime
  quotientKind(_ other: Power) -> ElapsedTime { ElapsedTime() }
}

impl ProductKind<ElapsedTime> for Power {
  type Output = Energy
  productKind(_ other: ElapsedTime) -> Energy { Energy() }
}

impl ProductKind<Power> for ElapsedTime {
  type Output = Energy
  productKind(_ other: Power) -> Energy { Energy() }
}

impl QuotientKind<ElapsedTime> for TemperatureDifference {
  type Output = TemperatureRate
  quotientKind(_ other: ElapsedTime) -> TemperatureRate { TemperatureRate() }
}

impl QuotientKind<TemperatureRate> for TemperatureDifference {
  type Output = ElapsedTime
  quotientKind(_ other: TemperatureRate) -> ElapsedTime { ElapsedTime() }
}

impl ProductKind<ElapsedTime> for ElectricCurrent {
  type Output = ElectricCharge
  productKind(_ other: ElapsedTime) -> ElectricCharge { ElectricCharge() }
}

impl ProductKind<ElectricCurrent> for ElapsedTime {
  type Output = ElectricCharge
  productKind(_ other: ElectricCurrent) -> ElectricCharge { ElectricCharge() }
}

impl QuotientKind<ElapsedTime> for ElectricCharge {
  type Output = ElectricCurrent
  quotientKind(_ other: ElapsedTime) -> ElectricCurrent { ElectricCurrent() }
}

impl QuotientKind<ElectricCurrent> for ElectricCharge {
  type Output = ElapsedTime
  quotientKind(_ other: ElectricCurrent) -> ElapsedTime { ElapsedTime() }
}

impl QuotientKind<ElectricCurrent> for Power {
  type Output = Voltage
  quotientKind(_ other: ElectricCurrent) -> Voltage { Voltage() }
}

impl QuotientKind<ElectricCurrent> for Voltage {
  type Output = Resistance
  quotientKind(_ other: ElectricCurrent) -> Resistance { Resistance() }
}

impl QuotientKind<Voltage> for ElectricCurrent {
  type Output = Conductance
  quotientKind(_ other: Voltage) -> Conductance { Conductance() }
}

impl QuotientKind<Voltage> for ElectricCharge {
  type Output = Capacitance
  quotientKind(_ other: Voltage) -> Capacitance { Capacitance() }
}

impl ProductKind<ElapsedTime> for Voltage {
  type Output = MagneticFlux
  productKind(_ other: ElapsedTime) -> MagneticFlux { MagneticFlux() }
}

impl QuotientKind<Area> for MagneticFlux {
  type Output = MagneticFluxDensity
  quotientKind(_ other: Area) -> MagneticFluxDensity { MagneticFluxDensity() }
}

impl QuotientKind<ElectricCurrent> for MagneticFlux {
  type Output = Inductance
  quotientKind(_ other: ElectricCurrent) -> Inductance { Inductance() }
}

impl QuotientKind<ElapsedTime> for AmountOfSubstance {
  type Output = CatalyticActivity
  quotientKind(_ other: ElapsedTime) -> CatalyticActivity { CatalyticActivity() }
}

impl QuotientKind<ElapsedTime> for Information {
  type Output = DataRate
  quotientKind(_ other: ElapsedTime) -> DataRate { DataRate() }
}

impl QuotientKind<DataRate> for Information {
  type Output = ElapsedTime
  quotientKind(_ other: DataRate) -> ElapsedTime { ElapsedTime() }
}

impl ProductKind<ElapsedTime> for DataRate {
  type Output = Information
  productKind(_ other: ElapsedTime) -> Information { Information() }
}

impl ProductKind<DataRate> for ElapsedTime {
  type Output = Information
  productKind(_ other: DataRate) -> Information { Information() }
}


impl ProductKind<ElectricCurrent> for Voltage {
  type Output = Power
  productKind(_ other: ElectricCurrent) -> Power { Power() }
}

impl ProductKind<Voltage> for ElectricCurrent {
  type Output = Power
  productKind(_ other: Voltage) -> Power { Power() }
}

impl ProductKind<ElectricCurrent> for Resistance {
  type Output = Voltage
  productKind(_ other: ElectricCurrent) -> Voltage { Voltage() }
}

impl QuotientKind<Resistance> for Voltage {
  type Output = ElectricCurrent
  quotientKind(_ other: Resistance) -> ElectricCurrent { ElectricCurrent() }
}

impl ProductKind<Voltage> for Conductance {
  type Output = ElectricCurrent
  productKind(_ other: Voltage) -> ElectricCurrent { ElectricCurrent() }
}

impl ProductKind<SolidAngle> for LuminousIntensity {
  type Output = LuminousFlux
  productKind(_ other: SolidAngle) -> LuminousFlux { LuminousFlux() }
}

impl QuotientKind<Area> for LuminousFlux {
  type Output = Illuminance
  quotientKind(_ other: Area) -> Illuminance { Illuminance() }
}

export (
  Scalar,
  Length, Area, Volume, Mass, ElapsedTime, Velocity, Acceleration,
  Force, Pressure, Energy, Torque, Power, Frequency,
  Temperature, TemperatureDifference, TemperatureRate,
  ElectricCurrent, ElectricCharge, Voltage, Resistance, Conductance,
  Capacitance, MagneticFlux, MagneticFluxDensity, Inductance,
  AmountOfSubstance, CatalyticActivity,
  LuminousIntensity, LuminousFlux, Illuminance,
  Angle, SolidAngle,
  Radioactivity, AbsorbedDose, EquivalentDose,
  Information, DataRate
)
