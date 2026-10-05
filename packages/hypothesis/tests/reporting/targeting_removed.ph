import { Phase, Property, Settings } from "hypothesis"

Assert.false(Property.respondsTo(#target))
Assert.true(Phase.Target != None)
Assert.false(Settings.standard.phases.includes(Phase.Target))
