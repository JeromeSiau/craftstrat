import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import ConditionGroup from '@/components/strategy/condition-group';
import ActionConfig from '@/components/strategy/action-config';
import RiskConfig from '@/components/strategy/risk-config';
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

const defaultConditionGroup: ConditionGroupType = {
    type: 'AND',
    rules: [{ indicator: 'abs_move_pct', operator: '>', value: 0 }],
};

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
            conditions: [...graph.conditions, { ...defaultConditionGroup, rules: [...defaultConditionGroup.rules] }],
        });
    }

    function handleActionChange(action: StrategyAction): void {
        onChange({ ...graph, action });
    }

    function handleRiskChange(risk: StrategyRisk): void {
        onChange({ ...graph, risk });
    }

    return (
        <div className="space-y-6">
            <div className="space-y-4">
                <div className="flex items-center justify-between">
                    <h3 className="text-sm font-semibold">Conditions (SI)</h3>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleAddConditionGroup}
                    >
                        <Plus className="size-4" />
                        Add Condition Group
                    </Button>
                </div>
                {graph.conditions.map((group, index) => (
                    <ConditionGroup
                        key={index}
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
