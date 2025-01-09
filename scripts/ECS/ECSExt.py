CustomParHelper: CustomParHelper = (
    next(d for d in me.docked if "ExtUtils" in d.tags)
    .mod("CustomParHelper")
    .CustomParHelper
)  # import
NoNode: NoNode = (
    next(d for d in me.docked if "ExtUtils" in d.tags).mod("NoNode").NoNode
)  # import
###

import ecs


class ECSExt:
    def __init__(self, ownerComp):
        self.ownerComp = ownerComp
        self.World = ecs.ecs.World()

        CustomParHelper.Init(
            self, ownerComp, enable_properties=True, enable_callbacks=True
        )
        NoNode.Init(
            ownerComp,
            enable_chopexec=True,
            enable_datexec=True,
            enable_parexec=True,
            enable_keyboard_shortcuts=True,
        )
