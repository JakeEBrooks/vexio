from datetime import datetime, timedelta

import vexio
from vexio.utils import vex_time

now = datetime.now()
vextime = vex_time(now)

vex = vexio.Vex()

global_ = vexio.Global()
global_.add_ref(vexio.VexRef("EXPER", "EXP1387"))
global_.add_ref(vexio.VexRef("SCHEDULING_PARAMS", "SKED1"))
global_.add_ref(vexio.VexRef("PROCEDURES", "STANDARD1"))
global_.add_ref(vexio.VexRef("EOP", "EOP129"))
vex.set_global(global_)

exper = vexio.Block("EXPER")
myexper = vexio.Def("exper_no_111")
myexper.add_param(vexio.Param("exper_nominal_start", [vextime]))
myexper.add_param(
    vexio.Param("exper_nominal_stop", [vex_time(now + timedelta(hours=12))])
)
exper.add_def(myexper)
vex.set_primitive(exper)

print(vex.to_vex())
