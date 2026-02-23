import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { indicators } from '@/components/strategy/indicator-options';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const functions = ['EMA', 'SMA', 'RSI'] as const;

type IndicatorNodeData = {
    fn: string;
    period: number;
    field: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function IndicatorNode({ id, data }: NodeProps & { data: IndicatorNodeData }) {
    return (
        <div className="rounded-md border-2 border-purple-400 bg-purple-50 p-2 shadow-sm dark:border-purple-600 dark:bg-purple-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-purple-600 dark:text-purple-400">
                Indicator
            </div>
            <div className="space-y-1">
                <Select
                    value={data.fn as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, fn: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Function" />
                    </SelectTrigger>
                    <SelectContent>
                        {functions.map((fn) => (
                            <SelectItem key={fn} value={fn}>
                                {fn}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Input
                    type="number"
                    value={data.period as number}
                    onChange={(e) => data.onUpdate(id, { ...data, period: Number(e.target.value) })}
                    placeholder="Period"
                    className="h-7 text-xs"
                />
                <Select
                    value={data.field as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, field: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Field" />
                    </SelectTrigger>
                    <SelectContent>
                        {indicators.map((ind) => (
                            <SelectItem key={ind.value} value={ind.value}>
                                {ind.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <Handle type="source" position={Position.Right} className="!bg-purple-400" />
        </div>
    );
}
