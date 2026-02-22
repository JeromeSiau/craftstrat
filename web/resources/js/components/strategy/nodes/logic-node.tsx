import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

type LogicNodeData = {
    operator: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function LogicNode({ id, data }: NodeProps & { data: LogicNodeData }) {
    return (
        <div className="rounded-md border-2 border-blue-400 bg-blue-50 p-2 shadow-sm dark:border-blue-600 dark:bg-blue-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-blue-600 dark:text-blue-400">
                Logic
            </div>
            <Select
                value={data.operator as string}
                onValueChange={(value) => data.onUpdate(id, { ...data, operator: value })}
            >
                <SelectTrigger className="h-7 text-xs">
                    <SelectValue placeholder="Operator" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="AND">AND</SelectItem>
                    <SelectItem value="OR">OR</SelectItem>
                </SelectContent>
            </Select>
            <Handle type="target" position={Position.Left} className="!bg-blue-400" />
            <Handle type="source" position={Position.Right} className="!bg-blue-400" />
        </div>
    );
}
