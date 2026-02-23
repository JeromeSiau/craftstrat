import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const operations = ['+', '-', '*', '/', '%', 'min', 'max', 'abs'] as const;

type MathNodeData = {
    operation: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function MathNode({ id, data }: NodeProps & { data: MathNodeData }) {
    return (
        <div className="rounded-md border-2 border-orange-400 bg-orange-50 p-2 shadow-sm dark:border-orange-600 dark:bg-orange-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-orange-600 dark:text-orange-400">
                Math
            </div>
            <Select
                value={data.operation as string}
                onValueChange={(value) => data.onUpdate(id, { ...data, operation: value })}
            >
                <SelectTrigger className="h-7 text-xs">
                    <SelectValue placeholder="Op" />
                </SelectTrigger>
                <SelectContent>
                    {operations.map((op) => (
                        <SelectItem key={op} value={op}>
                            {op}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
            <Handle type="target" id="a" position={Position.Left} className="!bg-orange-400" style={{ top: '30%' }} />
            <Handle type="target" id="b" position={Position.Left} className="!bg-orange-400" style={{ top: '70%' }} />
            <Handle type="source" position={Position.Right} className="!bg-orange-400" />
        </div>
    );
}
