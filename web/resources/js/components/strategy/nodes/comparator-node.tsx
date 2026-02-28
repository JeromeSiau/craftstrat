import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { operators } from '@/components/strategy/indicator-options';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

type ComparatorNodeData = {
    operator: string;
    value: number | [number, number];
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function ComparatorNode({ id, data }: NodeProps & { data: ComparatorNodeData }) {
    const isBetween = data.operator === 'between';
    const betweenValue = Array.isArray(data.value) ? data.value : [0, 0];

    return (
        <div className="rounded-md border border-gray-300 bg-white p-2 shadow-sm dark:border-gray-600 dark:bg-gray-800">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Compare
            </div>
            <div className="space-y-1">
                <Select
                    value={data.operator as string}
                    onValueChange={(value) => {
                        const newValue = value === 'between'
                            ? (Array.isArray(data.value) ? data.value : [0, 0])
                            : (Array.isArray(data.value) ? data.value[0] : data.value);
                        data.onUpdate(id, { ...data, operator: value, value: newValue });
                    }}
                >
                    <SelectTrigger className="h-7 text-xs">
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
                    <div className="flex gap-1">
                        <Input
                            type="number"
                            step="0.1"
                            value={betweenValue[0]}
                            onChange={(e) => data.onUpdate(id, { ...data, value: [Number(e.target.value), betweenValue[1]] })}
                            placeholder="Min"
                            className="h-7 text-xs"
                        />
                        <Input
                            type="number"
                            step="0.1"
                            value={betweenValue[1]}
                            onChange={(e) => data.onUpdate(id, { ...data, value: [betweenValue[0], Number(e.target.value)] })}
                            placeholder="Max"
                            className="h-7 text-xs"
                        />
                    </div>
                ) : (
                    <Input
                        type="number"
                        step="0.1"
                        value={Array.isArray(data.value) ? data.value[0] : (data.value as number)}
                        onChange={(e) => data.onUpdate(id, { ...data, value: Number(e.target.value) })}
                        placeholder="Value"
                        className="h-7 text-xs"
                    />
                )}
            </div>
            <Handle type="target" position={Position.Left} className="!bg-gray-400" />
            <Handle type="source" position={Position.Right} className="!bg-gray-400" />
        </div>
    );
}
