import pyesim as esim

dc = esim.LinearDcAnalysis()
dc.add_resistor(1, 0, 100)
dc.add_resistor(2, 1, 100)
dc.add_independent_voltage_source(2, 0, 5, 0)
voltages, currents = dc.solve()
