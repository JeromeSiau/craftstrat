import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { indicators } from '@/components/strategy/indicator-options';

type InputNodeData = {
    field: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function InputNode({ id, data }: NodeProps & { data: InputNodeData }) {
    return (
        <div className="rounded-md border border-gray-300 bg-white p-2 shadow-sm dark:border-gray-600 dark:bg-gray-800">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Input
            </div>
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
            <Handle type="source" position={Position.Right} className="!bg-gray-400" />
        </div>
    );
}
