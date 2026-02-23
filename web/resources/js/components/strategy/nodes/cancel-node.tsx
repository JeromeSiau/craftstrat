import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

type CancelNodeData = {
    outcome: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function CancelNode({ id, data }: NodeProps & { data: CancelNodeData }) {
    return (
        <div className="rounded-md border-2 border-red-400 bg-red-50 p-2 shadow-sm dark:border-red-600 dark:bg-red-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-red-600 dark:text-red-400">
                Cancel
            </div>
            <div className="space-y-1">
                <Select
                    value={data.outcome as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, outcome: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Outcome" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="UP">UP</SelectItem>
                        <SelectItem value="DOWN">DOWN</SelectItem>
                    </SelectContent>
                </Select>
            </div>
            <Handle type="target" position={Position.Left} className="!bg-red-400" />
        </div>
    );
}
