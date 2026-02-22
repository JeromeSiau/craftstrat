import { Plus, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import RuleRow from '@/components/strategy/rule-row';
import { uid } from '@/lib/formatters';
import type { ConditionGroup as ConditionGroupType, StrategyRule } from '@/types/models';

interface ConditionGroupProps {
    group: ConditionGroupType;
    index: number;
    onChange: (group: ConditionGroupType) => void;
    onRemove: () => void;
}

export default function ConditionGroup({ group, index, onChange, onRemove }: ConditionGroupProps) {
    function handleTypeChange(value: string): void {
        onChange({ ...group, type: value as 'AND' | 'OR' });
    }

    function handleRuleChange(ruleIndex: number, rule: StrategyRule): void {
        const updatedRules = [...group.rules];
        updatedRules[ruleIndex] = rule;
        onChange({ ...group, rules: updatedRules });
    }

    function handleRuleRemove(ruleIndex: number): void {
        if (group.rules.length <= 1) {
            return;
        }
        const updatedRules = group.rules.filter((_, i) => i !== ruleIndex);
        onChange({ ...group, rules: updatedRules });
    }

    function handleAddRule(): void {
        onChange({
            ...group,
            rules: [...group.rules, { id: uid(), indicator: 'abs_move_pct', operator: '>', value: 0 }],
        });
    }

    return (
        <Card>
            <CardHeader className="flex-row items-center justify-between">
                <div className="flex items-center gap-3">
                    <CardTitle className="text-sm">Group {index + 1}</CardTitle>
                    <Select value={group.type} onValueChange={handleTypeChange}>
                        <SelectTrigger className="w-24">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="AND">AND</SelectItem>
                            <SelectItem value="OR">OR</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
                <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    onClick={onRemove}
                    className="text-muted-foreground hover:text-destructive"
                >
                    <Trash2 className="size-4" />
                </Button>
            </CardHeader>
            <CardContent className="space-y-3">
                {group.rules.map((rule, ruleIndex) => (
                    <RuleRow
                        key={rule.id}
                        rule={rule}
                        onChange={(updatedRule) => handleRuleChange(ruleIndex, updatedRule)}
                        onRemove={() => handleRuleRemove(ruleIndex)}
                    />
                ))}
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleAddRule}
                >
                    <Plus className="size-4" />
                    Add Rule
                </Button>
            </CardContent>
        </Card>
    );
}
