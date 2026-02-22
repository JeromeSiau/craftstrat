import { Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { indicators, operators } from '@/components/strategy/indicator-options';
import { safeParseFloat } from '@/lib/formatters';
import type { StrategyRule } from '@/types/models';

interface RuleRowProps {
    rule: StrategyRule;
    onChange: (rule: StrategyRule) => void;
    onRemove: () => void;
}

const indicatorsByCategory = indicators.reduce(
    (acc, indicator) => {
        if (!acc[indicator.category]) {
            acc[indicator.category] = [];
        }
        acc[indicator.category].push(indicator);
        return acc;
    },
    {} as Record<string, (typeof indicators)[number][]>,
);

export default function RuleRow({ rule, onChange, onRemove }: RuleRowProps) {
    const isBetween = rule.operator === 'between';

    function handleIndicatorChange(value: string): void {
        onChange({ ...rule, indicator: value });
    }

    function handleOperatorChange(value: string): void {
        if (value === 'between') {
            const currentValue = typeof rule.value === 'number' ? rule.value : rule.value[0];
            onChange({ ...rule, operator: value, value: [currentValue, currentValue + 1] });
        } else {
            const currentValue = typeof rule.value === 'number' ? rule.value : rule.value[0];
            onChange({ ...rule, operator: value, value: currentValue });
        }
    }

    function handleValueChange(value: string): void {
        onChange({ ...rule, value: safeParseFloat(value) });
    }

    function handleBetweenMinChange(value: string): void {
        const currentMax = Array.isArray(rule.value) ? rule.value[1] : 0;
        onChange({ ...rule, value: [safeParseFloat(value), currentMax] });
    }

    function handleBetweenMaxChange(value: string): void {
        const currentMin = Array.isArray(rule.value) ? rule.value[0] : 0;
        onChange({ ...rule, value: [currentMin, safeParseFloat(value)] });
    }

    return (
        <div className="flex items-center gap-2">
            <Select value={rule.indicator} onValueChange={handleIndicatorChange}>
                <SelectTrigger className="w-44">
                    <SelectValue placeholder="Indicator" />
                </SelectTrigger>
                <SelectContent>
                    {Object.entries(indicatorsByCategory).map(([category, items]) => (
                        <SelectGroup key={category}>
                            <SelectLabel>{category}</SelectLabel>
                            {items.map((indicator) => (
                                <SelectItem key={indicator.value} value={indicator.value}>
                                    {indicator.label}
                                </SelectItem>
                            ))}
                        </SelectGroup>
                    ))}
                </SelectContent>
            </Select>

            <Select value={rule.operator} onValueChange={handleOperatorChange}>
                <SelectTrigger className="w-28">
                    <SelectValue placeholder="Operator" />
                </SelectTrigger>
                <SelectContent>
                    {operators.map((op) => (
                        <SelectItem key={op.value} value={op.value}>
                            {op.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>

            {isBetween ? (
                <div className="flex items-center gap-1">
                    <Input
                        type="number"
                        step="any"
                        className="w-24"
                        value={Array.isArray(rule.value) ? rule.value[0] : rule.value}
                        onChange={(e) => handleBetweenMinChange(e.target.value)}
                    />
                    <span className="text-sm text-muted-foreground">and</span>
                    <Input
                        type="number"
                        step="any"
                        className="w-24"
                        value={Array.isArray(rule.value) ? rule.value[1] : 0}
                        onChange={(e) => handleBetweenMaxChange(e.target.value)}
                    />
                </div>
            ) : (
                <Input
                    type="number"
                    step="any"
                    className="w-28"
                    value={typeof rule.value === 'number' ? rule.value : rule.value[0]}
                    onChange={(e) => handleValueChange(e.target.value)}
                />
            )}

            <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={onRemove}
                className="shrink-0 text-muted-foreground hover:text-destructive"
            >
                <Trash2 className="size-4" />
            </Button>
        </div>
    );
}
