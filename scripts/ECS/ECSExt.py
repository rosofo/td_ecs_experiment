CustomParHelper: CustomParHelper = (
    next(d for d in me.docked if "ExtUtils" in d.tags)
    .mod("CustomParHelper")
    .CustomParHelper
)  # import
NoNode: NoNode = (
    next(d for d in me.docked if "ExtUtils" in d.tags).mod("NoNode").NoNode
)  # import
###

from dataclasses import dataclass
from typing import Literal
import ecs
import importlib

ecs = importlib.reload(ecs)
from ecs import ecs


@dataclass
class ECSChange:
    type: Literal["inserted", "removed", "despawned"]
    op: int
    component: str | None


class ECSExt:
    # A mapping of components to the ops they're attached to
    lastComponents: dict[int, list[int]]
    # A mapping of components to their types
    knownInstances: dict[int, str]
    # Ops with components attached
    lastOps: set[int]

    def __init__(self, ownerComp):
        self.ownerComp = ownerComp
        self.World = ecs.World()
        self.lastComponents = {}
        self.knownInstances = {}
        self.lastOps = set()

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

    def getCurrentComponents(self) -> dict[int, list[int]]:
        return {
            int(id): op(int(id)).Ops for id in op("opfind1").col("id", val=True)[1:]
        }

    def collectChanges(self) -> list[ECSChange]:
        changes = []
        components = self.getCurrentComponents()

        deleted_last_ops = {op_id for op_id in self.lastOps if op(op_id) is None}
        for op_id in deleted_last_ops:
            changes.append(ECSChange("despawned", op_id, None))

        deleted_comp_set = set(self.lastComponents.keys()).difference(components)
        for comp_id in deleted_comp_set:
            ops_to_remove_from = self.lastComponents.get(comp_id)

            component = self.knownInstances.get(comp_id)

            if ops_to_remove_from and component is not None:
                # skip redundant removals on despawned ops
                ops_to_remove_from = set(ops_to_remove_from) - deleted_last_ops
                for op_id in ops_to_remove_from:
                    changes.append(ECSChange("removed", op_id, component))

        for comp_id, ops in components.items():
            component = op(comp_id).Component
            self.knownInstances[comp_id] = component
            added_set = set(ops).difference(set(self.lastComponents.get(comp_id, [])))
            removed_set = (
                set(self.lastComponents.get(comp_id, []))
                .difference(set(ops))
                .difference(
                    deleted_last_ops
                )  # skip redundant removals on despawned ops
            )

            for op_id in added_set:
                changes.append(ECSChange("inserted", op_id, component))
            for op_id in removed_set:
                changes.append(ECSChange("removed", op_id, component))

        debug(changes)
        self.lastComponents = components
        ops_with_comps_set = {op_id for ops in components.values() for op_id in ops}
        self.lastOps = ops_with_comps_set
        return changes

    def UpdateWorld(self):
        changes = self.collectChanges()
        for change in changes:
            if change.type == "inserted":
                if change.component == "random":
                    self.World.insert_random(change.op)
                elif change.component == "sample":
                    self.World.insert_sample(change.op, 300, "amp")
                elif change.component == "apply":
                    self.World.insert_apply(change.op, "amp")
            elif change.type == "removed":
                getattr(self.World, f"remove_{change.component}")(change.op)
            elif change.type == "despawned":
                self.World.despawn(change.op)
