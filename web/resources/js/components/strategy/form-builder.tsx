import { Filter, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import ConditionGroup from '@/components/strategy/condition-group';
import ActionConfig from '@/components/strategy/action-config';
import RiskConfig from '@/components/strategy/risk-config';
import { uid } from '@/lib/formatters';
import type {
    FormModeGraph,
    ConditionGroup as ConditionGroupType,
    StrategyAction,
    StrategyRisk,
} from '@/types/models';

interface FormBuilderProps {
    graph: FormModeGraph;
    onChange: (graph: FormModeGraph) => void;
}

export default function FormBuilder({ graph, onChange }: FormBuilderProps) {
    function handleConditionChange(index: number, group: ConditionGroupType): void {
        const updatedConditions = [...graph.conditions];
        updatedConditions[index] = group;
        onChange({ ...graph, conditions: updatedConditions });
    }

    function handleConditionRemove(index: number): void {
        if (graph.conditions.length <= 1) {
            return;
        }
        const updatedConditions = graph.conditions.filter((_, i) => i !== index);
        onChange({ ...graph, conditions: updatedConditions });
    }

    function handleAddConditionGroup(): void {
        onChange({
            ...graph,
            conditions: [
                ...graph.conditions,
                {
                    id: uid(),
                    type: 'AND',
                    rules: [{ id: uid(), indicator: 'abs_move_pct', operator: '>', value: 0 }],
                },
            ],
        });
    }

    function handleActionChange(action: StrategyAction): void {
        onChange({ ...graph, action });
    }

    function handleRiskChange(risk: StrategyRisk): void {
        onChange({ ...graph, risk });
    }

    return (
        <div className="space-y-8">
            <div className="space-y-4">
                <div className="flex items-center justify-between border-b pb-3">
                    <div className="flex items-center gap-3">
                        <div className="rounded-lg bg-blue-500/10 p-2 dark:bg-blue-500/15">
                            <Filter className="size-4 text-blue-600 dark:text-blue-400" />
                        </div>
                        <div>
                            <h3 className="font-semibold">Conditions</h3>
                            <p className="text-sm text-muted-foreground">
                                Define when your strategy should trigger.
                            </p>
                        </div>
                    </div>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleAddConditionGroup}
                    >
                        <Plus className="size-4" />
                        Add Group
                    </Button>
                </div>
                {graph.conditions.map((group, index) => (
                    <ConditionGroup
                        key={group.id}
                        group={group}
                        index={index}
                        onChange={(updatedGroup) => handleConditionChange(index, updatedGroup)}
                        onRemove={() => handleConditionRemove(index)}
                    />
                ))}
            </div>

            <ActionConfig action={graph.action} onChange={handleActionChange} />
            <RiskConfig risk={graph.risk} onChange={handleRiskChange} />
        </div>
    );
}
